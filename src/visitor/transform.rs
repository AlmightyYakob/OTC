use string_cache::Atom;
use swc_ecma_ast::*;

use super::vue::CompositionComponent;

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

    setup_statements
}

pub fn write_setup(mut stmts: Vec<Stmt>) -> MethodProp {
    // Declarations that will need to be included in the return statement
    let mut declarations: Vec<Ident> = vec![];
    for stmt in stmts.iter() {
        if !stmt.is_decl() {
            continue;
        }

        match stmt.as_decl().unwrap() {
            Decl::Class(cls) => declarations.push(cls.ident.clone()),
            Decl::Fn(func) => declarations.push(func.ident.clone()),
            Decl::Var(var) => declarations.extend(
                var.decls
                    .iter()
                    .filter_map(|decl| decl.name.as_ident())
                    .map(|ident| ident.id.clone()),
            ),
            _ => {}
        }
    }

    stmts.push(Stmt::Return(ReturnStmt {
        span: Default::default(),
        arg: Some(Box::new(Expr::Object(ObjectLit {
            span: Default::default(),
            props: declarations
                .into_iter()
                .map(|ident| PropOrSpread::Prop(Box::new(Prop::Shorthand(ident))))
                .collect(),
        }))),
    }));

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

pub fn write_composition_component(obj: &CompositionComponent) -> ExportDefaultExpr {
    let mut export_props: Vec<PropOrSpread> = vec![];

    // Inject Components
    if let Some(components) = &obj.components {
        export_props.push(PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
            key: PropName::Ident(Ident {
                optional: false,
                span: Default::default(),
                sym: Atom::from("components"),
            }),
            value: components.clone(),
        }))));
    }

    // Inject Props
    if let Some(props) = &obj.props {
        export_props.push(PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
            key: PropName::Ident(Ident {
                optional: false,
                span: Default::default(),
                sym: Atom::from("props"),
            }),
            value: props.clone(),
        }))));
    }

    let mut setup_stmts: Vec<Stmt> = vec![];

    // Inject inject
    if let Some(inject) = &obj.inject_stmts {
        setup_stmts.extend(inject.clone());
    }

    // Inject Refs
    if let Some(refs) = &obj.ref_stmts {
        setup_stmts.extend(refs.clone());
    }

    // Inject created
    if let Some(created) = &obj.created_stmts {
        setup_stmts.extend(created.clone());
    }

    // Inject methods
    if let Some(methods) = &obj.method_decls {
        setup_stmts.extend(
            methods
                .iter()
                .map(|fn_decl| Stmt::Decl(Decl::Fn(fn_decl.clone()))),
        )
    }

    // Inject mounted
    if let Some(mounted) = &obj.mounted {
        if let Some(body) = &mounted.body {
            setup_stmts.push(Stmt::Expr(ExprStmt {
                span: Default::default(),
                expr: Box::new(Expr::Call(CallExpr {
                    span: Default::default(),
                    type_args: None,
                    callee: Callee::Expr(Box::new(Expr::Ident(Ident {
                        optional: false,
                        span: Default::default(),
                        sym: Atom::from("onMounted"),
                    }))),
                    args: vec![ExprOrSpread {
                        spread: None,
                        expr: Box::new(Expr::Arrow(ArrowExpr {
                            span: Default::default(),
                            is_async: mounted.is_async,
                            is_generator: mounted.is_generator,
                            type_params: None,
                            return_type: None,
                            params: vec![],
                            body: BlockStmtOrExpr::BlockStmt(body.clone()),
                        })),
                    }],
                })),
            }));
        }
    }

    // Finally, write setup
    export_props.push(PropOrSpread::Prop(Box::new(Prop::Method(write_setup(
        setup_stmts,
    )))));

    // Return entire defineComponent export
    ExportDefaultExpr {
        span: Default::default(),
        expr: Box::new(Expr::Call(CallExpr {
            span: Default::default(),
            type_args: None,
            callee: Callee::Expr(Box::new(Expr::Ident(Ident {
                optional: false,
                span: Default::default(),
                sym: Atom::from("defineComponent"),
            }))),
            args: vec![ExprOrSpread {
                spread: None,
                expr: Box::new(Expr::Object(ObjectLit {
                    span: Default::default(),
                    props: export_props,
                })),
            }],
        })),
    }
}
