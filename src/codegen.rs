use swc_common::comments::SingleThreadedComments;
use swc_common::{sync::Lrc, SourceMap};
use swc_ecma_ast::Module;
use swc_ecma_codegen::text_writer::JsWriter;
use swc_ecma_codegen::{Config, Emitter};

pub fn emit_module(
    module: &Module,
    cm: Lrc<SourceMap>,
    comments: SingleThreadedComments,
) -> String {
    let mut buf = vec![];
    {
        let writer = Box::new(JsWriter::new(cm.clone(), "\n", &mut buf, None));
        let mut emitter = Emitter {
            cfg: Config::default(),
            comments: Some(&comments),
            cm: cm.clone(),
            wr: writer,
        };
        emitter.emit_module(&module).unwrap();

        // NOTE: This is how to write a newline
        // emitter.wr.write_comment("\n").expect("asd");
    }

    String::from_utf8(buf).unwrap()
}
