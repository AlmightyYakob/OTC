use std::collections::HashSet;

use string_cache::Atom;
use swc_core::testing_transform::test;
use swc_ecma_ast::*;
use swc_ecma_visit::{as_folder, FoldWith, Visit, VisitMut, VisitMutWith, VisitWith};

mod transform;
mod utils;
mod vue;

#[derive(Debug)]
pub struct Visitor {
    options: vue::OptionsComponent,
    composition: vue::CompositionComponent,
    // TODO: Set this to false if there is ever some issue parsing vue file,
    // and skip that file if so
    // valid: bool,
    props_set: Option<HashSet<String>>,
}
impl Default for Visitor {
    fn default() -> Visitor {
        Self {
            options: Default::default(),
            composition: Default::default(),
            // valid: true,
            props_set: Default::default(),
        }
    }
}
impl Visitor {
    fn populate_composition(&mut self) {
        // Pass through components
        if let Some(components) = &self.options.components {
            self.composition.components = Some(components.clone())
        }

        // Pass through props
        if let Some(props) = &self.options.props {
            self.composition.props = Some(props.clone())
        }

        // TODO:
        // Transform inject statements
        // if let Some(injects) = &self.options.inject {
        //     //
        // }

        // Transform data to refs
        if let Some(func) = &self.options.data {
            self.composition.ref_stmts =
                Some(transform::data_to_refs(&func.body.as_ref().unwrap().stmts));
        }

        // Transform computed
        if let Some(computed_decls) = &self.options.computed {
            self.composition.computed = Some(computed_decls.clone());
        }

        // Transform created statements
        if let Some(created) = &self.options.created {
            if let Some(block_stmt) = &created.body {
                self.composition.created_stmts = Some(block_stmt.stmts.clone());
            }
        }

        // Transform mounted
        if let Some(mounted) = &self.options.mounted {
            self.composition.mounted = Some(mounted.clone());
        }
        // Transform methods
        if let Some(methods) = &self.options.methods {
            self.composition.method_decls = Some(methods.clone());
        }
    }

    fn preprocess_default_export(&mut self, object: &ObjectLit) {
        // Build set of prop IDs
        for x in object.props.iter() {
            if let Some(prop) = x.as_prop() {
                if let Prop::KeyValue(kv) = &**prop {
                    if let Some(ident) = kv.key.as_ident() {
                        if ident.sym.to_string().as_str() == "props" {
                            self.props_set = utils::prop_set_from_object_lit(&kv.value);
                        }
                    }
                }
            }
        }
    }

    fn process_default_export(&mut self, object: &ObjectLit) {
        for prop in object.props.iter() {
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
                            "computed" => {
                                if let Expr::Object(obj) = &*kv.value {
                                    let mut computed_decls: Vec<FnDecl> = vec![];
                                    for prop in obj.props.iter() {
                                        if let PropOrSpread::Prop(boxed_expr) = prop {
                                            if let Prop::Method(method_expr) = &**boxed_expr {
                                                if let PropName::Ident(ident) = &method_expr.key {
                                                    computed_decls.push(FnDecl {
                                                        ident: ident.clone(),
                                                        declare: false,
                                                        function: method_expr.function.clone(),
                                                    });
                                                }
                                            }
                                        }
                                    }

                                    // Add to component
                                    if computed_decls.len() > 0 {
                                        self.options.computed = Some(computed_decls);
                                    }
                                }
                            }
                            "methods" => {
                                if let Expr::Object(obj) = &*kv.value {
                                    let mut methods: Vec<FnDecl> = vec![];
                                    for prop in obj.props.iter() {
                                        if let PropOrSpread::Prop(boxed_expr) = prop {
                                            if let Prop::Method(method_expr) = &**boxed_expr {
                                                if let PropName::Ident(ident) = &method_expr.key {
                                                    methods.push(FnDecl {
                                                        ident: ident.clone(),
                                                        declare: false,
                                                        function: method_expr.function.clone(),
                                                    });
                                                }
                                            }
                                        }
                                    }

                                    // Add to component
                                    if methods.len() > 0 {
                                        self.options.methods = Some(methods);
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

// This is used for analysis before modification
impl Visit for Visitor {
    fn visit_module_decl(&mut self, decl: &ModuleDecl) {
        if let Some(expr) = decl.as_export_default_expr() {
            if let Some(obj) = expr.expr.as_object() {
                self.preprocess_default_export(obj);
            }
        }
    }
}

// This is used for the AST modification
impl VisitMut for Visitor {
    // Since functions aren't defined as refs, they must be handled here first
    fn visit_mut_call_expr(&mut self, call_expr: &mut CallExpr) {
        if let Callee::Expr(e) = &call_expr.callee {
            if let Expr::Member(member_expr) = &**e {
                if let (Expr::This(_), MemberProp::Ident(id)) =
                    (&*member_expr.obj, &member_expr.prop)
                {
                    // Simply replace this.method() with method()
                    call_expr.callee = Callee::Expr(Box::new(Expr::Ident(id.clone())));
                }
            }
        }
    }

    // This will convert all uses of `this` to the corresponding refs
    fn visit_mut_member_expr(&mut self, member_expr: &mut MemberExpr) {
        if let (Expr::This(_), MemberProp::Ident(id)) = (&*member_expr.obj, &member_expr.prop) {
            // Ensure that props are treated special
            if self.props_set.is_some() {
                // Replace `this` with `props` if its in the props set
                let set = self.props_set.as_ref().unwrap();
                if set.contains(&id.sym.to_string()) {
                    member_expr.obj = Box::new(Expr::Ident(Ident {
                        optional: false,
                        span: Default::default(),
                        sym: Atom::from("props"),
                    }));

                    // Exit early
                    return;
                }
            }

            // Convert this.foo to foo.value
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

        // Find default export
        let res = module.body.iter().enumerate().find_map(|(index, item)| {
            if !item.is_module_decl() {
                return None;
            }

            let decl = item.as_module_decl().unwrap();
            if !decl.is_export_default_expr() {
                return None;
            }

            let expr = decl.as_export_default_expr().unwrap();
            if !expr.expr.is_object() {
                return None;
            }

            return Some((index, expr.expr.as_object().unwrap().clone()));
        });

        // Exit if not found
        if res.is_none() {
            return;
        }

        // Process default export props
        let (default_export_index, default_export) = res.unwrap();
        self.process_default_export(&default_export);

        // Populate composition
        self.populate_composition();

        // Convert
        module.body[default_export_index] = ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultExpr(
            transform::write_composition_component(&self.composition),
        ))
    }
}

pub fn visit_module(module: Module) -> Module {
    // dbg!(&module);
    let mut visitor = Visitor::default();
    module.visit_with(&mut visitor);

    let mut folder = as_folder(visitor);
    module.fold_with(&mut folder)
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
