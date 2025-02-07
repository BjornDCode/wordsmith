use std::{ops::Index, sync::Arc};

use gpui::{
    rgb, Font, FontWeight, Hsla, Pixels, SharedString, TextRun, WindowTextSystem, WrappedLine,
};

use crate::{text::WrappedText, COLOR_GRAY_700, COLOR_GRAY_800};

#[derive(Debug)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

impl Point {
    pub fn new(x: usize, y: usize) -> Point {
        Point { x, y }
    }
}

#[derive(Debug, Clone)]
pub struct Content {
    text: SharedString,
}

impl Content {
    pub fn new(text: SharedString) -> Content {
        Content { text }
    }

    pub fn blocks(&self) -> Vec<Block> {
        let mut blocks: Vec<Block> = Vec::new();
        let mut line_index = 0;

        let mut lines = self.text.lines();

        loop {
            let line = lines.next();

            if line.is_none() {
                break;
            }

            let line = line.unwrap();

            if line == "" {
                blocks.push(Block::Newline(Newline { line_index }));

                line_index += 1;

                continue;
            }

            if line.starts_with('#') {
                let level = line
                    .chars()
                    .take_while(|&character| character == '#')
                    .count();

                let headline = Headline {
                    content: WrappedText::new(line.into()),
                    level: HeadlineLevel::from(level),
                    line_index,
                };
                line_index += headline.line_length();

                let block = Block::Headline(headline);
                blocks.push(block);

                continue;
            }

            // Paragraph
            let paragraph = Paragraph {
                content: WrappedText::new(line.into()),
                line_index,
            };

            line_index += paragraph.line_length();

            let block = Block::Paragraph(paragraph);

            blocks.push(block);
        }

        return blocks;
    }

    pub fn cursor_position(&self, block_index: usize, offset: usize) -> Point {
        let block = self.block(block_index);

        return block.cursor_position(offset);
    }

    pub fn block_length(&self, block_index: usize) -> usize {
        let block = self.block(block_index);

        return block.length();
    }

    pub fn block(&self, block_index: usize) -> Block {
        self.blocks().index(block_index).clone()
    }
}

#[derive(Debug, Clone)]
pub enum Block {
    Newline(Newline),
    Paragraph(Paragraph),
    Headline(Headline),
}

impl Block {
    pub fn line_index(&self) -> usize {
        match self {
            Block::Newline(newline) => newline.line_index,
            Block::Paragraph(paragraph) => paragraph.line_index,
            Block::Headline(headline) => headline.line_index,
        }
    }

    pub fn cursor_position(&self, offset: usize) -> Point {
        let line_index = self.line_of_offset(offset);
        let line_start = self.line_start(line_index);
        let remaining = offset - line_start;

        return Point::new(remaining, self.line_index() + line_index);
    }

    pub fn line_of_offset(&self, offset: usize) -> usize {
        let (lines, wrap_points) = self.lines_and_wrap_points();
        let lines = lines.into_iter().enumerate();
        let mut line_index: usize = 0;
        let mut processed_offset = 0;

        for (index, line) in lines {
            processed_offset += line.len();

            // We need separate logic when it's a soft-wrapped line, compared to when it's not
            if wrap_points.contains(&processed_offset) {
                if processed_offset > offset {
                    line_index = index;

                    break;
                }
            } else {
                if processed_offset >= offset {
                    line_index = index;

                    break;
                }
            }
        }

        return line_index;
    }

    pub fn line_start(&self, line_index: usize) -> usize {
        let lines = self.lines().into_iter().enumerate();
        let mut offset = 0;

        for (index, line) in lines {
            if index == line_index {
                break;
            } else {
                offset += line.len();
            }
        }

        return offset;
    }

    pub fn offset_in_line(&self, line_index: usize, offset: usize) -> usize {
        let line_start = self.line_start(line_index);

        return offset - line_start;
    }
}

#[derive(Debug, Clone)]
pub struct Newline {
    line_index: usize,
}

#[derive(Debug, Clone)]
pub struct Paragraph {
    pub content: WrappedText,
    line_index: usize,
}

#[derive(Debug, Clone)]
pub struct Headline {
    pub content: WrappedText,
    level: HeadlineLevel,
    line_index: usize,
}

impl Headline {
    pub fn level(&self) -> usize {
        self.level.to_usize()
    }
}

