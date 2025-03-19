use std::ops::{Index, Range};

use gpui::SharedString;

use crate::{cursor::EditorPosition, text::WrappedText};

#[derive(Debug, Clone)]
pub enum LineType {
    HeadlineStart(usize),
    HeadlineNotStart,
    Normal,
}

#[derive(Debug, Clone)]
pub struct Line {
    pub text: String,
    pub kind: LineType,
}

impl Line {
    pub fn beginning(&self) -> isize {
        return match self.kind {
            LineType::HeadlineStart(level) => level as isize * -1 - 1,
            LineType::HeadlineNotStart => 0,
            LineType::Normal => 0,
        };
    }

    pub fn end(&self) -> isize {
        return match self.kind {
            LineType::HeadlineStart(level) => self.length() as isize - (level as isize) - 1,
            LineType::HeadlineNotStart => self.text.len() as isize,
            LineType::Normal => self.text.len() as isize,
        };
    }

    pub fn length(&self) -> usize {
        return self.text.len();
    }

    pub fn clamp_x(&self, preferred_x: isize) -> isize {
        if preferred_x < self.beginning() {
            return self.beginning();
        }

        if preferred_x > self.end() {
            return self.end();
        }

        return preferred_x;
    }
}

#[derive(Debug, Clone)]
pub struct Content {
    original: SharedString,
    wrapped: WrappedText,
}

impl Content {
    pub fn new(original: SharedString) -> Content {
        let wrapped = WrappedText::new(original.clone().into());

        return Content { original, wrapped };
    }

    pub fn empty() -> Content {
        return Content {
            original: SharedString::new_static(""),
            wrapped: WrappedText::empty(),
        };
    }

    pub fn to_string(&self) -> String {
        return self.original.clone().into();
    }

    pub fn text(&self) -> WrappedText {
        return self.wrapped.clone();
    }

    pub fn lines(&self) -> Vec<Line> {
        let raw_lines: Vec<_> = self
            .text()
            .to_string()
            .lines()
            .map(|s| s.to_string())
            .collect();
        let mut lines: Vec<Line> = vec![];
        let mut is_inside_headline = false;

        for raw in raw_lines {
            let is_start_of_headline = is_headline(raw.clone());

            if is_start_of_headline {
                is_inside_headline = true;
            }

            if raw.is_empty() {
                is_inside_headline = false;
            }

            let kind = if is_start_of_headline {
                let level = raw
                    .chars()
                    .take_while(|&character| character == '#')
                    .count();

                LineType::HeadlineStart(level)
            } else if is_inside_headline {
                LineType::HeadlineNotStart
            } else {
                LineType::Normal
            };

            lines.push(Line { text: raw, kind })
        }

        lines.push(Line {
            text: "".into(),
            kind: LineType::Normal,
        });

        return lines;
    }

    pub fn replace(&mut self, range: Range<usize>, replacement: String) {
        self.wrapped.replace(range, replacement);
        self.original = self.wrapped.original().to_string().into();
    }

    pub fn read_range(&self, range: Range<usize>) -> String {
        return self.text().read_range(range);
    }

    pub fn line(&self, index: usize) -> Line {
        let lines = self.lines();

        return lines.index(index).clone();
    }

    pub fn position_to_offset(&self, position: EditorPosition) -> usize {
        let lines = self.lines();

        if lines.is_empty() {
            return 0;
        }

        let previous_lines = lines.iter().take(position.y);
        let mut offset: isize = 0;

        for line in previous_lines {
            offset += line.length() as isize;
            offset += 1; // Newline
        }

        let line = lines.index(position.y);

        if let LineType::HeadlineStart(level) = line.kind {
            offset += level as isize + 1;
        }

        offset += position.x;

        return std::cmp::max(0, offset) as usize;
    }

    pub fn offset_to_position(&self, offset: usize) -> EditorPosition {
        let lines = self.lines();
        let mut x = offset;
        let mut y = 0;

        for line in lines {
            if x <= line.length() {
                break;
            }

            y += 1;
            x -= line.length();
            x = x.saturating_sub(1);
        }

        let line = self.line(y);

        if let LineType::HeadlineStart(level) = line.kind {
            x -= level + 1;
        }

        return EditorPosition::new(y, x as isize);
    }
}

fn is_headline(line: String) -> bool {
    let trimmed = line.trim_start();

    if !trimmed.starts_with('#') {
        return false;
    }

    let hash_count = trimmed.chars().take_while(|&c| c == '#').count();

    return hash_count > 0
        && hash_count < trimmed.len()
        && trimmed.chars().nth(hash_count) == Some(' ');
}
