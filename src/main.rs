use clap::Parser;
use std::{fs, path::PathBuf};

#[macro_use]
extern crate swc_common;
extern crate swc_ecma_parser;

// local
// mod vue_ast;
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

fn main() {
    let args = Cli::parse();
    println!("Running on: {:?}", args.paths[0]);

    let path = &args.paths[0];
    let script = match parser::get_script_contents(path) {
        Ok(data) => data,
        Err(parser::InvalidScriptError) => {
            eprintln!("Malformed script block in file: {:?}", path);
            std::process::exit(1);
        }
    };

    let res = visitor::process(script);
    fs::write("output.js", &res).unwrap();
}
