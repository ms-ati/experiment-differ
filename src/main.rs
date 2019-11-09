use serde::{Serialize, Deserialize};
use serde_json::{Deserializer, Value};
use std::default::Default;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::rc::Rc;

#[derive(Debug, Default, Deserialize, Serialize)]
struct InputConfig {
    path: PathBuf,
    primary_key: Option<Vec<String>>
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct DifferConfig {
    left: Option<InputConfig>,
    right: Option<InputConfig>
}

fn main() -> Result<(), Box<dyn Error>> {
    // Parse config from "example.yml"
    let cfg: DifferConfig =
        serde_yaml::from_slice(fs::read_to_string("example.yml")?.as_ref())?;
    println!("Config read from \"example.yml\":\n    {:?}\n\n", cfg);

    // Convert configured primary key paths into JMESPath functions
    let cfg_left = cfg.left.ok_or("Missing left")?;
    let cfg_left_pks = cfg_left.primary_key.ok_or("Missing left primary key")?;
    let jmespath_pks = cfg_left_pks.
        iter().
        map(|s| format!("\"{}\"", s)).                  // quoted to allow -RefId
        map(|s| jmespath::compile(s.as_str()).unwrap()).
        collect::<Vec<jmespath::Expression>>();

    // Parse primary keys from some jsonl values
    let jsonl_iter =
        Deserializer::from_reader(File::open(cfg_left.path)?).
        into_iter::<Value>().
        map(Result::unwrap);  // FIX: panics on failed parse

    for json in jsonl_iter.take(5000) {
        let pk_vals = jmespath_pks.
            iter().
            map(|pk| pk.search(jmespath::Variable::from(&json)).unwrap()).
            collect::<Vec<Rc<jmespath::Variable>>>();
        println!("{:?}", pk_vals)
    }

    Ok(())
}
