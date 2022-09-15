use swc_common::FilePathMapping;
use swc_common::{sync::Lrc, SourceMap};

#[macro_use]
extern crate swc_common;
extern crate swc_ecma_parser;

// Modules
pub mod codegen;
pub mod parser;
pub mod visitor;

// TODO: Use Result/Option
pub fn process(source: String) -> String {
    let cm = Lrc::new(SourceMap::new(FilePathMapping::empty()));
    match parser::parse_script_js(source, &cm) {
        Ok(module) => codegen::emit_module(&visitor::visit_module(module), cm),
        Err(_) => "".into(),
    }
}
