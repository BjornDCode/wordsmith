use core::panic;

use gpui::SharedString;

#[derive(Clone)]
pub struct Text {
    content: SharedString,
}

impl Text {
    pub fn new(content: impl Into<SharedString>) -> Text {
        Text {
            content: content.into(),
        }
    }

    pub fn to_display_content(&self) -> String {
        let blocks = self.blocks();
        let mut output = String::new();

        for block in blocks {
            output += &block.to_display_content(&self.content);
        }

        return output;
    }

    pub fn blocks(&self) -> Vec<Block> {
        let mut blocks: Vec<Block> = Vec::new();
        let mut offset = 0;
        let mut line_index = 0;

        let mut lines = self.lines();

        loop {
            let line = lines.next();

            if line.is_none() {
                break;
            }

            let line = line.unwrap();

            if line == "" {
                blocks.push(Block::Newline);
                offset += 1;
                line_index += 1;

                continue;
            }

            if line.starts_with('#') {
                let level = line
                    .chars()
                    .take_while(|&character| character == '#')
                    .count();

                let block = Block::Headline(Headline {
                    start: offset,
                    length: line.len(),
                    level: HeadlineLevel::from(level),
                    original_line_index: line_index,
                });

                blocks.push(block);
                offset += line.len();

                blocks.push(Block::Newline);
                offset += 1;
                line_index += 1;

                continue;
            }

            // Paragraph
            let block = Block::Paragraph(Paragraph {
                start: offset,
                length: line.len(),
                original_line_index: line_index,
            });

            blocks.push(block);
            offset += line.len();

            blocks.push(Block::Newline);
            offset += 1;
            line_index += 1;
        }

        return blocks;
    }
}

impl Text {
    pub fn lines(&self) -> std::str::Lines<'_> {
        self.content.lines()
    }

    pub fn get_line_length(&self, line_index: usize) -> usize {
        let line = self.lines().nth(line_index).unwrap();

        line.len()
    }
}

#[derive(Debug)]
pub enum Block {
    Newline,
    Headline(Headline),
    Paragraph(Paragraph),
}

impl Block {
    pub fn to_display_content(&self, content: &SharedString) -> String {
        match self {
            Block::Newline => "\n".into(),
            Block::Headline(headline) => {
                let level_length = headline.level.length() + 1;
                Block::get_content_slice(
                    content,
                    headline.start + level_length,
                    headline.length - level_length,
                )
            }
            Block::Paragraph(paragraph) => {
                Block::get_content_slice(content, paragraph.start, paragraph.length)
            }
        }
    }

    fn get_content_slice(content: &SharedString, start: usize, length: usize) -> String {
        let slice = &content[start..start + length];

        return String::from(slice);
    }
}

#[derive(Debug)]
pub enum HeadlineLevel {
    H1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

impl HeadlineLevel {
    pub fn from(number: usize) -> HeadlineLevel {
        match number {
            1 => HeadlineLevel::H1,
            2 => HeadlineLevel::H2,
            3 => HeadlineLevel::H3,
            4 => HeadlineLevel::H4,
            5 => HeadlineLevel::H5,
            6 => HeadlineLevel::H6,
            _ => panic!("Invalid headline level: {}", number),
        }
    }

    pub fn length(&self) -> usize {
        match self {
            HeadlineLevel::H1 => 1,
            HeadlineLevel::H2 => 2,
            HeadlineLevel::H3 => 3,
            HeadlineLevel::H4 => 4,
            HeadlineLevel::H5 => 5,
            HeadlineLevel::H6 => 6,
        }
    }
}

#[derive(Debug)]
pub struct Headline {
    pub start: usize,
    pub length: usize,
    pub level: HeadlineLevel,
    pub original_line_index: usize,
}

#[derive(Debug)]
pub struct Paragraph {
    pub start: usize,
    pub length: usize,
    pub original_line_index: usize,
}
