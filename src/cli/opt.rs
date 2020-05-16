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
    #[structopt(parse(from_os_str), name = "FILE", default_value = "example.yml")]
    #[validate(custom = "validate_readable_filename")]
    config: PathBuf,
}

#[derive(Debug)]
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

/// Newtype wrapping a `PathBuf` validated as referring to a readable file.
#[derive(Debug)]
pub struct ReadableFilename(PathBuf);

fn validate_readable_filename(path: &PathBuf) -> Result<(), ValidationError> {
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

//
// Unit tests
//
#[cfg(test)]
mod tests {
    use super::*;

    mod cli_options {
        use super::*;

        #[test]
        fn test_config_path_not_exist_is_validation_error() {
            let unvalidated = UnvalidatedCLIOptions { config: PathBuf::from("DOES_NOT_EXIST") };
            let validated = CLIOptions::new(&unvalidated);
            let errors = validated.err().unwrap();
            let config_errors = *errors.field_errors().get("config").unwrap();
            assert_eq!(config_errors.len(), 1);
            assert!(config_errors[0].code.eq("not_found"));
        }
    }

    mod validate_readable_filename {
        use super::*;

        #[test]
        fn test_opt_path_not_exist_is_err() {
            assert!(validate_readable_filename(&PathBuf::from("DOES_NOT_EXIST")).is_err());
        }

        #[test]
        fn test_opt_path_directory_is_err() {
            assert!(validate_readable_filename(&PathBuf::from("..")).is_err());
        }

        #[test]
        fn test_opt_path_real_file_is_ok() {
            println!("{:?}", validate_readable_filename(&PathBuf::from(file!())));
            assert!(validate_readable_filename(&PathBuf::from(file!())).is_ok());
        }
    }
}
