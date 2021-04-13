#![warn(rust_2018_idioms)]

mod data;
mod helpers;
mod writer;

use data::DllData;
use std::{fs::File, io::Write};

fn main() {
    let input = File::open("parsed.json").expect("Unable to read JSON dump");
    println!("Reading JSON dump");
    let json: DllData = serde_json::from_reader(input).unwrap();
    println!("Creating output file");
    let mut output = File::create("generated.rs").unwrap();
    println!("Generating tokens");
    let tokens = json.write_tokens();
    println!("Writing code");
    output.write_fmt(format_args!("{}", tokens)).unwrap();
    println!("Done");
}
