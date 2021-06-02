extern crate serde;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Page {
    opening_text: Option<String>,
    text: Option<String>,
}
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: wparse <dump_path> <out_path>");
        std::process::exit(1);
    }
    let dump_path = &args[1];
    let dump_file = match std::fs::File::open(&dump_path) {
        Err(error) => {
            eprintln!("Failed to open dump file: {}", error);
            std::process::exit(1);
        }
        Ok(file) => std::io::BufReader::new(file),
    };
    let out_path = &args[2];
    let out_file = match std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(out_path)
    {
        Err(error) => {
            eprintln!("Failed to open output file: {}", error);
            std::process::exit(1);
        }
        Ok(file) => std::io::BufWriter::new(file),
    };
    if dump_path.ends_with(".gz") {
        parse_dump(
            std::io::BufReader::new(flate2::bufread::GzDecoder::new(dump_file)),
            out_file,
        );
    } else {
        parse_dump(dump_file, out_file);
    }
}

fn parse_dump(source: impl std::io::BufRead, mut target: impl std::io::Write) {
    for line in source.lines() {
        let line = line.expect("Couldn't get line");
        match serde_json::from_str(&line) {
            Ok(Page { opening_text, text }) => {
                if let Some(opening_text_content) = opening_text {
                    if opening_text_content.len() > 0 {
                        write!(target, "{}\n", opening_text_content).unwrap();
                    }
                }
                if let Some(text_content) = text {
                    if text_content.len() > 0 {
                        write!(target, "{}\n", text_content).unwrap();
                    }
                }
            }
            Err(e) => {
                eprintln!("skipping line: {}", e);
            }
        }
    }
}
