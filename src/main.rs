use clap::Parser;
use nom::{
    bytes::complete::{take, take_until},
    IResult,
};
use std::{fs, path::PathBuf};

#[macro_use]
extern crate swc_common;
extern crate swc_ecma_parser;

// local
mod ast;
mod parser;

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

#[derive(Debug, Clone)]
struct InvalidScriptError;

fn get_script_contents(path: &PathBuf) -> Result<String, InvalidScriptError> {
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

fn main() {
    let args = Cli::parse();
    println!("Running on: {:?}", args.paths[0]);

    let path = &args.paths[0];
    let script = match get_script_contents(path) {
        Ok(data) => data,
        Err(InvalidScriptError) => {
            eprintln!("Malformed script block in file: {:?}", path);
            std::process::exit(1);
        }
    };
    // println!("{:?}", script);
    let noderes = parser::default_export_object_from_string(script);
    // println!("{:?}", noderes.unwrap());

    let node = noderes.unwrap();
    let vue = ast::create_vue_component(node);
    println!("{:?}", vue.data);

    // Read data between those bounds
    // TODO: Use nom to read each individual part of the defined component
    // inject
    // data
    // created
    // mounted
    // methods
    // etc.
}
