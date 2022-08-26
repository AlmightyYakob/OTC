use string_cache::Atom;
use swc_core::testing_transform::test;
use swc_ecma_ast::*;
use swc_ecma_visit::{as_folder, FoldWith, VisitMut, VisitMutWith};

pub fn data_to_refs(stmts: &Vec<Stmt>) -> Vec<Stmt> {
    let return_expr = stmts
        .iter()
        .find(|stmt| stmt.is_return_stmt() && stmt.as_return_stmt().unwrap().arg.is_some())
        .expect("Data return statement not found!")
        .as_return_stmt()
        .unwrap()
        .arg
        .as_ref()
        .unwrap();

    let props = &return_expr
        .as_object()
        .expect("Data return expr is not an object!")
        .props;
    let ref_callee = Callee::Expr(Box::new(Expr::Ident(Ident {
        optional: false,
        span: Default::default(),
        sym: Atom::from("ref"),
    })));

    // Create new setup statements
    let mut ref_names: Vec<Ident> = Vec::new();
    let mut setup_statements: Vec<Stmt> = Vec::new();
    for prop in props.iter() {
        let kv = prop.as_prop().unwrap().as_key_value().unwrap();
        ref_names.push(kv.key.as_ident().unwrap().clone());

        let ref_value = CallExpr {
            span: Default::default(),
            type_args: None,
            callee: ref_callee.clone(),
            args: vec![ExprOrSpread {
                spread: None,
                expr: kv.value.clone(),
            }],
        };

        // Push setup statement into statements
        setup_statements.push(Stmt::Decl(Decl::Var(VarDecl {
            kind: swc_ecma_ast::VarDeclKind::Const,
            span: Default::default(),
            declare: Default::default(),
            decls: vec![VarDeclarator {
                definite: false,
                span: Default::default(),
                name: Pat::Ident(BindingIdent {
                    id: kv.key.as_ident().unwrap().clone(),
                    type_ann: None,
                }),
                init: Some(Box::new(Expr::Call(ref_value))),
            }],
        })));
    }

    // Add return
    let return_stmt = Stmt::Return(ReturnStmt {
        span: Default::default(),
        arg: Some(Box::new(Expr::Object(ObjectLit {
            span: Default::default(),
            props: ref_names
                .into_iter()
                .map(|ident| PropOrSpread::Prop(Box::new(Prop::Shorthand(ident))))
                .collect(),
        }))),
    });

    setup_statements.push(return_stmt);
    setup_statements
}

pub struct Visitor;

impl VisitMut for Visitor {
    fn visit_mut_module(&mut self, module: &mut Module) {
        // Only visit children of default export
        let default_export = module.body.iter().find(|item| {
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

        // A default export with an object literal means this
        // is structured correctly, so visit children
        if default_export.is_some() {
            module.visit_mut_children_with(self);
        }
    }

    fn visit_mut_method_prop(&mut self, prop: &mut MethodProp) {
        if let Some(mut ident) = prop.key.as_mut_ident() {
            if ident.sym == Atom::from("data") {
                if let Some(mut block_stmt) = prop.function.body.as_mut() {
                    ident.sym = Atom::from("setup");
                    block_stmt.stmts = data_to_refs(&block_stmt.stmts);
                }
            }
        }
    }
}

pub fn visit_module(module: Module) -> Module {
    module.fold_with(&mut as_folder(Visitor))
}

test!(
    Default::default(),
    |_| as_folder(Visitor),
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
