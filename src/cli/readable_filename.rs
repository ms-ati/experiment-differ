use mockall::*;
use once_cell::sync::Lazy;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io;
use std::path::PathBuf;
use std::sync::Mutex;

/// Newtype wrapping a `PathBuf` validated as referring to a readable file.
#[derive(Debug)]
pub struct ReadableFilename(PathBuf);

impl ReadableFilename {
    /// Validate that an `OsStr` represents a readable file.
    pub fn try_from_os_str(arg: &OsStr) -> Result<ReadableFilename, OsString> {
        let path = PathBuf::from(arg);

        #[cfg(debug_assertions)]
        let guard = GLOBAL_PATH_INFO_MOCKABLE.lock().unwrap();
        let path_info_mockable= if cfg!(debug_assertions) { &**guard } else { &PathInfoDefault };

        if !path_info_mockable.path_exists_mockable(&path) {
            Err(format!("File {:?} not found", arg).into())
        } else if !path_info_mockable.path_is_file_mockable(&path) {
            Err(format!("Path {:?} is not a file", arg).into())
        } else if path_info_mockable.file_open_mockable(&path).is_err() {
            Err(format!("File {:?} could not be opened", arg).into())
        } else {
            Ok(ReadableFilename(path))
        }
    }
}

/// Mockable wrapper for calling path info functions, for testing
#[automock]
pub trait PathInfoMockable {
    fn path_exists_mockable(&self, path: &PathBuf) -> bool {
        path.exists()
    }

    fn path_is_file_mockable(&self, path: &PathBuf) -> bool {
        path.is_file()
    }

    fn file_open_mockable(&self, path: &PathBuf) -> io::Result<File> {
        File::open(&path)
    }
}

/// Default implementation of the mockable trait
struct PathInfoDefault;
impl PathInfoMockable for PathInfoDefault {}

// Concern: Mocking
#[cfg(debug_assertions)]
pub static GLOBAL_PATH_INFO_MOCKABLE: Lazy<Mutex<Box<dyn PathInfoMockable + Send>>> =
    Lazy::new(|| Mutex::new(Box::new(PathInfoDefault)));
