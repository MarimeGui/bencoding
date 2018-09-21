extern crate bencoding;
extern crate clap;

use bencoding::Bencoding;
use clap::{App, Arg};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::process::exit;

fn main() {
    let matches = App::new("Torrent file parser")
        .version("0.1")
        .author("Marime Gui")
        .about("Reads and print a Torrent file into a human readable format")
        .arg(
            Arg::with_name("INPUT")
                .help("Torrent file to parse")
                .required(true)
                .index(1),
        ).get_matches();

    let input_str = matches.value_of("INPUT").unwrap();
    let input_path = Path::new(input_str);
    if !input_path.exists() {
        eprintln!("Error: The specified input file does not exist or is unaccessible.");
        exit(1);
    }
    let data = Bencoding::import(&mut BufReader::new(File::open(input_path).unwrap())).unwrap();

    let structure = print_list(0, data);
    println!("{}", structure);
}

fn print_list(nb_spaces: u32, bencoded: Bencoding) -> String {
    let mut text = String::new();
    match bencoded {
        Bencoding::String(str) => {
            let len = str.len();
            match String::from_utf8(str) {
                Ok(decoded) => text.push_str(&format!("String ({}): '{}'", len, decoded)),
                Err(_) => text.push_str(&format!("String ({}): [redacted]", len)),
            }
        }
        Bencoding::Integer(int) => {
            text.push_str(&format!("Integer {}", int));
        }
        Bencoding::List(vec) => {
            text.push_str(&format!("List ({}):", vec.len()));
            for element in vec {
                text.push_str(&format!(
                    "\n{}{}",
                    &get_spaces(nb_spaces + 2),
                    &print_list(nb_spaces + 2, element)
                ));
            }
        }
        Bencoding::Dictionary(map) => {
            text.push_str(&format!("Dictionary ({}):", map.len()));
            for (key, element) in map {
                text.push_str(&format!(
                    "\n{}Key '{}': {}",
                    &get_spaces(nb_spaces + 2),
                    String::from_utf8(key).unwrap(),
                    &print_list(nb_spaces + 2, element)
                ));
            }
        }
    }
    text
}

fn get_spaces(nb_spaces: u32) -> String {
    let mut spaces = String::new();
    for _ in 0..nb_spaces {
        spaces.push(char::from(' '));
    }
    spaces
}
