use std::ops::Index;
use std::{cmp::Ordering, ops::Range};

use gpui::{
    div, fill, point, prelude::*, px, rgb, size, AppContext, Bounds, FocusHandle, FocusableView,
    Font, FontWeight, Hsla, LineLayout, PaintQuad, Pixels, Point, Position, ShapedLine,
    SharedString, Style, TextRun, View, ViewContext,
};

// use crate::content::{Block, Content, Size};
// use crate::content::{Render, RenderedBlock};
use crate::{
    content::{Content, Line, LineType},
    text::WrappedText,
    Backspace, MoveBeginningOfFile, MoveBeginningOfLine, MoveBeginningOfWord, MoveDown,
    MoveEndOfFile, MoveEndOfLine, MoveEndOfWord, MoveLeft, MoveRight, MoveUp, RemoveSelection,
    SelectBeginningOfFile, SelectBeginningOfLine, SelectBeginningOfWord, SelectDown,
    SelectEndOfFile, SelectEndOfLine, SelectEndOfWord, SelectLeft, SelectRight, SelectUp,
    COLOR_BLUE_DARK, COLOR_BLUE_LIGHT, COLOR_BLUE_MEDIUM, COLOR_GRAY_700, COLOR_GRAY_800,
    COLOR_PINK,
};

const CHARACTER_WIDTH: Pixels = px(10.24);
pub const CHARACTER_COUNT_PER_LINE: usize = 50;
const EDITOR_HORIZONTAL_MARGIN: Pixels = px(71.68); // 7 (6 headline markers + 1 space) * CHARACTERWIDTH;
const EDITOR_BASE_WIDTH: Pixels = px(512.);
pub const CONTAINER_WIDTH: Pixels = px(655.36); // Base width + Margin * 2

pub struct Editor {
    focus_handle: FocusHandle,
    content: Content,
    edit_location: EditLocation,
}

#[derive(Debug, Clone)]
enum EditLocation {
    Cursor(Cursor),
    Selection(Selection),
}

// impl EditLocation {
//     pub fn starting_point(&self, next_direction: SelectionDirection) -> CursorPoint {
//         match self.clone() {
//             EditLocation::Cursor(cursor) => cursor.position,
//             EditLocation::Selection(selection) => {
//                 let reversed = next_direction == SelectionDirection::Backwards;

//                 match (selection.direction(), reversed) {
//                     (SelectionDirection::Backwards, true) => selection.end,
//                     (SelectionDirection::Backwards, false) => selection.start,
//                     (SelectionDirection::Forwards, true) => selection.start,
//                     (SelectionDirection::Forwards, false) => selection.end,
//                 }
//             }
//         }
//     }
// }

// #[derive(Debug, Clone)]
// pub struct CursorPoint {
//     pub block_index: usize,
//     pub offset: usize,
// }

// impl CursorPoint {
//     pub fn new(block_index: usize, offset: usize) -> CursorPoint {
//         CursorPoint {
//             block_index,
//             offset,
//         }
//     }
// }

// impl PartialEq for CursorPoint {
//     fn eq(&self, other: &Self) -> bool {
//         self.block_index == other.block_index && self.offset == other.offset
//     }
// }

// impl Eq for CursorPoint {}

// impl Ord for CursorPoint {
//     fn cmp(&self, other: &Self) -> Ordering {
//         self.partial_cmp(other).unwrap()
//     }
// }

// impl PartialOrd for CursorPoint {
//     fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
//         if self.block_index == other.block_index {
//             if self.offset < other.offset {
//                 return Some(Ordering::Less);
//             }

//             if self.offset > other.offset {
//                 return Some(Ordering::Greater);
//             }

//             return Some(Ordering::Equal);
//         }

//         if self.block_index < other.block_index {
//             return Some(Ordering::Less);
//         }

//         if self.block_index > other.block_index {
//             return Some(Ordering::Greater);
//         }

//         return Some(Ordering::Equal);
//     }
// }

#[derive(Debug, Clone)]
pub struct EditorPosition {
    pub x: isize,
    pub y: usize,
}

impl EditorPosition {
    pub fn new(y: usize, x: isize) -> EditorPosition {
        return EditorPosition { x, y };
    }
}

impl PartialEq for EditorPosition {
    fn eq(&self, other: &Self) -> bool {
        return self.x == other.x && self.y == other.y;
    }
}

impl Eq for EditorPosition {}

impl Ord for EditorPosition {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialOrd for EditorPosition {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.y == other.y {
            if self.x < other.x {
                return Some(Ordering::Less);
            }

            if self.x > other.x {
                return Some(Ordering::Greater);
            }

            return Some(Ordering::Equal);
        }

        if self.y < other.y {
            return Some(Ordering::Less);
        }

        if self.y > other.y {
            return Some(Ordering::Greater);
        }

        return Some(Ordering::Equal);
    }
}

#[derive(Debug, Clone)]
struct Cursor {
    position: EditorPosition,
    preferred_x: isize,
}

impl Cursor {
    pub fn new(y: usize, x: isize, preferred_x: isize) -> Cursor {
        return Cursor {
            position: EditorPosition::new(y, x),
            preferred_x,
        };
    }
}

#[derive(Debug, Clone)]
struct Selection {
    start: EditorPosition,
    end: EditorPosition,
}

