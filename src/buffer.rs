use std::io::Seek;
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
};

use crate::content::Content;

pub struct Buffer {
    pub content: Content,
    file: File,
}

impl Buffer {
    pub fn from_path(path: String) -> Buffer {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        return Buffer {
            file,
            content: Content::new(contents.into()),
        };
    }

    pub fn save(&mut self) {
        let content = self.content.to_string();

        self.file.set_len(0).unwrap(); // Truncate the file to 0 bytes
        self.file.seek(std::io::SeekFrom::Start(0)).unwrap(); // Go to the beginning

        self.file.write(content.as_bytes()).unwrap();
    }
}
