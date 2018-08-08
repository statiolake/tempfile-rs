extern crate uuid;

use uuid::{Uuid, UuidVersion};

use std::env;
use std::fs;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};

/// A struct representing temporary file path.
/// This struct has Drop trait and Drop::drop() removes the file it points to.
/// So, as long as this struct is passed around the temporary file keeps alive.
struct TempFileCore {
    file_path: PathBuf,
}

/// Drop implementation to remove the temporary file.
impl Drop for TempFileCore {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.file_path);
    }
}

/// A struct representing temporary file.
/// This struct creates new temporary file when an instance is created, and remove the temporary
/// file when the instance is to be dropped.
pub struct TempFile {
    // file is Option<_> here, because file needs to be closed before it is removed.
    file: Option<File>,
    core: TempFileCore,
}

/// A struct representing CLOSED temporary file.
/// this struct is created by `close()` method in TempFile.
pub struct ClosedTempFile {
    core: TempFileCore,
}

impl TempFile {
    /// Creates a new instance containing `File` object of the specified path. If the file exists,
    /// then it returns Err.
    pub fn from_path(file_path: impl Into<PathBuf>) -> io::Result<TempFile> {
        let file_path = file_path.into();
        File::create(&file_path).map(|file| TempFile {
            file: Some(file),
            core: TempFileCore { file_path },
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
    pub fn close(self) -> ClosedTempFile {
        ClosedTempFile {
            // TempFile implements Drop, so we must clone() here. Sad.
            core: self.core,
        }
    }

    /// Get current `File` object.
    pub fn file(&self) -> &File {
        self.file
            .as_ref()
            .expect("internal error: file must be valid.")
    }

    /// Get current `File` object (mutable).
    pub fn file_mut(&mut self) -> &mut File {
        self.file
            .as_mut()
            .expect("internal error: file must be valid.")
    }

    /// Get current file path.
    pub fn file_path(&self) -> &Path {
        &self.core.file_path
    }
}

impl ClosedTempFile {
    pub fn reopen(self) -> io::Result<TempFile> {
        File::open(&self.core.file_path).map(|file| TempFile {
            file: Some(file),
            core: self.core,
        })
    }

    pub fn file_path(&self) -> &Path {
        &self.core.file_path
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
            let temp_file = TempFile::new().expect("failed to instantiate TempFile.");
            let closed = temp_file.close();
            let temp_file = closed.reopen().expect("reopen failed.");
            path = temp_file.file_path().to_path_buf();
        }

        use std::fs::File;
        assert!(File::open(&path).is_err(), "file still remains!");
    }
}
