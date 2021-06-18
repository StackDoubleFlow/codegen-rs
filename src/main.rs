#![warn(rust_2018_idioms)]

mod data;
mod helpers;
mod writer;

use anyhow::{Context, Result};
use data::DllData;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn read_dll_data() -> Result<DllData> {
    Ok(if Path::new("codegen.bc").exists() {
        let input = File::open("codegen.bc").context("Failed to open JSON dump cache")?;
        bincode::deserialize_from(input).context("Failed to parse JSON dump cache")?
    } else {
        let input = File::open("codegen.json").context("Failed to open JSON dump")?;
        println!("Codegen data cache has not been created yet, this may take a whie...");
        let dll_data: DllData =
            serde_json::from_reader(input).context("Failed to parse JSON dump")?;
        let cache_file = File::create("codegen.bc").context("Failed to create JSON dump cache")?;
        bincode::serialize_into(cache_file, &dll_data)
            .context("Failed to serialize JSON dump cache")?;
        dll_data
    })
}

fn main() -> Result<()> {
    println!("Reading codegen data");
    let json: DllData = read_dll_data()?;

    println!("Creating output file");
    let mut output = File::create("generated.rs")?;

    println!("Generating tokens");
    let tokens = json.write_tokens();

    println!("Writing code");
    output.write_fmt(format_args!("{}", tokens))?;

    println!("Done");
    Ok(())
}
