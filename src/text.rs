use core::panic;
use std::sync::Arc;

use gpui::{
    px, rgb, Font, FontWeight, Hsla, Pixels, SharedString, TextRun, WindowTextSystem, WrappedLine,
};

use crate::{COLOR_GRAY_700, COLOR_GRAY_800};

#[derive(Clone, Debug)]
pub struct Text {
    content: SharedString,
}

impl Text {
    pub fn new(content: impl Into<SharedString>) -> Text {
        Text {
            content: content.into(),
        }
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
                blocks.push(Block::Newline(Newline { rendered: None }));
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
                    content: line.into(),
                    start: offset,
                    length: line.len(),
                    level: HeadlineLevel::from(level),
                    original_line_index: line_index,
                    rendered: None,
                });

                blocks.push(block);
                offset += line.len();

                continue;
            }

            // Paragraph
            let block = Block::Paragraph(Paragraph {
                content: line.into(),
                start: offset,
                length: line.len(),
                original_line_index: line_index,
                rendered: None,
            });

            blocks.push(block);
            offset += line.len();
        }

        return blocks;
    }
}

impl Text {
    pub fn lines(&self) -> std::str::Lines<'_> {
        self.content.lines()
    }

    pub fn get_block_length(&self, block_index: usize) -> usize {
        let blocks = self.blocks();
        let block = blocks.get(block_index);

        if let Some(block) = block {
            return block.length();
        } else {
            return 0;
        }
    }
}

#[derive(Debug)]
pub enum Block {
    Newline(Newline),
    Headline(Headline),
    Paragraph(Paragraph),
}

impl Render for Block {
    fn render(
        &mut self,
        text_system: &Arc<WindowTextSystem>,
        font: Font,
        font_size: Pixels,
    ) -> Vec<WrappedLine> {
        match self {
            Block::Newline(newline) => newline.render(text_system, font, font_size),
            Block::Headline(headline) => headline.render(text_system, font, font_size),
            Block::Paragraph(paragraph) => paragraph.render(text_system, font, font_size),
        }
    }
}

impl Block {
    pub fn length(&self) -> usize {
        match self {
            Block::Newline(_) => 0,
            Block::Headline(headline) => headline.length - headline.level.length() - 1,
            Block::Paragraph(paragraph) => paragraph.length,
        }
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
pub struct Newline {
    rendered: Option<Vec<WrappedLine>>,
}

#[derive(Debug)]
pub struct Headline {
    pub content: String,
    pub start: usize,
    pub length: usize,
    pub level: HeadlineLevel,
    pub original_line_index: usize,
    rendered: Option<Vec<WrappedLine>>,
}

impl Headline {
    pub fn line_index() {}
}

#[derive(Debug)]
pub struct Paragraph {
    pub content: String,
    pub start: usize,
    pub length: usize,
    pub original_line_index: usize,
    rendered: Option<Vec<WrappedLine>>,
}

pub trait Render {
    fn render(
        &mut self,
        text_system: &Arc<WindowTextSystem>,
        font: Font,
        font_size: Pixels,
    ) -> Vec<WrappedLine>;
}

impl Render for Paragraph {
    fn render(
        &mut self,
        text_system: &Arc<WindowTextSystem>,
        font: Font,
        font_size: Pixels,
    ) -> Vec<WrappedLine> {
        if let Some(lines) = self.rendered.as_ref() {
            return lines.clone();
        }

        let content = self.content.clone();

        let runs: Vec<TextRun> = vec![TextRun {
            len: content.len(),
            font: font.clone(),
            color: Hsla::from(rgb(COLOR_GRAY_700)),
            background_color: None,
            underline: None,
            strikethrough: None,
        }];
        let slice: &[TextRun] = &runs;

        let lines = text_system
            .shape_text(content.into(), font_size, slice, Some(px(480.)))
            .unwrap()
            .into_vec();

        self.rendered = Some(lines.clone());

        return lines;
    }
}

impl Render for Headline {
    fn render(
        &mut self,
        text_system: &Arc<WindowTextSystem>,
        font: Font,
        font_size: Pixels,
    ) -> Vec<WrappedLine> {
        if let Some(lines) = self.rendered.as_ref() {
            return lines.clone();
        }

        let content = self.content.clone();
        let trimmed_content = &content[self.level.length() + 1..].to_string();
        let runs: Vec<TextRun> = vec![TextRun {
            len: trimmed_content.len(),
            font: Font {
                weight: FontWeight::EXTRA_BOLD,
                ..font.clone()
            },
            color: Hsla::from(rgb(COLOR_GRAY_800)),
            background_color: None,
            underline: None,
            strikethrough: None,
        }];
        let slice: &[TextRun] = &runs;

        let lines = text_system
            .shape_text(trimmed_content.into(), font_size, slice, Some(px(480.)))
            .unwrap()
            .into_vec();

        self.rendered = Some(lines.clone());

        return lines;
    }
}

impl Render for Newline {
    fn render(
        &mut self,
        text_system: &Arc<WindowTextSystem>,
        font: Font,
        font_size: Pixels,
    ) -> Vec<WrappedLine> {
        if let Some(lines) = self.rendered.as_ref() {
            return lines.clone();
        }

        let runs: Vec<TextRun> = vec![];
        let slice: &[TextRun] = &runs;

        let lines = text_system
            .shape_text("".into(), font_size, slice, Some(px(480.)))
            .unwrap()
            .into_vec();

        self.rendered = Some(lines.clone());

        return lines;
    }
}

pub trait Size {
    fn line_length(
        &mut self,
        text_system: &Arc<WindowTextSystem>,
        font: Font,
        font_size: Pixels,
    ) -> usize;
}

impl Size for Block {
    fn line_length(
        &mut self,
        text_system: &Arc<WindowTextSystem>,
        font: Font,
        font_size: Pixels,
    ) -> usize {
        let length = match self {
            Block::Newline(newline) => newline.line_length(text_system, font, font_size),
            Block::Headline(headline) => headline.line_length(text_system, font, font_size),
            Block::Paragraph(paragraph) => paragraph.line_length(text_system, font, font_size),
        };

        return length;
    }
}

impl Size for Newline {
    fn line_length(
        &mut self,
        _text_system: &Arc<WindowTextSystem>,
        _font: Font,
        _font_size: Pixels,
    ) -> usize {
        return 1;
    }
}

impl Size for Headline {
    fn line_length(
        &mut self,
        text_system: &Arc<WindowTextSystem>,
        font: Font,
        font_size: Pixels,
    ) -> usize {
        let lines = self.render(text_system, font, font_size);

        let sum = lines.iter().fold(0, |accumulator, line| {
            accumulator + line.wrap_boundaries.len() + 1
        });

        return sum;
    }
}

impl Size for Paragraph {
    fn line_length(
        &mut self,
        text_system: &Arc<WindowTextSystem>,
        font: Font,
        font_size: Pixels,
    ) -> usize {
        let lines = self.render(text_system, font, font_size);

        let sum = lines.iter().fold(0, |accumulator, line| {
            accumulator + line.wrap_boundaries.len() + 1
        });

        return sum;
    }
}
