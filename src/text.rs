use std::ops::{Index, Range};

use crate::editor::CHARACTER_COUNT_PER_LINE;

#[derive(Debug, Clone)]
struct RawText {
    text: String,
}

impl RawText {
    pub fn new(text: String) -> RawText {
        RawText { text }
    }

    pub fn to_string(&self) -> String {
        return self.text.clone();
    }

    pub fn replace(&mut self, range: Range<usize>, replacement: String) {
        self.text.replace_range(range, &replacement);
    }
}

#[derive(Debug, Clone)]
struct TrimmedText {
    text: RawText,
}

impl TrimmedText {
    pub fn new(text: String) -> TrimmedText {
        TrimmedText {
            text: RawText::new(text),
        }
    }
}

impl TrimmedText {
    pub fn to_string(&self) -> String {
        let content = self.text.to_string();

        // if content.starts_with('#') {
        //     let level = content
        //         .chars()
        //         .take_while(|&character| character == '#')
        //         .count();

        //     return content.chars().skip(level + 1).collect();
        // }

        return content;
    }
}

#[derive(Debug, Clone)]
pub struct WrappedText {
    text: RawText,
}

impl WrappedText {
    pub fn new(text: String) -> WrappedText {
        WrappedText {
            text: RawText::new(text),
        }
    }

    pub fn replace(&mut self, range: Range<usize>, replacement: String) {
        let start_offset = self.resolve_offset(range.start);
        let end_offset = self.resolve_offset(range.end);

        self.text.replace(start_offset..end_offset, replacement);
    }

    pub fn line_length(&self) -> usize {
        let content = self.to_string();

        content.lines().count()
    }

    pub fn lines(&self) -> Vec<String> {
        let content = self.to_string();

        content.lines().map(String::from).collect()
    }

    pub fn lines_and_wrap_points(&self) -> (Vec<String>, Vec<usize>) {
        let (_, wrap_points) = self.to_string_with_wrap_points();

        return (self.lines(), wrap_points);
    }

    pub fn line(&self, line_index: usize) -> String {
        let lines = self.lines();
        let line = lines.index(line_index);

        return line.clone();
    }

    pub fn length(&self) -> usize {
        let (content, wrap_points) = self.to_string_with_wrap_points();

        return content.len() - wrap_points.len();
    }

    pub fn is_soft_wrapped_line(&self, line_index: usize) -> bool {
        let (lines, wrap_points) = self.lines_and_wrap_points();

        let lines = lines.into_iter().enumerate();
        let mut offset = 0;

        for (index, line) in lines {
            offset += line.len();

            if index == line_index {
                break;
            }
        }

        return wrap_points.contains(&offset);
    }

    fn resolve_offset(&self, offset: usize) -> usize {
        let (_, wrap_points) = self.to_string_with_wrap_points();
        let wrap_points_before_offset = wrap_points.iter().filter(|point| **point < offset).count();

        return offset - wrap_points_before_offset;
    }

    pub fn previous_word_boundary(&self, offset: usize) -> usize {
        let content = self.text.to_string();
        let chars: Vec<char> = content.chars().collect();

        // Handle edge cases
        if offset == 0 {
            return 0;
        }

        if offset >= chars.len() {
            return self.previous_word_boundary(chars.len() - 1);
        }

        let mut cursor = offset;

        // Case 1: We're at the beginning of a word already
        let at_word_beginning = offset < chars.len()
            && !chars[offset].is_whitespace()
            && (offset == 0 || chars[offset - 1].is_whitespace());

        if at_word_beginning {
            // Skip back through current word
            while cursor > 0 && !chars[cursor - 1].is_whitespace() {
                cursor -= 1;
            }

            if cursor == 0 {
                return 0;
            }

            // Skip back through whitespace
            while cursor > 0 && chars[cursor - 1].is_whitespace() {
                cursor -= 1;
            }

            // Skip back through previous word
            while cursor > 0 && !chars[cursor - 1].is_whitespace() {
                cursor -= 1;
            }

            return cursor;
        }

        // Case 2: We're in the middle of a word or at whitespace

        // Skip back through whitespace
        while cursor > 0 && chars[cursor - 1].is_whitespace() {
            cursor -= 1;
        }

        // Find the beginning of the current word
        while cursor > 0 && !chars[cursor - 1].is_whitespace() {
            cursor -= 1;
        }

        return cursor;
    }

    pub fn next_word_boundary(&self, offset: usize) -> Option<usize> {
        let content = self.text.to_string();
        let chars: Vec<char> = content.chars().collect();

        // Handle edge case
        if offset >= chars.len() {
            return None;
        }

        let mut cursor = offset;

        // Skip any whitespace after current position
        while cursor < chars.len() && chars[cursor].is_whitespace() {
            cursor += 1;
        }

        // If we reached the end after skipping whitespace
        if cursor >= chars.len() {
            return None;
        }

        // Find end of current word
        while cursor < chars.len() && !chars[cursor].is_whitespace() {
            cursor += 1;
        }

        if cursor == offset {
            return None;
        }

        return Some(cursor);
    }

    fn to_string_with_wrap_points(&self) -> (String, Vec<usize>) {
        let content = self.text.to_string();

        let mut wrap_points: Vec<usize> = vec![];
        let mut output = String::new();
        let mut offset = 0;

        for line in content.lines() {
            let mut cloned_line = line.to_string();

            loop {
                if cloned_line.len() <= CHARACTER_COUNT_PER_LINE {
                    output += cloned_line.as_str();
                    output += "\n";

                    offset += cloned_line.len();

                    break;
                }

                // Take all the characters that would result in a full line
                let soft_wrapped_line_without_wordbreak: String =
                    cloned_line.chars().take(CHARACTER_COUNT_PER_LINE).collect();

                // Find the reverse index (from the back of the line) of the first word break
                let word_break_index_from_back = soft_wrapped_line_without_wordbreak
                    .chars()
                    .rev()
                    .position(|character| character.is_whitespace());

                // If there is no word break in the entire line
                if let None = word_break_index_from_back {
                    panic!("Properly handle 1 word that is too long for a single line");
                }

                // Find the word break index from the front instead
                let word_break_index =
                    CHARACTER_COUNT_PER_LINE - word_break_index_from_back.unwrap();

                // Find the actual content we want to be on this line
                // I.e. all the content up to the last word break before the line is full
                let soft_wrapped_line: String = soft_wrapped_line_without_wordbreak
                    .chars()
                    .take(word_break_index)
                    .collect();

                // Remove the taken content from the full, non-wrapped line content
                cloned_line = cloned_line.split_off(word_break_index);

                output += soft_wrapped_line.as_str();
                output += "\n";

                offset += soft_wrapped_line.len();

                wrap_points.push(offset);

                continue;
            }
        }

        return (output, wrap_points);
    }

    pub fn to_string(&self) -> String {
        let (output, _) = self.to_string_with_wrap_points();

        return output;
    }
}
