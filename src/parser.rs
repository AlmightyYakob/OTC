use std::fs;
use std::path::PathBuf;

use nom::{
    bytes::complete::{take, take_until},
    IResult,
};
use swc_common::errors::{ColorConfig, Handler};
use swc_common::{sync::Lrc, FileName, SourceMap};
use swc_ecma_ast::Module;
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax};

// TODO: Use vue-sfc
#[derive(Debug, Clone)]
pub struct InvalidScriptError;
pub fn parse_vue_script(path: &PathBuf) -> Result<String, InvalidScriptError> {
    let pathstr = path.to_str().unwrap();
    let data =
        fs::read_to_string(path).expect(format!("Unable to read file: {}", pathstr).as_str());

    let prescriptres: IResult<&str, &str> = take_until("<script>")(data.as_str());
    if prescriptres.is_err() {
        return Err(InvalidScriptError);
    }

    // Take <script>
    let prescript: IResult<&str, &str> = take(8usize)(prescriptres.unwrap().0);
    let remaining = prescript.unwrap().0;

    // Grab remaining data
    let scriptres: IResult<&str, &str> = take_until("</script>")(remaining);
    if scriptres.is_err() {
        return Err(InvalidScriptError);
    }

    return Ok(scriptres.unwrap().1.to_string());
}

#[derive(Debug, Clone)]
pub struct CouldNotParseModule;
pub fn parse_script_js(source: String, cm: &Lrc<SourceMap>) -> Result<Module, CouldNotParseModule> {
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
