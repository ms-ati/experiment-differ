use rkv::{Manager, Rkv, SingleStore, StoreOptions};
use serde::{Serialize, Deserialize};
use serde_json::{Deserializer, Value};
use std::default::Default;
use std::error::Error;
use std::io::BufReader;
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use tempfile::Builder;

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

    // Lazily parse jsonl values
    let jsonl_iter =
        Deserializer::from_reader(BufReader::new(File::open(cfg_left.path)?)).
        into_iter::<Value>().
        map(Result::unwrap);  // FIX: panics on failed parse

    // TODO: fast path for single key lookups, slow path for general JMESPath
    let is_simple_lookup = true;

    // Lazily extract their joined primary key
    let extract_pk = |json: &Value| -> String {
        if is_simple_lookup {
            cfg_left_pks.
                iter().
                map(|pk| json[pk].as_str().unwrap()).
                collect::<Vec<&str>>().
                join("|")
        }
        else {
            jmespath_pks.
                iter().
                map(|pk| pk.search(jmespath::Variable::from(json)).unwrap()).
                map(|rcv| rcv.as_string().unwrap().to_owned()).
                collect::<Vec<String>>().
                join("|")
        }
    };

    let jsonl_pks_iter =
        jsonl_iter.map(|json: Value| (extract_pk(&json), json));

    let write_to_stdout = true;
    let write_to_lmdb = false;

    // First determine the path to the environment, which is represented
    // on disk as a directory containing two files:
    //
    //   * a data file containing the key/value stores
    //   * a lock file containing metadata about current transactions
    //
    // In this example, we use the `tempfile` crate to create the directory.
    //
    let root = Builder::new().prefix("differ").tempdir().unwrap();
    fs::create_dir_all(root.path()).unwrap();
    let path = root.path();

    // The Manager enforces that each process opens the same environment
    // at most once by caching a handle to each environment that it opens.
    // Use it to retrieve the handle to an opened environment—or create one
    // if it hasn't already been opened:
    let created_arc = Manager::singleton().write().unwrap().get_or_create(path, Rkv::new).unwrap();
    let env = created_arc.read().unwrap();

    // Then you can use the environment handle to get a handle to a datastore:
    let store: SingleStore = env.open_single("mydb", StoreOptions::create()).unwrap();

    // Use a write transaction to mutate the store via a `Writer`.
    // There can be only one writer for a given environment, so opening
    // a second one will block until the first completes.
    let mut writer = env.write().unwrap();

    // Keys are `AsRef<[u8]>`, while values are `Value` enum instances.
    // Use the `Blob` variant to store arbitrary collections of bytes.
    // Putting data returns a `Result<(), StoreError>`, where StoreError
    // is an enum identifying the reason for a failure.
    for (pk, json) in jsonl_pks_iter {
        if write_to_stdout {
            println!("{}", pk);
            println!("{}", json);
        }

        if write_to_lmdb {
            store.put(&mut writer, pk, &rkv::Value::Blob(json.to_string().as_bytes())).unwrap();
        }
    }

    // You must commit a write transaction before the writer goes out
    // of scope, or the transaction will abort and the data won't persist.
    writer.commit().unwrap();

    Ok(())
}
