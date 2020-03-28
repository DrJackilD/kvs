use crate::kv::{Result, Storage};
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::io::{BufReader, ErrorKind, SeekFrom};

/// This is implementation of log-based file-system storage.
/// Each storage represent single file in the filesystem,
/// containing commands, located each on the new line
pub struct FileStorage {
    file: File,
    reader: BufReader<File>,
}

impl Storage for FileStorage {
    fn new(db_name: String) -> Result<Self> {
        let f = match OpenOptions::new().append(true).open(&db_name) {
            Ok(f) => f,
            Err(err) => {
                if err.kind() == ErrorKind::NotFound {
                    File::create(&db_name)?
                } else {
                    return Err(err.into());
                }
            }
        };
        Ok(Self {
            file: f,
            reader: BufReader::new(File::open(&db_name)?),
        })
    }

    fn write(&mut self, value: String) -> Result<()> {
        self.file.write_all(value.as_bytes())?;
        Ok(())
    }
}

impl Iterator for FileStorage {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buff = String::new();
        match self.reader.read_line(&mut buff) {
            Ok(size) => {
                if size == 0 {
                    // Since in every get request to storage we should read entire file,
                    // we need to return cursor to the start, to enable reader re-usage
                    // in case of few get requests from one KvStore instance
                    if self.reader.seek(SeekFrom::Start(0)).is_err() {};
                    None
                } else {
                    Some(buff.trim_end().to_owned())
                }
            }
            Err(_) => None,
        }
    }
}
