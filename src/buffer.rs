use std::io::Seek;
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
};

use crate::content::Content;

pub struct Buffer {
    pub content: Content,
    file: Option<File>,
}

impl Buffer {
    pub fn empty() -> Buffer {
        return Buffer {
            content: Content::empty(),
            file: None,
        };
    }

    pub fn from_path(path: String) -> Buffer {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        return Buffer {
            file: Some(file),
            content: Content::new(contents.into()),
        };
    }

    pub fn save(&mut self) {
        let content = self.content.to_string();

        match &mut self.file {
            Some(file) => {
                file.set_len(0).unwrap(); // Truncate the file to 0 bytes
                file.seek(std::io::SeekFrom::Start(0)).unwrap(); // Go to the beginning

                file.write(content.as_bytes()).unwrap();
            }
            None => todo!(),
        };
    }
}
