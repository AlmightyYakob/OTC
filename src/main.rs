use clap::Parser;
use std::{fs, path::PathBuf};
use vue_sfc::{Block, BlockName, Section};

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
    println!("{:?}", args.paths);

    let path = &args.paths[0];
    let data = fs::read_to_string(path).expect("Unable to read file");

    let sfc = vue_sfc::parse(&data).unwrap();
    for section in sfc {
        match section {
            Section::Block(Block {
                name,
                attributes,
                content,
            }) => {
                println!(
                    "Got a block named `{}` with {} attributes, content is {} bytes long.",
                    name,
                    attributes.len(),
                    content.len()
                );

                if name.as_str() == "script" {
                    println!("{:?}", attributes);
                }
            }
            _ => {
                panic!("This shouldn't happen")
            }
        }
    }
}
