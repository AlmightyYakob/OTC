use clap::Parser;
use std::{fs, path::PathBuf};

// Import Lib
use otc::*;

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
    for path in &args.paths {
        if path.is_dir() {
            println!(
                "Skipping directory {}. Use the -r flag to run on folders.",
                path.clone().into_os_string().into_string().unwrap()
            );
            continue;
        }

        let script = parser::parse_vue_script(path);
        if script.is_err() {
            continue;
        }

        let res = process(script.unwrap());
        fs::write("output.js", &res).unwrap();
    }
}
