use serde::{Serialize, Deserialize};
use serde_json::{Deserializer, Value};
use std::error::Error;
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::default::Default;

#[derive(Debug, Default, Deserialize, Serialize)]
struct DifferConfig {
    left: Option<PathBuf>,
    right: Option<PathBuf>
}

fn main() -> Result<(), Box<dyn Error>> {
    // Parse config from "example.yml"
    let cfg: DifferConfig = serde_yaml::from_slice(
        fs::read_to_string("example.yml")?.as_ref()
    )?;
    println!("Config read from \"example.yml\":\n    {:?}\n\n", cfg);

    // Parse some jsonl values
    let jsonl_iter = Deserializer::from_reader(
        File::open(cfg.left.ok_or("foo")?)?
    ).into_iter::<Value>();

    for json in jsonl_iter.take(5) {
        println!("{}", json?.to_string())
    }

    Ok(())
}
