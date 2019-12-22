use rkv::{Manager, Rkv, SingleStore, StoreOptions};
use serde::{Deserialize, Serialize};
use serde_json::{Deserializer, Value};
use std::default::Default;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::ops::Deref;
use std::path::PathBuf;
use structopt::StructOpt;
use tempfile::Builder;

#[derive(Debug, StructOpt)]
#[structopt(name = "example-differ")]
/// Diff structured data files using key fields with high performance.
///
/// TODO: Add additional intro paragraph for the --help output here!
struct CliOpt {
    /// Config file
    #[structopt(
        short,
        long,
        parse(from_os_str),
        name = "FILE",
        default_value = "example.yml"
    )]
    config: PathBuf,
}

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
    // Parse config file from command-line option or default
    let cli_opt: CliOpt = CliOpt::from_args();
    let cfg_path = cli_opt.config.as_path();
    if !cfg_path.exists() {
        if cfg_path.as_os_str() == "example.yml" {
            CliOpt::clap().print_help()?;
            println!();
            return Ok(());
        } else {
            return Err(format!("Config file not found: {}", cfg_path.display()).into());
        }
    }
    let cfg_yaml: Result<String, Box<dyn Error>> = fs::read_to_string(cfg_path)
        .map_err(|e| format!("Failed to read config file {}: {}", cfg_path.display(), e).into());
    let cfg_result = serde_yaml::from_slice::<DifferConfig>(cfg_yaml?.as_ref());
    let cfg_error: Result<DifferConfig, Box<dyn Error>> = cfg_result.map_err(|e| {
        format!(
            "Failed to parse {}: {}",
            cfg_path.display(),
            e
        )
        .into()
    });
    let cfg = cfg_error?;

    println!(
        "Config read from \"{}\":\n    {:?}\n\n",
        cfg_path.display(),
        cfg
    );

    let cfg_left = cfg.left.ok_or("Missing left")?;
    let cfg_left_pks = cfg_left.primary_key.ok_or("Missing left primary key")?;

    // Convert configured primary key paths into JMESPath functions
    let jmespath_pks = cfg_left_pks
        .iter()
        .map(|s| {
            if s.starts_with("-") {
                // NOTE: Wrap strings starting with "-" in quotes per JMESPath docs e.g. "-RefId"
                format!("\"{}\"", s)
            } else {
                s.to_owned()
            }
        })
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
    let extract_pk_jmespath = |json: &Value| -> String {
        jmespath_pks
            .iter()
            .map(|pk| pk.search(json).expect("Successful JMESPath search result"))
            .map(|rcv| rcv.as_string().expect("a JSON string").to_owned())
            .collect::<Vec<String>>()
            .join("|")
    };

    // Prefer simple lookup unless we see JMESPath-specific characters
    let use_simple_lookup = cfg_left_pks.iter().all(|s: &String| !s.contains("."));

    let extract_pk: Box<dyn Fn(&Value) -> String> = match use_simple_lookup {
        true => {
            println!("Using simple key lookup");
            Box::new(extract_pk_simple)
        }
        _ => {
            println!("Using JMESPath key lookup");
            Box::new(extract_pk_jmespath)
        }
    };

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
    let root = Builder::new()
        .prefix("experiment-differ")
        .tempdir()
        .unwrap();
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
                    &rkv::Value::Json(json.to_string().as_str()),
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

    let (pk_slice, maybe_json) = iter.next().unwrap().unwrap();
    let pk = std::str::from_utf8(pk_slice)?;
    let json_str = match maybe_json {
        Some(rkv::Value::Json(s)) => s,
        _ => "darn it!",
    };
    println!("{}, {}", pk, json_str);

    let (last_pk_slice, last_maybe_json) = iter.last().unwrap().unwrap();
    let last_pk = std::str::from_utf8(last_pk_slice)?;
    let last_json_str = match last_maybe_json {
        Some(rkv::Value::Json(s)) => s,
        _ => "darn it!",
    };
    println!("{}, {}", last_pk, last_json_str);

    Ok(())
}
