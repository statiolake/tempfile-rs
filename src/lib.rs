extern crate uuid;

use uuid::{Uuid, UuidVersion};

use std::env;
use std::fs;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};

/// A struct representing temporary file.
/// This struct creates new temporary file when an instance is created, and remove the temporary
/// file when the instance is to be dropped.
pub struct TempFile {
    file: Option<File>,
    file_path: PathBuf,
}

impl TempFile {
    /// Creates a new instance containing `File` object of the specified path. If the file exists,
    /// then it returns Err.
    pub fn from_path(file_path: impl Into<PathBuf>) -> io::Result<TempFile> {
        let file_path = file_path.into();
        File::create(&file_path).map(|file| TempFile {
            file: Some(file),
            file_path: file_path,
        })
    }

    /// Creates a new instance containing `File` object, whose name is randomly generated (by
    /// UUID v4)
    pub fn in_dir(dir_path: PathBuf) -> io::Result<TempFile> {
        let file_path = dir_path.join(create_tmp_file_name("temp"));
        TempFile::from_path(file_path)
    }

    /// Creates a new instance containing `File` object, whose path is randomly generated in
    /// `env::temp_dir()`.
    pub fn new() -> io::Result<TempFile> {
        TempFile::in_dir(env::temp_dir())
    }

    /// Close temporary file.
    pub fn close(&mut self) {
        self.file.take();
    }

    /// Close the temporary file if necessary, and re-open it for reading.
    pub fn reopen(&mut self) -> io::Result<()> {
        self.file = Some(File::open(&self.file_path)?);
        Ok(())
    }

    /// Get current `File` object.
    pub fn file(&self) -> Option<&File> {
        self.file.as_ref()
    }

    /// Get current `File` object (mutable).
    pub fn file_mut(&mut self) -> Option<&mut File> {
        self.file.as_mut()
    }

    /// Get current file path.
    pub fn file_path(&self) -> &Path {
        &self.file_path
    }
}

/// Close the temporary file and remove that file.
impl Drop for TempFile {
    fn drop(&mut self) {
        // take the value (drop the file) to close this file.
        self.file.take();
        let _ = fs::remove_file(&self.file_path);
    }
}

fn create_tmp_file_name(prefix: impl Into<String>) -> String {
    let uuid =
        Uuid::new(UuidVersion::Random).expect("internal error: could not instantiate UUID object");
    prefix.into() + &uuid.simple().to_string()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        use TempFile;
        let path;

        {
            let mut temp_file = TempFile::new().expect("failed to instantiate TempFile.");
            temp_file.reopen().expect("reopen failed.");
            path = temp_file.file_path().to_path_buf();
        }

        use std::fs::File;
        assert!(File::open(&path).is_err(), "file still remains!");
    }
}
