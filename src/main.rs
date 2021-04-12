mod data;

use data::DllData;
use std::fs::File;

fn main() {
    let f = File::open("parsed.json").expect("Unable to JSON dump");
    let json: DllData = serde_json::from_reader(f).unwrap();
    dbg!(&json.types[0].this.name);
}
