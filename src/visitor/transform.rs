use std::collections::HashMap;

use string_cache::Atom;
use swc_ecma_ast::*;

use super::{vue::WatchDecl, Visitor};

impl Visitor {
    pub fn transform_component(&mut self) {
        // Pass through components
        if let Some(components) = &self.options.components {
            self.composition.components = Some(components.clone())
        }

        // Pass through props
        if let Some(props) = &self.options.props {
            self.composition.props = Some(props.clone())
        }

        // Transform inject statements
        if let Some(injects) = &self.inject_set {
            self.composition.inject_stmts = Some(transform_inject(injects));
        }

        // Transform data to refs
        if let Some(func) = &self.options.data {
            self.composition.ref_stmts = Some(transform_data(&func.body.as_ref().unwrap().stmts));
        }

        // Transform created statements
        if let Some(created) = &self.options.created {
            if let Some(block_stmt) = &created.body {
                self.composition.created_stmts = Some(block_stmt.stmts.clone());
            }
        }

        // Transform computed
        if let Some(computed_decls) = &self.options.computed {
            self.composition.computed = Some(transform_computed(computed_decls));
        }

        // Transform watch
        if let Some(watch_decls) = &self.options.watch {
            self.composition.watch = Some(transform_watch(watch_decls));
        }

        // Transform methods
        if let Some(methods) = &self.options.methods {
            self.composition.method_decls = Some(
                methods
                    .iter()
                    .map(|fn_decl| Stmt::Decl(Decl::Fn(fn_decl.clone())))
                    .collect(),
            );
        }

        // Transform mounted
        if let Some(mounted) = &self.options.mounted {
            self.composition.mounted = Some(transform_mounted(mounted));
        }
    }
}

pub fn transform_computed(fn_decls: &Vec<FnDecl>) -> Vec<Stmt> {
    let computed_callee = Callee::Expr(Box::new(Expr::Ident(Ident {
        optional: false,
        span: Default::default(),
        sym: Atom::from("computed"),
    })));

    // Map and return
    fn_decls
        .iter()
        .filter_map(|decl| {
            if decl.function.body.is_none() {
                return None;
            }

            // Optimize return statement if possible
            let body = decl.function.body.as_ref().unwrap();
            let mut arrow_expr_body = BlockStmtOrExpr::BlockStmt(body.clone());
            {
                let stmts = &arrow_expr_body.as_block_stmt().unwrap().stmts;
                if stmts.len() == 1 {
                    if let Stmt::Return(r_stmt) = &stmts[0] {
                        arrow_expr_body = BlockStmtOrExpr::Expr(r_stmt.arg.clone().unwrap())
                    }
                }
            }

            Some(Stmt::Decl(Decl::Var(VarDecl {
                span: Default::default(),
                declare: false,
                kind: VarDeclKind::Const,
                decls: vec![VarDeclarator {
                    span: Default::default(),
                    definite: false,
                    name: Pat::Ident(BindingIdent {
                        type_ann: None,
                        id: decl.ident.clone(),
                    }),
                    init: Some(Box::new(Expr::Call(CallExpr {
                        span: Default::default(),
                        callee: computed_callee.clone(),
                        type_args: None,
                        args: vec![ExprOrSpread {
                            spread: None,
                            expr: Box::new(Expr::Arrow(ArrowExpr {
                                span: Default::default(),
                                is_async: false,
                                is_generator: false,
                                type_params: None,
                                return_type: None,
                                params: vec![],
                                body: arrow_expr_body,
                            })),
                        }],
                    }))),
                }],
            })))
        })
        .collect()
}

