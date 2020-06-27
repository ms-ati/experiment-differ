use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::path::PathBuf;

/// Newtype wrapping a `PathBuf` validated as referring to a readable file.
#[derive(Debug)]
pub struct ReadableFilename(PathBuf);

impl ReadableFilename {
    /// Try to validate that an `OsStr` represents a readable file.
    pub fn try_from_os_str(arg: &OsStr) -> Result<ReadableFilename, OsString> {
        let path: PathBuf = arg.into();

        if !path.exists() {
            Err(format!("File {:?} not found", arg).into())
        } else if !path.is_file() {
            Err(format!("Path {:?} is not a file", arg).into())
        } else if File::open(&path).is_err() {
            Err(format!("File {:?} could not be opened", arg).into())
        } else {
            Ok(ReadableFilename(path))
        }
    }
}
