use std::io::Seek;
use std::ops::Range;
use std::path::PathBuf;
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
};

use crate::content::{Content, Line};
use crate::editor::EditorPosition;

pub struct Buffer {
    content: Content,
    file: Option<File>,
    is_saved: bool,
}

impl Buffer {
    pub fn empty() -> Buffer {
        return Buffer {
            content: Content::empty(),
            file: None,
            is_saved: true,
        };
    }

    pub fn from_path(path: PathBuf) -> Buffer {
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
            is_saved: true,
        };
    }

    pub fn save(&mut self) -> Result<(), SaveError> {
        let content = self.content.to_string();

        match &mut self.file {
            Some(file) => {
                file.set_len(0).unwrap(); // Truncate the file to 0 bytes
                file.seek(std::io::SeekFrom::Start(0)).unwrap(); // Go to the beginning

                file.write(content.as_bytes()).unwrap();

                self.is_saved = true;
                Ok(())
            }
            None => Err(SaveError::NoFileAssociated),
        }
    }

    pub fn content(&self) -> Content {
        return self.content.clone();
    }

    pub fn is_empty(&self) -> bool {
        return self.content().text().to_string().is_empty();
    }

    pub fn pristine(&self) -> bool {
        return self.is_saved;
    }

    pub fn has_file(&self) -> bool {
        return self.file.is_some();
    }

    pub fn set_file(&mut self, path: PathBuf) -> Result<(), SaveError> {
        // Open or create the file
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;

        self.file = Some(file);

        // Save the content to the new file
        self.save()?;

        Ok(())
    }
}

impl Buffer {
    pub fn lines(&self) -> Vec<Line> {
        return self.content.lines();
    }

    pub fn line(&self, index: usize) -> Line {
        return self.content.line(index);
    }

    pub fn position_to_offset(&self, position: EditorPosition) -> usize {
        return self.content.position_to_offset(position);
    }

    pub fn offset_to_position(&self, offset: usize) -> EditorPosition {
        return self.content.offset_to_position(offset);
    }

    pub fn read_range(&self, range: Range<usize>) -> String {
        return self.content.read_range(range);
    }

    pub fn replace(&mut self, range: Range<usize>, replacement: String) {
        self.is_saved = false;

        return self.content.replace(range, replacement);
    }
}

#[derive(Debug)]
pub enum SaveError {
    NoFileAssociated,
    IoError(std::io::Error),
}

impl From<std::io::Error> for SaveError {
    fn from(error: std::io::Error) -> Self {
        SaveError::IoError(error)
    }
}
