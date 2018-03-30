

use std::io;
use std::path::{PathBuf};
use std::fs::File;

pub struct FileHandler {
    basePath: PathBuf
}

impl FileHandler {
    pub fn new<P: Into<PathBuf>>(path: P) -> io::Result<Self> {
            Ok(FileHandler {
                basePath: path.into().canonicalize()?
            })
    }

    pub fn handle(&self, path_from_req: & str) ->io::Result<File> {
        println!("start handle");
        let mut path = path_from_req;
        while path.starts_with("/") {
            path = &path[1..];
        }

        let path = PathBuf::new().join(path).canonicalize()?;
        if !path.starts_with(&self.basePath) {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied, "wrong path!"))
        }
        File::open(path) 
    }
}