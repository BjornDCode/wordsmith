use std::ops::Index;

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

        if content.starts_with('#') {
            let level = content
                .chars()
                .take_while(|&character| character == '#')
                .count();

            return content.chars().skip(level + 1).collect();
        }

        return content;
    }
}

#[derive(Debug, Clone)]
pub struct WrappedText {
    text: TrimmedText,
}

impl WrappedText {
    pub fn new(text: String) -> WrappedText {
        WrappedText {
            text: TrimmedText::new(text),
        }
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

        return offset + wrap_points_before_offset;
    }

    pub fn previous_word_boundary(&self, offset: usize) -> Option<usize> {
        if offset == 0 {
            return None;
        }

        let resolved_offset = self.resolve_offset(offset);

        let content = self.to_string();
        let slice = &content[..resolved_offset];

        let initial_whitespaces = slice
            .chars()
            .rev()
            .take_while(|character| character.is_whitespace())
            .count();

        let index = slice
            .chars()
            .rev()
            .skip(initial_whitespaces)
            .position(|character| character.is_whitespace());

        match index {
            Some(index) => Some(offset - index - initial_whitespaces),
            None => Some(0),
        }
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
