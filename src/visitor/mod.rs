use string_cache::Atom;
use swc_core::testing_transform::test;
use swc_ecma_ast::*;
use swc_ecma_visit::{as_folder, FoldWith, VisitMut, VisitMutWith};

mod transform;
mod vue;

#[derive(Debug)]
pub struct Visitor {
    // All the statements that come before the default export (imports, etc.)
    pre_component: Option<Vec<ModuleItem>>,
    options: vue::OptionsComponent,
    composition: vue::CompositionComponent,
}
impl Default for Visitor {
    fn default() -> Visitor {
        Self {
            pre_component: None,
            options: Default::default(),
            composition: Default::default(),
        }
    }
}
// impl Visitor {
//     fn
// }

impl VisitMut for Visitor {
    // This will convert all uses of `this` to the corresponding refs
    // TODO: Don't add .value to non-refs
    fn visit_mut_member_expr(&mut self, member_expr: &mut MemberExpr) {
        if let (Expr::This(_), MemberProp::Ident(id)) = (&*member_expr.obj, &member_expr.prop) {
            member_expr.obj = Box::new(Expr::Ident(id.clone()));
            member_expr.prop = MemberProp::Ident(Ident {
                optional: false,
                span: Default::default(),
                sym: Atom::from("value"),
            });
        }
    }

    fn visit_mut_module(&mut self, module: &mut Module) {
        // Visit children before top level processing
        module.visit_mut_children_with(self);

        // TODO: Place all items aside from default export into pre_component
        // Only visit children of default export
        let maybe_default_export_index = module.body.iter().position(|item| {
            if !item.is_module_decl() {
                return false;
            }

            let decl = item.as_module_decl().unwrap();
            if !decl.is_export_default_expr() {
                return false;
            }

            let expr = decl.as_export_default_expr().unwrap();
            if !expr.expr.is_object() {
                return false;
            }

            return true;
        });

        // Exit if invalid
        if maybe_default_export_index.is_none() {
            return;
        }
        let default_export_index = maybe_default_export_index.unwrap();

        // TODO: Visit entire AST and fill in the Visitor struct.
        // After above are defined, do the following (in this order)
        // * Convert inject into new syntax
        // * Convert data into refs
        // * Convert created method into valid vector of statements
        // * Convert mounted into onMounted call
        // * Convert methods object into vector of function declarations
        //
        // Finally, write all statements into setup and add return statement

        // let object = default_export.unwrap();
        let default_export = module.body[default_export_index]
            .as_module_decl()
            .unwrap()
            .as_export_default_expr()
            .unwrap()
            .expr
            .as_object()
            .unwrap();

        for prop in default_export.props.iter() {
            if !prop.is_prop() {
                continue;
            }

            match &**prop.as_prop().unwrap() {
                Prop::Method(method_prop) => {
                    if let Some(ident) = method_prop.key.as_ident() {
                        match ident.sym.to_string().as_str() {
                            "data" => {
                                self.options.data = Some(method_prop.function.clone());
                            }
                            "created" => {
                                self.options.created = Some(method_prop.function.clone());
                            }
                            "mounted" => {
                                self.options.mounted = Some(method_prop.function.clone());
                            }
                            _ => {}
                        }
                    }
                }
                Prop::KeyValue(kv) => {
                    if let Some(ident) = kv.key.as_ident() {
                        match ident.sym.to_string().as_str() {
                            "components" => {
                                self.options.components = Some(kv.value.clone());
                            }
                            "inject" => {
                                self.options.inject = Some(kv.value.clone());
                            }
                            "props" => {
                                self.options.props = Some(kv.value.clone());
                            }
                            "methods" => {
                                self.options.methods = Some(kv.value.clone());
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        // Pass through components
        if let Some(components) = &self.options.components {
            self.composition.components = Some(components.clone())
        }

        // Pass through props
        if let Some(props) = &self.options.props {
            self.composition.props = Some(props.clone())
        }

        // Transform data to refs
        if let Some(func) = &self.options.data {
            self.composition.ref_stmts =
                Some(transform::data_to_refs(&func.body.as_ref().unwrap().stmts));
        }

        // Transform created statements
        if let Some(created) = &self.options.created {
            if let Some(block_stmt) = &created.body {
                self.composition.created_stmts = Some(block_stmt.stmts.clone());
            }
        }

        // Transform mounted
        if let Some(mounted) = &self.options.mounted {
            if let Some(block_stmt) = &mounted.body {
                self.composition.mounted_stmts = Some(block_stmt.stmts.clone());
            }
        }
        // TODO: Transform methods

        // Convert default export
        module.body[default_export_index] = ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr(
            transform::write_composition_component(&self.composition),
        ))
    }
}

pub fn visit_module(module: Module) -> Module {
    // dbg!(&module);
    module.fold_with(&mut as_folder(Visitor::default()))
}

test!(
    Default::default(),
    |_| as_folder(Visitor::default()),
    data_to_setup,
    // Input codes
    r#"export default {
        data() {
            return {
                loading: false,
                foo: null,
                count: 0,
                headers: [
                    {
                        text: 'Name',
                        value: 'name',
                    },
                    {
                        text: 'Identifier',
                        value: 'identifier',
                    },
                ],
            };
        },
    };"#,
    // Output codes after transformed with plugin
    r#"export default {
        setup() {
            const loading = ref(false);
            const foo = ref(null);
            const count = ref(0);
            const headers = ref([
                {
                    text: 'Name',
                    value: 'name',
                },
                {
                    text: 'Identifier',
                    value: 'identifier',
                },
            ]);

            return {
                loading,
                foo,
                count,
                headers,
            }
        },
    };"#
);