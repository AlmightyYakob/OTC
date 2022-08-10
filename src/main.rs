use clap::Parser;
use std::{
    fs::File,
    io::{self, BufRead, BufReader, Lines, Seek},
    path::PathBuf,
};

// local
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

// Finds the lines at which the script block exists
fn find_script_bounds(lines: Lines<BufReader<File>>) -> Result<(usize, usize), InvalidScriptError> {
    let mut start: Option<usize> = None;
    let mut end: Option<usize> = None;

    for (index, line) in lines.enumerate() {
        if let Ok(data) = line {
            if data.contains("<script>") {
                if start != None {
                    return Err(InvalidScriptError);
                }

                start = Some(index);
            }
            if data.contains("</script>") {
                if end != None {
                    return Err(InvalidScriptError);
                }

                end = Some(index);
            }
        }
    }

    // Return err if bounds not found
    if start == None || end == None {
        return Err(InvalidScriptError);
    }

    Ok((start.unwrap(), end.unwrap()))
}

fn main() {
    let args = Cli::parse();
    println!("{:?}", args.paths);

    let path = &args.paths[0];
    // let data = fs::read_to_string(path).expect("Unable to read file");

    let file = File::open(path).unwrap();
    let lines = io::BufReader::new(file).lines();
    let maybe_bounds = find_script_bounds(lines);

    // Check and unwrap bounds
    let bounds = match maybe_bounds {
        Ok(bounds) => bounds,
        Err(_) => {
            eprintln!("Malformed script block in file: {:?}", path);
            std::process::exit(1);
        }
    };

    println!("{:?}", bounds);

    // Read data between those bounds
    // let mut input = File::open(path).unwrap();
    // input.seek(pos)
}
