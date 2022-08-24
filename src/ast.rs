use string_cache::Atom;
use swc_common::{EqIgnoreSpan, Span};
use swc_ecma_ast::{
    BindingIdent, BlockStmt, CallExpr, Callee, Decl, Expr, ExprOrSpread, Function, Ident,
    KeyValueProp, MethodProp, ObjectLit, Param, Pat, Prop, PropName, Stmt, VarDecl, VarDeclarator,
};

// WIP: AST to contain vue component
#[derive(Debug)]
pub struct VueOptionsComponent {
    pub components: Option<KeyValueProp>,
    pub props: Option<KeyValueProp>,
    pub data: Option<MethodProp>,
    pub created: Option<MethodProp>,
    pub mounted: Option<MethodProp>,
    pub methods: Option<KeyValueProp>,
}
impl Default for VueOptionsComponent {
    fn default() -> VueOptionsComponent {
        Self {
            components: None,
            props: None,
            data: None,
            created: None,
            mounted: None,
            methods: None,
        }
    }
}

// WIP: AST to contain vue component
#[derive(Debug)]
pub struct VueCompositionComponent {
    pub props: Option<KeyValueProp>,
    pub setup: Option<MethodProp>,
}
impl Default for VueCompositionComponent {
    fn default() -> VueCompositionComponent {
        Self {
            props: None,
            setup: None,
        }
    }
}

pub fn create_vue_component(object: ObjectLit) -> VueOptionsComponent {
    let mut vue = VueOptionsComponent::default();
    for prop in object
        .props
        .into_iter()
        .map(|x| x.prop())
        .filter(|x| x.is_some())
        .map(|x| x.unwrap())
    {
        let key = match *prop {
            Prop::KeyValue(ref x) => Some(&x.key),
            Prop::Method(ref x) => Some(&x.key),
            _ => None,
        };
        if key.is_none() {
            continue;
        }

        // Extract name
        let propname = key.unwrap();
        let keystr = match propname {
            PropName::Ident(ref id) => Some(&id.sym),
            PropName::Str(ref s) => Some(&s.value),
            _ => None,
        };
        if keystr.is_none() {
            continue;
        }

        // Assign
        match keystr.unwrap().to_string().as_str() {
            "components" => vue.components = Some(prop.key_value().unwrap()),
            "props" => vue.props = Some(prop.key_value().unwrap()),
            "data" => vue.data = Some(prop.method().unwrap()),
            "created" => vue.created = Some(prop.method().unwrap()),
            "mounted" => vue.mounted = Some(prop.method().unwrap()),
            "methods" => vue.methods = Some(prop.key_value().unwrap()),
            _ => {}
        }
    }

    return vue;
}

// WIP: This code wraps the provided statements in the setup method prop
pub fn generate_setup_shell(stmts: Vec<Stmt>) -> MethodProp {
    return MethodProp {
        key: PropName::Ident(Ident {
            optional: false,
            span: Span::default(),
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

pub fn data_to_refs(data: &MethodProp) -> Vec<Stmt> {
    let return_expr = data
        .function
        .body
        .as_ref()
        .unwrap()
        .stmts
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
    let mut setup_statements: Vec<Stmt> = Vec::new();
    for prop in props.iter() {
        let kv = prop.as_prop().unwrap().as_key_value().unwrap();

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
