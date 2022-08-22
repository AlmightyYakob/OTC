use swc_common::sync::Lrc;
use swc_common::{
    errors::{ColorConfig, Handler},
    FileName, SourceMap,
};
use swc_ecma_ast::ObjectLit;
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};

#[derive(Debug, Clone)]
pub struct NoDefaultExportFound;

pub fn default_export_object_from_string(
    script_data: String,
) -> Result<ObjectLit, NoDefaultExportFound> {
    let cm: Lrc<SourceMap> = Default::default();
    let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));
    let fm = cm.new_source_file(FileName::Custom("test.js".into()), script_data);
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
    let module = parser
        .parse_module()
        .map_err(|e| {
            // Unrecoverable fatal error occurred
            e.into_diagnostic(&handler).emit()
        })
        .expect("failed to parser module");

    for item in module.body.into_iter() {
        if !item.is_module_decl() {
            continue;
        }

        let decl = item.as_module_decl().unwrap();
        if !decl.is_export_default_expr() {
            continue;
        }

        let expr = decl.as_export_default_expr().unwrap();
        if !expr.expr.is_object() {
            continue;
        }
        return Ok(expr.expr.as_object().unwrap().clone());
    }

    return Err(NoDefaultExportFound);
}
