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

pub fn emit_module(module: &Module, cm: Lrc<SourceMap>) -> String {
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
