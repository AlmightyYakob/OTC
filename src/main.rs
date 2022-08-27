use clap::Parser;
use std::{fs, path::PathBuf};

use swc_common::FilePathMapping;
use swc_common::{sync::Lrc, SourceMap};

#[macro_use]
extern crate swc_common;
extern crate swc_ecma_parser;

// local
// mod vue_ast;
mod codegen;
mod parser;
mod visitor;

/// Search for a pattern in a file and display the lines that contain it.
#[derive(Parser)]
struct Cli {
    /// The paths to convert
    #[clap(name = "PATHS", parse(from_os_str), required = true)]
    paths: Vec<PathBuf>,

    #[clap(
        short,
        long,
        help = "Recurse into directories.",
        default_value_t = false
    )]
    recursive: bool,
}

// TODO: Use Result/Option
pub fn process(source: String) -> String {
    let cm = Lrc::new(SourceMap::new(FilePathMapping::empty()));
    match parser::parse_script_js(source, &cm) {
        Ok(module) => codegen::emit_module(&visitor::visit_module(module), cm),
        Err(_) => "".into(),
    }
}

fn main() {
    let args = Cli::parse();
    println!("Running on: {:?}", args.paths[0]);

    let path = &args.paths[0];
    let script = match parser::parse_vue_script(path) {
        Ok(data) => data,
        Err(parser::InvalidScriptError) => {
            eprintln!("Malformed script block in file: {:?}", path);
            std::process::exit(1);
        }
    };

    let res = process(script);
    fs::write("output.js", &res).unwrap();
}
