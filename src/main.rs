use rkv::{Manager, Rkv, SingleStore, StoreOptions};
use serde::{Deserialize, Serialize};
use serde_json::{Deserializer, Value};
use std::default::Default;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::ops::Deref;
use std::path::PathBuf;
use tempfile::Builder;

#[derive(Debug, Default, Deserialize, Serialize)]
struct InputConfig {
    path: PathBuf,
    primary_key: Option<Vec<String>>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct DifferConfig {
    left: Option<InputConfig>,
    right: Option<InputConfig>,
}

fn main() -> Result<(), Box<dyn Error>> {
    // Parse config from "example.yml"
    let cfg: DifferConfig = serde_yaml::from_slice(fs::read_to_string("example.yml")?.as_ref())?;
    println!("Config read from \"example.yml\":\n    {:?}\n\n", cfg);

    // Convert configured primary key paths into JMESPath functions
    // NOTE: NOT CURRENTLY USED - Could handle more complex json paths
    let cfg_left = cfg.left.ok_or("Missing left")?;
    let cfg_left_pks = cfg_left.primary_key.ok_or("Missing left primary key")?;
    let jmespath_pks = cfg_left_pks
        .iter()
        .map(|s| format!("\"{}\"", s)) // quoted to allow -RefId
        .map(|s| jmespath::compile(s.as_str()).unwrap())
        .collect::<Vec<jmespath::Expression>>();

    // Lazily parse jsonl values from memory mapped file (fastest way)
    let file = File::open(cfg_left.path)?;
    let mmap = unsafe { memmap::Mmap::map(&file) }?;
    let jsonl_iter = Deserializer::from_slice(mmap.deref())
        .into_iter::<Value>()
        .map(Result::unwrap); // FIX: panics on failed parse

    // Lazily extract their joined primary key - simple key lookup, not JMESPath
    let extract_pk_simple = |json: &Value| -> String {
        cfg_left_pks
            .iter()
            .map(|pk| json[pk].as_str().unwrap())
            .collect::<Vec<&str>>()
            .join("|")
    };

    // Lazily extract their joined primary key - more complex lookup w/ JMESPath
    // NOTE: NOT CURRENTLY USED
    let _extract_pk_jmespath = |json: &Value| -> String {
        jmespath_pks
            .iter()
            .map(|pk| pk.search(jmespath::Variable::from(json)).unwrap())
            .map(|rcv| rcv.as_string().unwrap().to_owned())
            .collect::<Vec<String>>()
            .join("|")
    };

    // NOTE: Using only the simple primary key lookup now
    let extract_pk = extract_pk_simple;

    let jsonl_pks_iter = jsonl_iter.map(|json: Value| (extract_pk(&json), json));

    let write_to_stdout = false;
    let write_to_lmdb = true;

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
    println!("TempDir: {}", root.path().display());

    // The Manager enforces that each process opens the same environment
    // at most once by caching a handle to each environment that it opens.
    // Use it to retrieve the handle to an opened environment—or create one
    // if it hasn't already been opened:
    let created_arc = Manager::singleton()
        .write()
        .unwrap()
        .get_or_create(path, Rkv::new)
        .unwrap();
    let env = created_arc.read().unwrap();

    // NOTE: Needs to be large enough for full db, but actual map is sparse on
    // disk and allocated incrementally up to this max size.
    let lmdb_max_map_size: usize = 1024 * 1024 * 1024;
    env.set_map_size(lmdb_max_map_size).unwrap();

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
            store
                .put(
                    &mut writer,
                    pk,
                    &rkv::Value::Blob(json.to_string().as_bytes()),
                )
                .unwrap();
        }
    }

    // You must commit a write transaction before the writer goes out
    // of scope, or the transaction will abort and the data won't persist.
    writer.commit().unwrap();

    // Read back first item
    let reader = env.read().unwrap();
    let mut iter = store.iter_start(&reader).unwrap();

    let (pk_slice, maybe_blob) = iter.next().unwrap().unwrap();
    let pk = std::str::from_utf8(pk_slice)?;
    let blob_bytes = maybe_blob.unwrap().to_bytes().unwrap();
    let json_str = std::str::from_utf8(blob_bytes.as_slice())?;
    println!("{}, {}", pk, json_str);

    Ok(())
}
