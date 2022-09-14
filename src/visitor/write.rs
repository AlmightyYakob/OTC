use string_cache::Atom;
use swc_ecma_ast::*;

use super::vue::CompositionComponent;

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
                stmts,
            }),
            decorators: vec![],
        },
    };
}

pub fn write_composition_component(obj: &CompositionComponent) -> ExportDefaultExpr {
    let mut export_props: Vec<PropOrSpread> = vec![];

    // TODO: Component and props probably don't need to be stored on composition component
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

    // Inject Computed
    if let Some(fn_decls) = &obj.computed {
        setup_stmts.extend(fn_decls.clone());
    }

    // Inject watch
    if let Some(watch_decls) = &obj.watch {
        setup_stmts.extend(watch_decls.clone());
    }

    // Inject created
    if let Some(created) = &obj.created_stmts {
        setup_stmts.extend(created.clone());
    }

    // Inject methods
    if let Some(methods) = &obj.method_decls {
        setup_stmts.extend(methods.clone());
    }

    // Inject mounted
    if let Some(mounted) = &obj.mounted {
        setup_stmts.extend(mounted.clone());
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
