use core::panic;

use gpui::SharedString;

// use crate::display_map::DisplayMap;

#[derive(Clone)]
pub struct Text {
    content: SharedString,
    // display_map: DisplayMap,
}

impl Text {
    pub fn new(content: impl Into<SharedString>) -> Text {
        Text {
            content: content.into(),
            // display_map: DisplayMap::new(),
        }
    }

    pub fn to_string(&self) -> String {
        self.content.to_string()
    }

    pub fn blocks(&self) -> Vec<Block> {
        let mut blocks: Vec<Block> = Vec::new();
        let mut offset = 0;

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
                });

                blocks.push(block);
                offset += line.len();

                blocks.push(Block::Newline);
                offset += 1;

                continue;
            }

            // Paragraph
            let block = Block::Paragraph(Paragraph {
                start: offset,
                length: line.len(),
            });

            blocks.push(block);
            offset += line.len();

            blocks.push(Block::Newline);
            offset += 1;
        }

        return blocks;
    }
}

// Rendering
impl Text {
    // pub fn get_spans(&mut self) -> Vec<TextSpan> {
    //     let mut spans: Vec<TextSpan> = vec![];
    //     let mut offset = 0;

    //     for (index, line) in self.lines().enumerate() {
    //         if !line.is_empty() {
    //             if line.starts_with('#') {
    //                 let removed_count = &self.display_map.get_removed_count();
    //                 let level = line
    //                     .chars()
    //                     .take_while(|&character| character == '#')
    //                     .count();

    //                 self.display_map.push_hidden_range(offset, level + 1);
    //                 self.display_map.push_headline(index, level);

    //                 spans.push(TextSpan {
    //                     start: offset - removed_count,
    //                     length: line.len() - level - 1,
    //                     kind: TextSpanType::Headline,
    //                 });
    //             }

    //             offset += line.len();
    //         }

    //         offset += 1; // Newline character
    //     }

    //     return spans;
    // }

    // pub fn get_display_content(&self) -> String {
    //     let mut modified = &self.content.to_string();

    //     let mut count = 0;

    //     for range in &self.display_map.hidden {
    //         modified.drain(range.start - count..range.start + range.length - count);
    //         count += range.length;
    //     }

    //     return modified;
    // }
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

// #[derive(Debug, Clone, Copy)]
// struct TextSpan {
//     start: usize,
//     length: usize,
//     kind: TextSpanType,
// }

// #[derive(Debug, Clone, Copy)]
// enum TextSpanType {
//     Normal,
//     Headline,
// }

#[derive(Debug)]
pub enum Block {
    Newline,
    Headline(Headline),
    Paragraph(Paragraph),
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
}

#[derive(Debug)]
pub struct Headline {
    pub start: usize,
    pub length: usize,
    pub level: HeadlineLevel,
}

#[derive(Debug)]
pub struct Paragraph {
    pub start: usize,
    pub length: usize,
}