#[derive(Debug, Clone)]
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

    pub fn to_usize(&self) -> usize {
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

#[derive(Debug, Clone)]
pub struct RenderedBlock {
    pub lines: Vec<WrappedLine>,
    pub line_length: usize,
    pub line_index: usize,
}

pub trait Render {
    fn render(
        &mut self,
        text_system: &Arc<WindowTextSystem>,
        font: Font,
        font_size: Pixels,
    ) -> RenderedBlock;
}

impl Render for Block {
    fn render(
        &mut self,
        text_system: &Arc<WindowTextSystem>,
        font: Font,
        font_size: Pixels,
    ) -> RenderedBlock {
        match self {
            Block::Newline(newline) => newline.render(text_system, font, font_size),
            Block::Headline(headline) => headline.render(text_system, font, font_size),
            Block::Paragraph(paragraph) => paragraph.render(text_system, font, font_size),
        }
    }
}

impl Render for Newline {
    fn render(
        &mut self,
        text_system: &Arc<WindowTextSystem>,
        _font: Font,
        font_size: Pixels,
    ) -> RenderedBlock {
        let runs: Vec<TextRun> = vec![];

        let lines = text_system
            .shape_text("".into(), font_size, &runs, None)
            .unwrap()
            .into_vec();

        return RenderedBlock {
            lines,
            line_length: self.line_length(),
            line_index: self.line_index,
        };
    }
}

impl Render for Paragraph {
    fn render(
        &mut self,
        text_system: &Arc<WindowTextSystem>,
        font: Font,
        font_size: Pixels,
    ) -> RenderedBlock {
        let content = self.content.to_string();

        let runs: Vec<TextRun> = vec![TextRun {
            len: content.len(),
            font: font.clone(),
            color: Hsla::from(rgb(COLOR_GRAY_700)),
            background_color: None,
            underline: None,
            strikethrough: None,
        }];

        let lines = text_system
            .shape_text(content.into(), font_size, &runs, None)
            .unwrap()
            .into_vec();

        return RenderedBlock {
            lines,
            line_length: self.line_length(),
            line_index: self.line_index,
        };
    }
}

impl Render for Headline {
    fn render(
        &mut self,
        text_system: &Arc<WindowTextSystem>,
        font: Font,
        font_size: Pixels,
    ) -> RenderedBlock {
        let content = self.content.to_string();
        let runs: Vec<TextRun> = vec![TextRun {
            len: content.len(),
            font: Font {
                weight: FontWeight::EXTRA_BOLD,
                ..font.clone()
            },
            color: Hsla::from(rgb(COLOR_GRAY_800)),
            background_color: None,
            underline: None,
            strikethrough: None,
        }];

        let lines = text_system
            .shape_text(content.into(), font_size, &runs, None)
            .unwrap()
            .into_vec();

        return RenderedBlock {
            lines,
            line_length: self.line_length(),
            line_index: self.line_index,
        };
    }
}

pub trait Size {
    fn line_length(&self) -> usize;

    fn length(&self) -> usize;

    fn length_of_line(&self, line_index: usize) -> usize;

    fn lines(&self) -> Vec<String>;

    fn lines_and_wrap_points(&self) -> (Vec<String>, Vec<usize>);

    fn is_soft_wrapped_line(&self, line_index: usize) -> bool;
}

impl Size for Block {
    fn line_length(&self) -> usize {
        match self {
            Block::Newline(newline) => newline.line_length(),
            Block::Paragraph(paragraph) => paragraph.line_length(),
            Block::Headline(headline) => headline.line_length(),
        }
    }

    fn length(&self) -> usize {
        match self {
            Block::Newline(newline) => newline.length(),
            Block::Paragraph(paragraph) => paragraph.length(),
            Block::Headline(headline) => headline.length(),
        }
    }

    fn length_of_line(&self, line_index: usize) -> usize {
        match self {
            Block::Newline(newline) => newline.length_of_line(line_index),
            Block::Paragraph(paragraph) => paragraph.length_of_line(line_index),
            Block::Headline(headline) => headline.length_of_line(line_index),
        }
    }

    fn lines(&self) -> Vec<String> {
        match self {
            Block::Newline(newline) => newline.lines(),
            Block::Paragraph(paragraph) => paragraph.lines(),
            Block::Headline(headline) => headline.lines(),
        }
    }

    fn lines_and_wrap_points(&self) -> (Vec<String>, Vec<usize>) {
        match self {
            Block::Newline(newline) => newline.lines_and_wrap_points(),
            Block::Paragraph(paragraph) => paragraph.lines_and_wrap_points(),
            Block::Headline(headline) => headline.lines_and_wrap_points(),
        }
    }

    fn is_soft_wrapped_line(&self, line_index: usize) -> bool {
        match self {
            Block::Newline(newline) => newline.is_soft_wrapped_line(line_index),
            Block::Paragraph(paragraph) => paragraph.is_soft_wrapped_line(line_index),
            Block::Headline(headline) => headline.is_soft_wrapped_line(line_index),
        }
    }
}

impl Size for Newline {
    fn line_length(&self) -> usize {
        1
    }

    fn lines(&self) -> Vec<String> {
        "".lines().map(String::from).collect()
    }

    fn length(&self) -> usize {
        return 0;
    }

    fn lines_and_wrap_points(&self) -> (Vec<String>, Vec<usize>) {
        let lines = self.lines();

        return (lines, vec![]);
    }

    fn length_of_line(&self, _line_index: usize) -> usize {
        return 0;
    }

    fn is_soft_wrapped_line(&self, line_index: usize) -> bool {
        false
    }
}

impl Size for Headline {
    fn line_length(&self) -> usize {
        self.content.line_length()
    }

    fn lines(&self) -> Vec<String> {
        self.content.lines()
    }

    fn length(&self) -> usize {
        self.content.length()
    }

    fn lines_and_wrap_points(&self) -> (Vec<String>, Vec<usize>) {
        self.content.lines_and_wrap_points()
    }

    fn length_of_line(&self, line_index: usize) -> usize {
        let line = self.content.line(line_index);

        return line.len();
    }

    fn is_soft_wrapped_line(&self, line_index: usize) -> bool {
        self.content.is_soft_wrapped_line(line_index)
    }
}

impl Size for Paragraph {
    fn line_length(&self) -> usize {
        self.content.line_length()
    }

    fn lines(&self) -> Vec<String> {
        self.content.lines()
    }

    fn length(&self) -> usize {
        self.content.length()
    }

    fn lines_and_wrap_points(&self) -> (Vec<String>, Vec<usize>) {
        self.content.lines_and_wrap_points()
    }

    fn length_of_line(&self, line_index: usize) -> usize {
        let line = self.content.line(line_index);

        return line.len();
    }

    fn is_soft_wrapped_line(&self, line_index: usize) -> bool {
        self.content.is_soft_wrapped_line(line_index)
    }
}