impl Selection {
    pub fn new(start: EditorPosition, end: EditorPosition) -> Selection {
        return Selection { start, end };
    }
}

impl Selection {
    pub fn direction(&self) -> SelectionDirection {
        if self.end < self.start {
            SelectionDirection::Backwards
        } else {
            SelectionDirection::Forwards
        }
    }

    pub fn smallest(&self) -> EditorPosition {
        if self.start < self.end {
            return self.start.clone();
        } else {
            return self.end.clone();
        }
    }

    pub fn largest(&self) -> EditorPosition {
        if self.start > self.end {
            return self.start.clone();
        } else {
            return self.end.clone();
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SelectionDirection {
    Backwards,
    Forwards,
}

impl Editor {
    pub fn new(focus_handle: FocusHandle) -> Editor {
        let edit_location = EditLocation::Cursor(Cursor::new(0, 1, 1));
        // let edit_location = EditLocation::Selection(Selection::new(
        //     EditorPosition::new(0, -2),
        //     EditorPosition::new(5, 20),
        // ));

        return Editor {
            focus_handle,
            content: Content::new("## This is a very very very very very very very very headline\n\nDolor elend vitae porta iaculis etiam commodo. Mus erat lacus penatibus congue ultricies. Elementum tristique sociosqu curae etiam consequat et arcu placerat est.\n\nHabitant primis praesent malesuada lorem parturient lobortis metus. Pulvinar ultrices ligula id ac quisque curae, leo est.\n\n### Another headline\n\nYo, some more text\n\n## Headline".into()),
            edit_location
        };
    }

    fn move_left(&mut self, _: &MoveLeft, context: &mut ViewContext<Self>) {
        let starting_point = match self.edit_location.clone() {
            EditLocation::Cursor(cursor) => cursor.position,
            EditLocation::Selection(selection) => selection.smallest(),
        };

        let position = self.left_position(starting_point.clone());

        self.move_to(position.clone(), position.x, context);
    }

    fn move_right(&mut self, _: &MoveRight, context: &mut ViewContext<Self>) {
        let starting_point = match self.edit_location.clone() {
            EditLocation::Cursor(cursor) => cursor.position,
            EditLocation::Selection(selection) => selection.largest(),
        };

        let position = self.right_position(starting_point.clone());

        self.move_to(position.clone(), position.x, context);
    }

    fn move_up(&mut self, _: &MoveUp, context: &mut ViewContext<Self>) {
        let starting_point = match self.edit_location.clone() {
            EditLocation::Cursor(cursor) => cursor.position,
            EditLocation::Selection(selection) => selection.smallest(),
        };
        let preferred_x = match self.edit_location.clone() {
            EditLocation::Cursor(cursor) => cursor.preferred_x,
            EditLocation::Selection(selection) => selection.smallest().x,
        };

        let position = self.up_position(starting_point.clone(), preferred_x);

        let preferred_x = match self.edit_location.clone() {
            EditLocation::Cursor(cursor) => cursor.preferred_x,
            EditLocation::Selection(_selection) => position.x,
        };

        self.move_to(position, preferred_x, context);
    }

    fn move_down(&mut self, _: &MoveDown, context: &mut ViewContext<Self>) {
        let starting_point = match self.edit_location.clone() {
            EditLocation::Cursor(cursor) => cursor.position,
            EditLocation::Selection(selection) => selection.largest(),
        };
        let preferred_x = match self.edit_location.clone() {
            EditLocation::Cursor(cursor) => cursor.preferred_x,
            EditLocation::Selection(selection) => selection.largest().x,
        };

        let position = self.down_position(starting_point.clone(), preferred_x);
        let preferred_x = match self.edit_location.clone() {
            EditLocation::Cursor(cursor) => cursor.preferred_x,
            EditLocation::Selection(_selection) => position.x,
        };

        self.move_to(position, preferred_x, context);
    }

    fn move_beginning_of_file(&mut self, _: &MoveBeginningOfFile, context: &mut ViewContext<Self>) {
        let position = self.beginning_of_file_position();

        self.move_to(position.clone(), position.x, context);
    }

    fn move_end_of_file(&mut self, _: &MoveEndOfFile, context: &mut ViewContext<Self>) {
        let position = self.end_of_file_position();

        let line = self.content.line(position.y);

        self.move_to(position, line.end(), context);
    }

    fn move_beginning_of_line(&mut self, _: &MoveBeginningOfLine, context: &mut ViewContext<Self>) {
        let starting_point = match self.edit_location.clone() {
            EditLocation::Cursor(cursor) => cursor.position,
            EditLocation::Selection(selection) => match selection.direction() {
                SelectionDirection::Backwards => selection.smallest(),
                SelectionDirection::Forwards => selection.largest(),
            },
        };
        let position = self.beginning_of_line_position(starting_point);

        self.move_to(position.clone(), position.x, context);
    }

    fn move_end_of_line(&mut self, _: &MoveEndOfLine, context: &mut ViewContext<Self>) {
        let starting_point = match self.edit_location.clone() {
            EditLocation::Cursor(cursor) => cursor.position,
            EditLocation::Selection(selection) => match selection.direction() {
                SelectionDirection::Backwards => selection.smallest(),
                SelectionDirection::Forwards => selection.largest(),
            },
        };
        let position = self.end_of_line_position(starting_point);

        self.move_to(position.clone(), position.x, context);
    }

    fn move_beginning_of_word(&mut self, _: &MoveBeginningOfWord, context: &mut ViewContext<Self>) {
        let starting_point = match self.edit_location.clone() {
            EditLocation::Cursor(cursor) => cursor.position,
            EditLocation::Selection(selection) => selection.smallest(),
        };
        let position = self.beginning_of_word_position(starting_point);

        self.move_to(position.clone(), position.x, context);
    }

    fn move_end_of_word(&mut self, _: &MoveEndOfWord, context: &mut ViewContext<Self>) {
        let starting_point = match self.edit_location.clone() {
            EditLocation::Cursor(cursor) => cursor.position,
            EditLocation::Selection(selection) => selection.largest(),
        };
        let position = self.end_of_word_position(starting_point);

        self.move_to(position.clone(), position.x, context);
    }

    // fn select_left(&mut self, _: &SelectLeft, context: &mut ViewContext<Self>) {
    //     match self.edit_location.clone() {
    //         EditLocation::Cursor(cursor) => self.select(
    //             cursor.position.clone(),
    //             self.left_position(cursor.position),
    //             context,
    //         ),
    //         EditLocation::Selection(selection) => {
    //             self.select(selection.start, self.left_position(selection.end), context)
    //         }
    //     }
    // }

    // fn select_right(&mut self, _: &SelectRight, context: &mut ViewContext<Self>) {
    //     match self.edit_location.clone() {
    //         EditLocation::Cursor(cursor) => self.select(
    //             cursor.position.clone(),
    //             self.right_position(cursor.position),
    //             context,
    //         ),
    //         EditLocation::Selection(selection) => {
    //             self.select(selection.start, self.right_position(selection.end), context)
    //         }
    //     }
    // }

    // fn select_up(&mut self, _: &SelectUp, context: &mut ViewContext<Self>) {
    //     match self.edit_location.clone() {
    //         EditLocation::Cursor(cursor) => {
    //             self.select_to(self.up_position(cursor.position), context);
    //         }
    //         EditLocation::Selection(selection) => match selection.direction() {
    //             SelectionDirection::Backwards => {
    //                 self.select_to(self.up_position(selection.end), context)
    //             }
    //             SelectionDirection::Forwards => {
    //                 self.select(selection.start, self.up_position(selection.end), context)
    //             }
    //         },
    //     };
    // }

    // fn select_down(&mut self, _: &SelectDown, context: &mut ViewContext<Self>) {
    //     match self.edit_location.clone() {
    //         EditLocation::Cursor(cursor) => {
    //             self.select_to(self.down_position(cursor.position), context);
    //         }
    //         EditLocation::Selection(selection) => match selection.direction() {
    //             SelectionDirection::Backwards => {
    //                 self.select_to(self.down_position(selection.end), context)
    //             }
    //             SelectionDirection::Forwards => {
    //                 self.select(selection.start, self.down_position(selection.end), context)
    //             }
    //         },
    //     };
    // }

    // fn select_beginning_of_file(
    //     &mut self,
    //     _: &SelectBeginningOfFile,
    //     context: &mut ViewContext<Self>,
    // ) {
    //     let starting_point = match self.edit_location.clone() {
    //         EditLocation::Cursor(cursor) => cursor.position,
    //         EditLocation::Selection(selection) => selection.start,
    //     };

    //     self.select(starting_point, self.beginning_of_file_position(), context);
    // }

    // fn select_end_of_file(&mut self, _: &SelectEndOfFile, context: &mut ViewContext<Self>) {
    //     let starting_point = match self.edit_location.clone() {
    //         EditLocation::Cursor(cursor) => cursor.position,
    //         EditLocation::Selection(selection) => selection.start,
    //     };

    //     self.select(starting_point, self.end_of_file_position(), context);
    // }

    // fn select_beginning_of_line(
    //     &mut self,
    //     _: &SelectBeginningOfLine,
    //     context: &mut ViewContext<Self>,
    // ) {
    //     match self.edit_location.clone() {
    //         EditLocation::Cursor(cursor) => self.select(
    //             cursor.position.clone(),
    //             self.beginning_of_line_position(cursor.position),
    //             context,
    //         ),
    //         EditLocation::Selection(selection) => match selection.direction() {
    //             SelectionDirection::Backwards => {
    //                 self.select_to(self.beginning_of_line_position(selection.end), context)
    //             }
    //             SelectionDirection::Forwards => self.select(
    //                 selection.start,
    //                 self.beginning_of_line_position(selection.end),
    //                 context,
    //             ),
    //         },
    //     };
    // }

    // fn select_end_of_line(&mut self, _: &SelectEndOfLine, context: &mut ViewContext<Self>) {
    //     match self.edit_location.clone() {
    //         EditLocation::Cursor(cursor) => self.select(
    //             cursor.position.clone(),
    //             self.end_of_line_position(cursor.position),
    //             context,
    //         ),
    //         EditLocation::Selection(selection) => match selection.direction() {
    //             SelectionDirection::Backwards => {
    //                 self.select_to(self.end_of_line_position(selection.end), context)
    //             }
    //             SelectionDirection::Forwards => self.select(
    //                 selection.start,
    //                 self.end_of_line_position(selection.end),
    //                 context,
    //             ),
    //         },
    //     };
    // }

    // fn select_beginning_of_word(
    //     &mut self,
    //     _: &SelectBeginningOfWord,
    //     context: &mut ViewContext<Self>,
    // ) {
    //     match self.edit_location.clone() {
    //         EditLocation::Cursor(cursor) => {
    //             self.select_to(self.beginning_of_word_position(cursor.position), context);
    //         }
    //         EditLocation::Selection(selection) => match selection.direction() {
    //             SelectionDirection::Backwards => {
    //                 self.select_to(self.beginning_of_word_position(selection.end), context)
    //             }
    //             SelectionDirection::Forwards => self.select(
    //                 selection.start,
    //                 self.beginning_of_word_position(selection.end),
    //                 context,
    //             ),
    //         },
    //     };
    // }

    // fn select_end_of_word(&mut self, _: &SelectEndOfWord, context: &mut ViewContext<Self>) {
    //     match self.edit_location.clone() {
    //         EditLocation::Cursor(cursor) => {
    //             self.select_to(self.end_of_word_position(cursor.position), context);
    //         }
    //         EditLocation::Selection(selection) => match selection.direction() {
    //             SelectionDirection::Backwards => {
    //                 self.select_to(self.end_of_word_position(selection.end), context)
    //             }
    //             SelectionDirection::Forwards => self.select(
    //                 selection.start,
    //                 self.end_of_word_position(selection.end),
    //                 context,
    //             ),
    //         },
    //     };
    // }

    // fn remove_selection(&mut self, _: &RemoveSelection, context: &mut ViewContext<Self>) {
    //     if let EditLocation::Selection(selection) = self.edit_location.clone() {
    //         let preferred_x = self.preferred_x(selection.start.clone());

    //         self.move_to(selection.start, preferred_x, context);
    //     }
    // }

    fn backspace(&mut self, _: &Backspace, context: &mut ViewContext<Self>) {
        match self.edit_location.clone() {
            EditLocation::Cursor(cursor) => {
                if cursor.position == self.beginning_of_file_position() {
                    return;
                }

                let line = self.content.line(cursor.position.y);

                match (line.clone().kind, cursor.position.x) {
                    (LineType::HeadlineStart(level), 0) => {
                        let position = EditorPosition::new(cursor.position.y, line.beginning());
                        let range = position.clone()..cursor.position;

                        self.replace_range(range, "".into(), context);
                    }
                    _ => {
                        let position = self.left_position(cursor.position.clone());
                        let range = position.clone()..cursor.position;

                        self.replace_range(range, "".into(), context);

                        if position.x >= 0 {
                            self.move_to(position.clone(), position.x, context);
                        }
                    }
                };
            }
            EditLocation::Selection(selection) => todo!(),
        }
    }

    fn move_to(
        &mut self,
        position: EditorPosition,
        preferred_x: isize,
        context: &mut ViewContext<Self>,
    ) {
        self.edit_location = EditLocation::Cursor(Cursor {
            position,
            preferred_x,
        });

        context.notify();
    }

    // fn select(&mut self, start: CursorPoint, end: CursorPoint, context: &mut ViewContext<Self>) {
    //     if start == end {
    //         let preferred_x = self.preferred_x(start.clone());

    //         self.move_to(start, preferred_x, context);
    //     } else {
    //         self.edit_location = EditLocation::Selection(Selection { start, end });
    //     }

    //     context.notify();
    // }

    // fn select_to(&mut self, end: CursorPoint, context: &mut ViewContext<Self>) {
    //     let start = match self.edit_location.clone() {
    //         EditLocation::Cursor(cursor) => cursor.position,
    //         EditLocation::Selection(selection) => selection.start,
    //     };

    //     self.select(start, end, context);
    // }

    fn replace_range(
        &mut self,
        range: Range<EditorPosition>,
        replacement: String,
        context: &mut ViewContext<Self>,
    ) {
        let start_offset = self.content.position_to_offset(range.start);
        let end_offset = self.content.position_to_offset(range.end);

        self.content.replace(start_offset..end_offset, replacement);

        context.notify();
    }

    fn left_position(&self, point: EditorPosition) -> EditorPosition {
        let line = self.content.line(point.y);

        if point.y == 0 && point.x == line.beginning() {
            return point;
        }

        if point.x == line.beginning() {
            let new_line_index = point.y - 1;
            let line = self.content.line(new_line_index);

            return EditorPosition::new(new_line_index, line.end());
        }

        return EditorPosition::new(point.y, point.x - 1);
    }

    fn right_position(&self, point: EditorPosition) -> EditorPosition {
        let line_length = self.content.lines().len();
        let line = self.content.line(point.y);

        if point.y == line_length - 1 && point.x == line.end() {
            return point;
        }

        if point.x == line.end() {
            return EditorPosition::new(point.y + 1, 0);
        }

        return EditorPosition::new(point.y, point.x + 1);
    }

    fn up_position(&self, point: EditorPosition, preferred_x: isize) -> EditorPosition {
        if point.y == 0 {
            let line = self.content.line(0);
            return EditorPosition::new(0, line.beginning());
        }

        let previous_line = self.content.line(point.y - 1);

        let x = previous_line.clamp_x(preferred_x);

        return EditorPosition::new(point.y - 1, x);
    }

    fn down_position(&self, point: EditorPosition, preferred_x: isize) -> EditorPosition {
        let line = self.content.line(point.y);

        if point.y == self.content.lines().len() - 1 {
            return EditorPosition::new(point.y, line.end());
        }

        let next_line = self.content.line(point.y + 1);

        let x = next_line.clamp_x(preferred_x);

        return EditorPosition::new(point.y + 1, x);
    }

    fn beginning_of_file_position(&self) -> EditorPosition {
        let line = self.content.line(0);

        return EditorPosition::new(0, line.beginning());
    }

    fn end_of_file_position(&self) -> EditorPosition {
        let y = self.content.lines().len() - 1;
        let line = self.content.line(y);

        return EditorPosition::new(y, line.end());
    }

    fn beginning_of_line_position(&self, point: EditorPosition) -> EditorPosition {
        let line = self.content.line(point.y);

        return EditorPosition::new(point.y, line.beginning());
    }

    fn end_of_line_position(&self, point: EditorPosition) -> EditorPosition {
        let line = self.content.line(point.y);

        return EditorPosition::new(point.y, line.end());
    }

    fn beginning_of_word_position(&self, point: EditorPosition) -> EditorPosition {
        let line = self.content.line(point.y);
        let line_offset = (point.x - line.beginning()) as usize;

        // Handle edge case: at beginning of file
        if point.y == 0 && point.x <= line.beginning() {
            return self.beginning_of_file_position();
        }

        // First attempt: find previous word boundary in current line
        if point.x > line.beginning() && line_offset <= line.text.len() {
            let wrapped_text = WrappedText::new(line.text.clone());
            let word_boundary = wrapped_text.previous_word_boundary(line_offset);
            let new_x = line.beginning() + (word_boundary as isize);

            // Use this position if it's actually before the current position
            if new_x < point.x {
                return EditorPosition::new(point.y, new_x);
            }
        }

        // If we're here, we need to look at previous line

        // Handle edge case: already at first line
        if point.y == 0 {
            return self.beginning_of_file_position();
        }

        // Get previous line
        let previous_line = self.content.line(point.y - 1);

        // Handle edge case: previous line is empty
        if previous_line.text.trim().is_empty() {
            return EditorPosition::new(point.y - 1, previous_line.beginning());
        }

        // Find last word in previous line
        let wrapped_text = WrappedText::new(previous_line.text.clone());
        let last_valid_offset = previous_line.text.len();
        let word_boundary = wrapped_text.previous_word_boundary(last_valid_offset);
        let new_x = previous_line.beginning() + (word_boundary as isize);

        return EditorPosition::new(point.y - 1, new_x);
    }

    fn end_of_word_position(&self, point: EditorPosition) -> EditorPosition {
        let line = self.content.line(point.y);
        let line_offset = (point.x - line.beginning()) as usize;

        // First attempt: find next word boundary in current line
        if line_offset < line.text.len() {
            let wrapped_text = WrappedText::new(line.text.clone());

            if let Some(word_boundary) = wrapped_text.next_word_boundary(line_offset) {
                let new_x = line.beginning() + (word_boundary as isize);

                // Use this position if it doesn't exceed the end of the line
                if new_x <= line.end() {
                    return EditorPosition::new(point.y, new_x);
                }
            }
        }

        // If we're here, we need to look at the next line

        // Handle edge case: already at last line
        if point.y >= self.content.lines().len() - 1 {
            return EditorPosition::new(point.y, line.end());
        }

        // Get next line
        let next_line = self.content.line(point.y + 1);

        // Handle edge case: next line is empty
        if next_line.text.trim().is_empty() {
            return EditorPosition::new(point.y + 1, next_line.beginning());
        }

        // Find first non-whitespace character in next line
        let mut start_offset = 0;
        while start_offset < next_line.text.len()
            && next_line
                .text
                .chars()
                .nth(start_offset)
                .unwrap_or(' ')
                .is_whitespace()
        {
            start_offset += 1;
        }

        // If we reached the end of the next line
        if start_offset >= next_line.text.len() {
            return EditorPosition::new(point.y + 1, next_line.beginning());
        }

        // Go to the first word in the next line
        return EditorPosition::new(point.y + 1, next_line.beginning() + start_offset as isize);
    }
}

impl FocusableView for Editor {
    fn focus_handle(&self, _context: &AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl gpui::Render for Editor {
    fn render(&mut self, context: &mut gpui::ViewContext<Self>) -> impl gpui::IntoElement {
        div()
            .track_focus(&self.focus_handle(context))
            .key_context("editor")
            .on_action(context.listener(Self::move_left))
            .on_action(context.listener(Self::move_right))
            .on_action(context.listener(Self::move_up))
            .on_action(context.listener(Self::move_down))
            .on_action(context.listener(Self::move_beginning_of_file))
            .on_action(context.listener(Self::move_end_of_file))
            .on_action(context.listener(Self::move_beginning_of_line))
            .on_action(context.listener(Self::move_end_of_line))
            .on_action(context.listener(Self::move_beginning_of_word))
            .on_action(context.listener(Self::move_end_of_word))
            // .on_action(context.listener(Self::select_left))
            // .on_action(context.listener(Self::select_right))
            // .on_action(context.listener(Self::select_up))
            // .on_action(context.listener(Self::select_down))
            // .on_action(context.listener(Self::select_beginning_of_file))
            // .on_action(context.listener(Self::select_end_of_file))
            // .on_action(context.listener(Self::select_beginning_of_line))
            // .on_action(context.listener(Self::select_end_of_line))
            // .on_action(context.listener(Self::select_beginning_of_word))
            // .on_action(context.listener(Self::select_end_of_word))
            // .on_action(context.listener(Self::remove_selection))
            .on_action(context.listener(Self::backspace))
            .pt_8()
            .group("editor-container")
            .bg(rgb(COLOR_PINK))
            .child(
                div()
                    .bg(rgb(COLOR_PINK))
                    .w(CONTAINER_WIDTH)
                    .line_height(px(24.))
                    .child(EditorElement {
                        input: context.view().clone(),
                    }),
            )
    }
}

struct EditorElement {
    input: View<Editor>,
}

// impl ViewInputHandler for Editor {}

impl IntoElement for EditorElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

#[derive(Debug, Clone)]
struct RenderedLine {
    shaped_line: ShapedLine,
    raw_line: Line,
}

impl RenderedLine {
    pub fn new(raw_line: Line, shaped_line: ShapedLine) -> RenderedLine {
        return RenderedLine {
            raw_line,
            shaped_line,
        };
    }
}

struct PrepaintState {
    lines: Vec<RenderedLine>,
    edit_location_rectangles: Vec<PaintQuad>,
    // blocks: Vec<RenderedBlock>,
    // headline_markers: Vec<RenderedHeadlineMarker>,
}

impl Element for EditorElement {
    type RequestLayoutState = ();
    type PrepaintState = PrepaintState;

    fn id(&self) -> Option<gpui::ElementId> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&gpui::GlobalElementId>,
        context: &mut gpui::WindowContext,
    ) -> (gpui::LayoutId, Self::RequestLayoutState) {
        let style = Style::default();

        (context.request_layout(style, []), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&gpui::GlobalElementId>,
        bounds: gpui::Bounds<gpui::Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        context: &mut gpui::WindowContext,
    ) -> Self::PrepaintState {
        let input = self.input.read(context);
        let content = input.content.clone();
        let style = context.text_style();
        let font_size = style.font_size.to_pixels(context.rem_size());

        let mut lines: Vec<RenderedLine> = vec![];
        let mut is_inside_headline = false;

        // let text = content.text();
        // let raw_lines: Vec<_> = text.lines().map(|s| s.to_string()).collect();
        let raw_lines = content.lines();
        for line in &raw_lines {
            let run = match line.kind {
                LineType::HeadlineStart(_) => TextRun {
                    len: line.length(),
                    font: Font {
                        weight: FontWeight::EXTRA_BOLD,
                        ..style.font()
                    },
                    color: Hsla::from(rgb(COLOR_GRAY_800)),
                    background_color: None,
                    underline: None,
                    strikethrough: None,
                },
                LineType::HeadlineNotStart => TextRun {
                    len: line.length(),
                    font: Font {
                        weight: FontWeight::EXTRA_BOLD,
                        ..style.font()
                    },
                    color: Hsla::from(rgb(COLOR_GRAY_800)),
                    background_color: None,
                    underline: None,
                    strikethrough: None,
                },
                LineType::Normal => TextRun {
                    len: line.length(),
                    font: style.font(),
                    color: Hsla::from(rgb(COLOR_GRAY_700)),
                    background_color: None,
                    underline: None,
                    strikethrough: None,
                },
            };
            let runs = vec![run];

            let shaped_line = context
                .text_system()
                .shape_line(SharedString::from(line.text.clone()), font_size, &runs)
                .unwrap();

            lines.push(RenderedLine::new(line.clone(), shaped_line));

            // let is_start_of_headline = line.starts_with('#');

            // if is_start_of_headline {
            //     is_inside_headline = true;
            // }

            // if line.is_empty() {
            //     is_inside_headline = false;
            // }

            // if is_inside_headline {
            //     let runs = vec![TextRun {
            //         len: line.len(),
            //         font: Font {
            //             weight: FontWeight::EXTRA_BOLD,
            //             ..style.font()
            //         },
            //         color: Hsla::from(rgb(COLOR_GRAY_800)),
            //         background_color: None,
            //         underline: None,
            //         strikethrough: None,
            //     }];
            //     let shaped_line = context
            //         .text_system()
            //         .shape_line(SharedString::from(line), font_size, &runs)
            //         .unwrap();

            //     let level = line
            //         .chars()
            //         .take_while(|&character| character == '#')
            //         .count();

            //     let kind = if is_start_of_headline {
            //         LineType::HeadlineStart(level)
            //     } else {
            //         LineType::HeadlineNotStart
            //     };

            //     lines.push(RenderedLine::new(kind, shaped_line));
            // } else {
            //     let runs = vec![TextRun {
            //         len: line.len(),
            //         font: style.font(),
            //         color: Hsla::from(rgb(COLOR_GRAY_700)),
            //         background_color: None,
            //         underline: None,
            //         strikethrough: None,
            //     }];
            //     let shaped_line = context
            //         .text_system()
            //         .shape_line(SharedString::from(line), font_size, &runs)
            //         .unwrap();

            //     lines.push(RenderedLine::new(LineType::Regular, shaped_line));
            // }
        }

        let edit_location_rectangles = match input.edit_location.clone() {
            EditLocation::Cursor(cursor) => {
                let left = bounds.left()
                    + EDITOR_HORIZONTAL_MARGIN
                    + px(cursor.position.x as f32) * CHARACTER_WIDTH
                    - px(1.);
                let top =
                    bounds.top() + context.line_height() * px(cursor.position.y as f32) + px(2.);

                let rectangles = vec![fill(
                    Bounds::new(point(left, top), size(px(2.), px(20.))),
                    rgb(COLOR_BLUE_DARK),
                )];

                rectangles
            }
            EditLocation::Selection(selection) => {
                let mut rectangles = vec![];

                let smallest = selection.smallest();
                let largest = selection.largest();

                let line_range = smallest.y..largest.y + 1;

                for index in line_range.clone() {
                    let start = if index == line_range.start {
                        smallest.x
                    } else {
                        0
                    };
                    let end = if index == line_range.end - 1 {
                        largest.x
                    } else {
                        CHARACTER_COUNT_PER_LINE as isize
                    };

                    let left = bounds.left()
                        + EDITOR_HORIZONTAL_MARGIN
                        + px(start as f32) * CHARACTER_WIDTH
                        - px(1.);
                    let top = bounds.top() + px(index as f32) * context.line_height();
                    let width = px((end - start) as f32) * CHARACTER_WIDTH + px(2.);

                    let bounds = Bounds::new(point(left, top), size(width, context.line_height()));
                    let rectangle = fill(bounds, rgb(COLOR_BLUE_MEDIUM));

                    rectangles.push(rectangle);
                }

                rectangles

                //         let mut rectangles = vec![];
                //         let smallest_point = std::cmp::min(selection.start.clone(), selection.end.clone());
                //         let largest_point = std::cmp::max(selection.start.clone(), selection.end.clone());
                //         let block_range = smallest_point.block_index..largest_point.block_index + 1;

                //         for block_index in block_range.clone() {
                //             let block_start_line_index = content.block_start(block_index);
                //             let block = content.block(block_index);
                //             let min = if block_index == block_range.start {
                //                 smallest_point.offset
                //             } else {
                //                 0
                //             };
                //             let max = if block_index == block_range.end - 1 {
                //                 largest_point.offset
                //             } else {
                //                 block.length()
                //             };
                //             let line_range = block.line_range(min, max);

                //             for line_index in line_range.clone() {
                //                 let start = if line_index == line_range.start {
                //                     block.offset_in_line(line_index, min)
                //                 } else {
                //                     0
                //                 };
                //                 let end = if block_index == block_range.end - 1
                //                     && line_index == line_range.end - 1
                //                 {
                //                     if block_index == block_range.end - 1 {
                //                         block.offset_in_line(line_index, max)
                //                     } else {
                //                         let offset = block.offset_in_line(line_index, min);

                //                         CHARACTER_COUNT_PER_LINE - offset
                //                     }
                //                 } else {
                //                     CHARACTER_COUNT_PER_LINE - start
                //                 };

                //                 let left = bounds.left() + px(start as f32) * CHARACTER_WIDTH - px(1.);
                //                 let top = bounds.top()
                //                     + px(block_start_line_index as f32) * context.line_height()
                //                     + px(line_index as f32) * context.line_height();
                //                 let width = if line_range.start == line_range.end - 1 {
                //                     (px(end as f32) - px(start as f32)) * CHARACTER_WIDTH + px(2.)
                //                 } else {
                //                     px(end as f32) * CHARACTER_WIDTH + px(2.)
                //                 };

                //                 let bounds =
                //                     Bounds::new(point(left, top), size(width, context.line_height()));
                //                 let rectangle = fill(bounds, rgb(COLOR_BLUE_MEDIUM));

                //                 rectangles.push(rectangle);
                //             }
                //         }

                //         rectangles
            }
        };

        // let blocks = content.blocks();

        // let mut headline_markers = vec![];

        // for block in &blocks {
        //     if let Block::Headline(ref headline) = block {
        //         let width = px(16. * headline.level() as f32);
        //         let content = "#".repeat(headline.level()) + " ";
        //         let runs = vec![TextRun {
        //             len: content.len(),
        //             font: Font {
        //                 weight: FontWeight::EXTRA_BOLD,
        //                 ..style.font()
        //             },
        //             color: Hsla::from(rgb(COLOR_GRAY_800)),
        //             background_color: None,
        //             underline: None,
        //             strikethrough: None,
        //         }];
        //         let shaped_text = context
        //             .text_system()
        //             .shape_line(content.into(), font_size, &runs)
        //             .unwrap();
        //         let origin = point(
        //             bounds.origin.x - width,
        //             bounds.origin.y + (context.line_height() * block.line_index()),
        //         );

        //         headline_markers.push(RenderedHeadlineMarker {
        //             shaped_text,
        //             origin,
        //         });
        //     }
        // }

        // let rendered_blocks: Vec<RenderedBlock> = blocks
        //     .into_iter()
        //     .map(|mut block| block.render(context.text_system(), style.font(), font_size))
        //     .collect();

        // let edit_location_rectangles = match input.edit_location.clone() {
        //     EditLocation::Cursor(caret) => {
        //         let position = content.cursor_position(caret.position);

        //         let rectangles = vec![fill(
        //             Bounds::new(
        //                 point(
        //                     bounds.left() + px(position.x as f32) * CHARACTER_WIDTH - px(1.),
        //                     bounds.top() + context.line_height() * px(position.y as f32) + px(2.),
        //                 ),
        //                 size(px(2.), px(20.)),
        //             ),
        //             rgb(COLOR_BLUE_DARK),
        //         )];

        //         rectangles
        //     }
        //     EditLocation::Selection(selection) => {
        //         let mut rectangles = vec![];
        //         let smallest_point = std::cmp::min(selection.start.clone(), selection.end.clone());
        //         let largest_point = std::cmp::max(selection.start.clone(), selection.end.clone());
        //         let block_range = smallest_point.block_index..largest_point.block_index + 1;

        //         for block_index in block_range.clone() {
        //             let block_start_line_index = content.block_start(block_index);
        //             let block = content.block(block_index);
        //             let min = if block_index == block_range.start {
        //                 smallest_point.offset
        //             } else {
        //                 0
        //             };
        //             let max = if block_index == block_range.end - 1 {
        //                 largest_point.offset
        //             } else {
        //                 block.length()
        //             };
        //             let line_range = block.line_range(min, max);

        //             for line_index in line_range.clone() {
        //                 let start = if line_index == line_range.start {
        //                     block.offset_in_line(line_index, min)
        //                 } else {
        //                     0
        //                 };
        //                 let end = if block_index == block_range.end - 1
        //                     && line_index == line_range.end - 1
        //                 {
        //                     if block_index == block_range.end - 1 {
        //                         block.offset_in_line(line_index, max)
        //                     } else {
        //                         let offset = block.offset_in_line(line_index, min);

        //                         CHARACTER_COUNT_PER_LINE - offset
        //                     }
        //                 } else {
        //                     CHARACTER_COUNT_PER_LINE - start
        //                 };

        //                 let left = bounds.left() + px(start as f32) * CHARACTER_WIDTH - px(1.);
        //                 let top = bounds.top()
        //                     + px(block_start_line_index as f32) * context.line_height()
        //                     + px(line_index as f32) * context.line_height();
        //                 let width = if line_range.start == line_range.end - 1 {
        //                     (px(end as f32) - px(start as f32)) * CHARACTER_WIDTH + px(2.)
        //                 } else {
        //                     px(end as f32) * CHARACTER_WIDTH + px(2.)
        //                 };

        //                 let bounds =
        //                     Bounds::new(point(left, top), size(width, context.line_height()));
        //                 let rectangle = fill(bounds, rgb(COLOR_BLUE_MEDIUM));

        //                 rectangles.push(rectangle);
        //             }
        //         }

        //         rectangles
        //     }
        // };

        PrepaintState {
            lines,
            edit_location_rectangles,
            // blocks: rendered_blocks,
            // headline_markers,
        }
    }

    fn paint(
        &mut self,
        _id: Option<&gpui::GlobalElementId>,
        bounds: gpui::Bounds<gpui::Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        context: &mut gpui::WindowContext,
    ) {
        let focus_handle = self.input.read(context).focus_handle.clone();
        let edit_location_rectangles = prepaint.edit_location_rectangles.clone();
        let lines = prepaint.lines.clone();

        if focus_handle.is_focused(context) {
            for rectangle in edit_location_rectangles {
                context.paint_quad(rectangle);
            }
        }

        for (index, line) in lines.iter().enumerate() {
            let offset = match line.raw_line.kind {
                LineType::HeadlineStart(level) => {
                    EDITOR_HORIZONTAL_MARGIN - px(level as f32 + 1 as f32) * CHARACTER_WIDTH
                }
                LineType::HeadlineNotStart => EDITOR_HORIZONTAL_MARGIN,
                LineType::Normal => EDITOR_HORIZONTAL_MARGIN,
            };

            let point = Point::new(
                bounds.origin.x + offset,
                bounds.origin.y + (context.line_height() * index),
            );

            line.shaped_line
                .paint(point, context.line_height(), context)
                .unwrap();
        }

        // let blocks = prepaint.blocks.clone().into_iter();
        // let headline_markers = prepaint.headline_markers.clone();

        // for block in blocks {
        //     // The reason we are not just looping over lines directly is that there seem to be a rogue newline at the end
        //     // So this is a hacky way to avoid that
        //     // Should probably fix that issue properly at some point

        //     let mut line_count = 0;

        //     for index in 0..block.line_length {
        //         let line = &block.lines.index(index);

        //         let offset = match block.block {
        //             Block::Newline(_) => px(0.),
        //             Block::Paragraph(_) => px(0.),
        //             Block::Headline(ref headline) => {
        //                 px((headline.level() + 1) as f32) * CHARACTER_WIDTH
        //             }
        //         };
        //         let point = Point::new(
        //             bounds.origin.x + EDITOR_HORIZONTAL_MARGIN - offset,
        //             bounds.origin.y
        //                 + (context.line_height() * px(block.line_index as f32 + line_count as f32)),
        //         );

        //         line.paint(point, context.line_height(), context).unwrap();

        //         line_count += 1;
        //     }
        // }

        // for marker in headline_markers {
        //     marker.render(context);
        // }
    }
}

#[derive(Debug, Clone)]
pub struct RenderedHeadlineMarker {
    shaped_text: ShapedLine,
    origin: Point<Pixels>,
}

impl RenderedHeadlineMarker {
    pub fn render(&self, context: &mut gpui::WindowContext) {
        self.shaped_text
            .paint(self.origin, context.line_height(), context)
            .unwrap();
    }
}
