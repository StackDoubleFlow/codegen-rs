#![warn(rust_2018_idioms)]

mod data;
mod helpers;
mod writer;

use data::DllData;
use std::{fs::File, io::Write};

fn main() {
    let input = File::open("parsed.json").expect("Unable to read JSON dump");
    let json: DllData = serde_json::from_reader(input).unwrap();
    // println!("{:#?}", &json);
    let mut output = File::create("generated.rs").unwrap();
    output
        .write_fmt(format_args!("{}", json.write_tokens()))
        .unwrap();
}
