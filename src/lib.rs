use swc_common::FilePathMapping;
use swc_common::{sync::Lrc, SourceMap};
use swc_ecma_visit::{as_folder, FoldWith};

#[macro_use]
extern crate swc_common;
extern crate swc_ecma_parser;

// Modules
pub mod codegen;
pub mod parser;
pub mod visitor;

use visitor::Visitor;

// TODO: Use Result/Option
pub fn process(source: String) -> String {
    let cm = Lrc::new(SourceMap::new(FilePathMapping::empty()));
    match parser::parse_script_js(source, &cm) {
        Ok((module, comments)) => {
            // dbg!(&module);
            let mut visitor = as_folder(Visitor {
                comments: comments.clone(),
                ..Default::default()
            });
            let visited = module.fold_with(&mut visitor);
            // dbg!(&visited);

            // TODO: pass comments in here
            codegen::emit_module(&visited, cm, comments)
        }
        Err(_) => "".into(),
    }
}
