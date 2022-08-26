use string_cache::Atom;
use swc_common::errors::{ColorConfig, Handler};
use swc_common::FilePathMapping;
use swc_common::{sync::Lrc, FileName, SourceMap};
use swc_core::testing_transform::test;
use swc_ecma_ast::*;
use swc_ecma_codegen::text_writer::JsWriter;
use swc_ecma_codegen::{Config, Emitter};
use swc_ecma_parser::lexer::Lexer;
use swc_ecma_parser::{Parser, StringInput, Syntax};
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

struct Visitor;

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

#[derive(Debug, Clone)]
pub struct CouldNotParseModule;

pub fn parse(source: String, cm: &Lrc<SourceMap>) -> Result<Module, CouldNotParseModule> {
    // let cm: Lrc<SourceMap> = Default::default();
    let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));
    let fm = cm.new_source_file(FileName::Custom("test.js".into()), source);
    let lexer = Lexer::new(
        // We want to parse ecmascript
        Syntax::Es(Default::default()),
        // EsVersion defaults to es5
        Default::default(),
        StringInput::from(&*fm),
        None,
    );

    let mut parser = Parser::new_from(lexer);
    for e in parser.take_errors() {
        e.into_diagnostic(&handler).emit();
    }

    let parse_res = parser.parse_module().map_err(|e| {
        // Unrecoverable fatal error occurred
        e.into_diagnostic(&handler).emit()
    });

    match parse_res {
        Ok(module) => Ok(module),
        Err(_) => Err(CouldNotParseModule),
    }
}

pub fn visit_module(module: Module) -> Module {
    module.fold_with(&mut as_folder(Visitor))
}

pub fn emit(module: &Module, cm: Lrc<SourceMap>) -> String {
    let mut buf = vec![];
    {
        let writer = Box::new(JsWriter::new(cm.clone(), "\n", &mut buf, None));
        let mut emitter = Emitter {
            cfg: Config::default(),
            comments: None,
            cm: cm.clone(),
            wr: writer,
        };
        emitter.emit_module(&module).unwrap();
    }

    String::from_utf8(buf).unwrap()
}

pub fn process(source: String) -> String {
    let cm = Lrc::new(SourceMap::new(FilePathMapping::empty()));
    match parse(source, &cm) {
        Ok(module) => emit(&visit_module(module), cm),
        Err(_) => "".into(),
    }
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
