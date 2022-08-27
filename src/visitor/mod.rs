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

pub fn write_setup(stmts: Vec<Stmt>) -> MethodProp {
    return MethodProp {
        key: PropName::Ident(Ident {
            optional: false,
            span: Default::default(),
            sym: Atom::from("setup"),
        }),
        function: Function {
            is_async: false,
            is_generator: false,
            return_type: None,
            type_params: None,
            span: Default::default(),
            params: vec![
                Param {
                    decorators: vec![],
                    span: Default::default(),
                    pat: Pat::Ident(BindingIdent {
                        type_ann: None,
                        id: Ident {
                            span: Default::default(),
                            sym: Atom::from("props"),
                            optional: false,
                        },
                    }),
                },
                Param {
                    decorators: vec![],
                    span: Default::default(),
                    pat: Pat::Ident(BindingIdent {
                        type_ann: None,
                        id: Ident {
                            span: Default::default(),
                            sym: Atom::from("ctx"),
                            optional: false,
                        },
                    }),
                },
            ],
            body: Some(BlockStmt {
                span: Default::default(),
                stmts: stmts,
            }),
            decorators: vec![],
        },
    };
}

pub struct Visitor {
    // All the statements that come before the default export (imports, etc.)
    pre_component: Option<Vec<ModuleItem>>,

    // The components object
    components: Option<Box<Expr>>,

    // The inject array/expr
    inject: Option<Box<Expr>>,

    // The props
    props: Option<Box<Expr>>,

    // The data() method
    data: Option<Function>,

    // The created() method
    created: Option<Box<Function>>,

    // The mounted() method
    mounted: Option<Box<Function>>,

    // The method object
    methods: Option<Box<Expr>>,
}
impl Default for Visitor {
    fn default() -> Visitor {
        Self {
            pre_component: None,
            components: None,
            inject: None,
            props: None,
            data: None,
            created: None,
            mounted: None,
            methods: None,
        }
    }
}

impl VisitMut for Visitor {
    fn visit_mut_module(&mut self, module: &mut Module) {
        // Only visit children of default export

        let default_export = module.body.iter().find_map(|item| {
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

            return expr.expr.as_object();
        });

        // Exit if invalid
        if default_export.is_none() {
            return;
        }

        // TODO: Visit entire AST and fill in the Visitor struct.
        // After above are defined, do the following (in this order)
        // * Convert inject into new syntax
        // * Convert data into refs
        // * Convert created method into valid vector of statements
        // * Convert mounted into onMounted call
        // * Convert methods object into vector of function declarations
        //
        // Finally, write all statements into setup and add return statement

        let object = default_export.unwrap();
        for prop in object.props.iter() {
            if !prop.is_prop() {
                continue;
            }

            match &**prop.as_prop().unwrap() {
                Prop::Method(method_prop) => {
                    // TODO: Match data, created, mounted
                    if let Some(ident) = method_prop.key.as_ident() {
                        match ident.sym.to_string().as_str() {
                            "data" => {
                                self.data = Some(method_prop.function.clone());
                            }
                            "created" => {}
                            "mounted" => {}
                            _ => {}
                        }
                    }
                }
                Prop::KeyValue(kv) => {
                    // TODO: Match components, inject, props, methods
                }
                _ => {}
            }
        }
    }
}

pub fn visit_module(module: Module) -> Module {
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
