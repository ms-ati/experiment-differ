use std::fs::File;
use std::path::PathBuf;
use structopt::StructOpt;
use validator::{Validate, ValidationError, ValidationErrors};
use validator_derive::*;

//
// NOTE: The following doc comments on the `Opt` structure are used by the `StructOpt` library's
// macros to define the `--help` output of the CLI (command-line interface) binary.
//
/// Diff structured data files using key fields with high performance.
///
/// TODO: Add additional intro paragraph for the --help output here!
///
#[derive(Debug, StructOpt, Validate)]
#[structopt(name = "experiment-differ")]
pub struct UnvalidatedCLIOptions {
    /// Config file
    #[structopt(
        short,
        long,
        parse(from_os_str),
        name = "FILE",
        default_value = "example.yml"
    )]
    #[validate(custom = "validate_readable_filename")]
    config: PathBuf,
}

pub struct CLIOptions {
    config: ReadableFilename,
}

impl CLIOptions {
    pub fn new(unvalidated: &UnvalidatedCLIOptions) -> Result<CLIOptions, ValidationErrors> {
        unvalidated.validate().map(|_| CLIOptions {
            config: ReadableFilename(unvalidated.config.to_owned()),
        })
    }
}

/// Newtype with validation that a given `PathBuf` is actually the filename of a readable file.
#[derive(Debug)]
pub struct ReadableFilename(PathBuf);

fn validate_readable_filename(path: &PathBuf) -> Result<(), ValidationError> {
    println!("{:?}", path);
    if !path.exists() {
        Err(ValidationError::new("not_found"))
    } else if !path.is_file() {
        Err(ValidationError::new("not_file"))
    } else if File::open(path).is_err() {
        Err(ValidationError::new("cannot_open"))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_not_exist_is_err() {
        assert!(validate_readable_filename(&PathBuf::from("DOES_NOT_EXIST")).is_err());
    }

    #[test]
    fn test_path_directory_is_err() {
        assert!(validate_readable_filename(&PathBuf::from("..")).is_err());
    }

    #[test]
    fn test_path_real_file_is_ok() {
        println!("{:?}", validate_readable_filename(&PathBuf::from("opt.rs")));
        assert!(validate_readable_filename(&PathBuf::from("opt.rs")).is_err());
    }
}