pub fn transform_watch(watch_decls: &Vec<WatchDecl>) -> Vec<Stmt> {
    let computed_callee = Callee::Expr(Box::new(Expr::Ident(Ident {
        optional: false,
        span: Default::default(),
        sym: Atom::from("watch"),
    })));

    // Map and return
    watch_decls
        .iter()
        .filter_map(|decl| {
            if decl.function.body.is_none() {
                return None;
            }

            // Optimize return statement if possible
            let body = decl.function.body.as_ref().unwrap();
            let mut arrow_expr_body = BlockStmtOrExpr::BlockStmt(body.clone());
            {
                let stmts = &arrow_expr_body.as_block_stmt().unwrap().stmts;
                if stmts.len() == 1 {
                    if let Stmt::Return(r_stmt) = &stmts[0] {
                        arrow_expr_body = BlockStmtOrExpr::Expr(r_stmt.arg.clone().unwrap())
                    }
                }
            }

            // Create params/args
            let params = decl.function.params.iter().map(|p| p.pat.clone()).collect();
            let mut args = vec![
                ExprOrSpread {
                    spread: None,
                    expr: Box::new(Expr::Ident(decl.ident.clone())),
                },
                ExprOrSpread {
                    spread: None,
                    expr: Box::new(Expr::Arrow(ArrowExpr {
                        span: Default::default(),
                        is_async: false,
                        is_generator: false,
                        type_params: None,
                        return_type: None,
                        params,
                        body: arrow_expr_body,
                    })),
                },
            ];

            // Inject deep/immediatge if needed
            if decl.deep.is_some() || decl.immediate.is_some() {
                let mut props = vec![];
                if let Some(deep) = &decl.deep {
                    props.push(PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                        key: PropName::Ident(Ident {
                            optional: false,
                            span: Default::default(),
                            sym: Atom::from("deep"),
                        }),
                        value: deep.clone(),
                    }))));
                }
                if let Some(immediate) = &decl.immediate {
                    props.push(PropOrSpread::Prop(Box::new(Prop::KeyValue(KeyValueProp {
                        key: PropName::Ident(Ident {
                            optional: false,
                            span: Default::default(),
                            sym: Atom::from("immediate"),
                        }),
                        value: immediate.clone(),
                    }))));
                }

                // Push argument into vector
                args.push(ExprOrSpread {
                    spread: None,
                    expr: Box::new(Expr::Object(ObjectLit {
                        span: Default::default(),
                        props,
                    })),
                });
            }

            Some(Stmt::Expr(ExprStmt {
                span: Default::default(),
                expr: Box::new(Expr::Call(CallExpr {
                    span: Default::default(),
                    callee: computed_callee.clone(),
                    type_args: None,
                    args,
                })),
            }))
        })
        .collect()
}

pub fn transform_mounted(mounted: &Function) -> Vec<Stmt> {
    if mounted.body.is_none() {
        return vec![];
    }

    // Return vec with single onMounted wrawpper
    let body = mounted.body.as_ref().unwrap();
    vec![Stmt::Expr(ExprStmt {
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
    })]
}

pub fn transform_inject(injects: &HashMap<String, Str>) -> Vec<Stmt> {
    let inject_callee = Callee::Expr(Box::new(Expr::Ident(Ident {
        optional: false,
        span: Default::default(),
        sym: Atom::from("inject"),
    })));

    return injects
        .iter()
        .map(|(_, s)| {
            Stmt::Decl(Decl::Var(VarDecl {
                span: Default::default(),
                declare: false,
                kind: VarDeclKind::Const,
                decls: vec![VarDeclarator {
                    definite: false,
                    span: Default::default(),
                    name: Pat::Ident(BindingIdent {
                        type_ann: None,
                        id: Ident {
                            span: Default::default(),
                            sym: s.value.clone(),
                            optional: false,
                        },
                    }),
                    init: Some(Box::new(Expr::Call(CallExpr {
                        span: Default::default(),
                        type_args: None,
                        callee: inject_callee.clone(),
                        args: vec![ExprOrSpread {
                            spread: None,
                            expr: Box::new(Expr::Lit(Lit::Str(Str {
                                span: Default::default(),
                                raw: s.raw.clone(),
                                value: s.value.clone(),
                            }))),
                        }],
                    }))),
                }],
            }))
        })
        .collect();

    // TODO: Handle cases besides array literal
}

pub fn transform_data(stmts: &Vec<Stmt>) -> Vec<Stmt> {
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
        // TODO: Handle shorthands
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
