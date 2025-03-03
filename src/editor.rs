use std::cmp::Ordering;
use std::ops::Index;

use gpui::{
    div, fill, point, prelude::*, px, rgb, size, AppContext, Bounds, FocusHandle, FocusableView,
    Font, FontWeight, Hsla, LineLayout, PaintQuad, Pixels, Point, ShapedLine, SharedString, Style,
    TextRun, View, ViewContext,
};

// use crate::content::{Block, Content, Size};
// use crate::content::{Render, RenderedBlock};
use crate::{
    content::Content, MoveBeginningOfFile, MoveBeginningOfLine, MoveBeginningOfWord, MoveDown,
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
    // edit_location: EditLocation,
}

#[derive(Debug, Clone)]
enum EditLocation {
    Cursor(Cursor),
    Selection(Selection),
}

impl EditLocation {
    pub fn starting_point(&self, next_direction: SelectionDirection) -> CursorPoint {
        match self.clone() {
            EditLocation::Cursor(cursor) => cursor.position,
            EditLocation::Selection(selection) => {
                let reversed = next_direction == SelectionDirection::Backwards;

                match (selection.direction(), reversed) {
                    (SelectionDirection::Backwards, true) => selection.end,
                    (SelectionDirection::Backwards, false) => selection.start,
                    (SelectionDirection::Forwards, true) => selection.start,
                    (SelectionDirection::Forwards, false) => selection.end,
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct CursorPoint {
    pub block_index: usize,
    pub offset: usize,
}

impl CursorPoint {
    pub fn new(block_index: usize, offset: usize) -> CursorPoint {
        CursorPoint {
            block_index,
            offset,
        }
    }
}

impl PartialEq for CursorPoint {
    fn eq(&self, other: &Self) -> bool {
        self.block_index == other.block_index && self.offset == other.offset
    }
}

impl Eq for CursorPoint {}

impl Ord for CursorPoint {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialOrd for CursorPoint {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.block_index == other.block_index {
            if self.offset < other.offset {
                return Some(Ordering::Less);
            }

            if self.offset > other.offset {
                return Some(Ordering::Greater);
            }

            return Some(Ordering::Equal);
        }

        if self.block_index < other.block_index {
            return Some(Ordering::Less);
        }

        if self.block_index > other.block_index {
            return Some(Ordering::Greater);
        }

        return Some(Ordering::Equal);
    }
}

#[derive(Debug, Clone)]
struct Cursor {
    position: CursorPoint,
    preferred_x: usize,
}

#[derive(Debug, Clone)]
struct Selection {
    start: CursorPoint,
    end: CursorPoint,
}

impl Selection {
    pub fn direction(&self) -> SelectionDirection {
        if self.end < self.start {
            SelectionDirection::Backwards
        } else {
            SelectionDirection::Forwards
        }
    }

    pub fn smallest(&self) -> CursorPoint {
        if self.start < self.end {
            return self.start.clone();
        } else {
            return self.end.clone();
        }
    }

    pub fn largest(&self) -> CursorPoint {
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
        let edit_location = EditLocation::Selection(Selection {
            start: CursorPoint {
                offset: 6,
                block_index: 4,
            },
            end: CursorPoint {
                offset: 65,
                block_index: 4,
            },
        });

        return Editor {
            focus_handle,
            content: Content::new("## This is a headline\n\nDolor elend vitae porta iaculis etiam commodo. Mus erat lacus penatibus congue ultricies. Elementum tristique sociosqu curae etiam consequat et arcu placerat est.\n\nHabitant primis praesent malesuada lorem parturient lobortis metus. Pulvinar ultrices ligula id ac quisque curae, leo est.\n\n### Another headline\n\nYo, some more text\n\n## Headline".into()),
            // edit_location
        };
    }

    // fn move_left(&mut self, _: &MoveLeft, context: &mut ViewContext<Self>) {
    //     match self.edit_location.clone() {
    //         EditLocation::Cursor(cursor) => {
    //             let position = self.left_position(cursor.position);
    //             let preferred_x = self.preferred_x(position.clone());

    //             self.move_to(position, preferred_x, context);
    //         }
    //         EditLocation::Selection(selection) => {
    //             let position = selection.smallest();
    //             let preferred_x = self.preferred_x(position.clone());

    //             self.move_to(position, preferred_x, context);
    //         }
    //     }
    // }

    // fn move_right(&mut self, _: &MoveRight, context: &mut ViewContext<Self>) {
    //     match self.edit_location.clone() {
    //         EditLocation::Cursor(cursor) => {
    //             let position = self.right_position(cursor.position);
    //             let preferred_x = self.preferred_x(position.clone());

    //             self.move_to(position, preferred_x, context);
    //         }
    //         EditLocation::Selection(selection) => {
    //             let position = selection.largest();
    //             let preferred_x = self.preferred_x(position.clone());

    //             self.move_to(position, preferred_x, context);
    //         }
    //     }
    // }

    // fn move_up(&mut self, _: &MoveUp, context: &mut ViewContext<Self>) {
    //     let starting_point = self
    //         .edit_location
    //         .starting_point(SelectionDirection::Backwards);
    //     let position = self.up_position(starting_point.clone());
    //     let preferred_x = match self.edit_location.clone() {
    //         EditLocation::Cursor(cursor) => cursor.preferred_x,
    //         EditLocation::Selection(selection) => self.preferred_x(selection.start),
    //     };

    //     self.move_to(position, preferred_x, context);
    // }

    // fn move_down(&mut self, _: &MoveDown, context: &mut ViewContext<Self>) {
    //     let starting_point = self
    //         .edit_location
    //         .starting_point(SelectionDirection::Forwards);

    //     let position = self.down_position(starting_point.clone());
    //     let preferred_x = match self.edit_location.clone() {
    //         EditLocation::Cursor(cursor) => cursor.preferred_x,
    //         EditLocation::Selection(selection) => self.preferred_x(selection.start),
    //     };

    //     self.move_to(position, preferred_x, context);
    // }

    // fn move_beginning_of_file(&mut self, _: &MoveBeginningOfFile, context: &mut ViewContext<Self>) {
    //     let position = self.beginning_of_file_position();

    //     self.move_to(position, 0, context);
    // }

    // fn move_end_of_file(&mut self, _: &MoveEndOfFile, context: &mut ViewContext<Self>) {
    //     let position = self.end_of_file_position();

    //     let block = self.content.block(position.block_index);
    //     let last_line_index = block.line_length() - 1;
    //     let preferred_x = block.length_of_line(last_line_index);

    //     self.move_to(position, preferred_x, context);
    // }

    // fn move_beginning_of_line(&mut self, _: &MoveBeginningOfLine, context: &mut ViewContext<Self>) {
    //     let starting_point = self
    //         .edit_location
    //         .starting_point(SelectionDirection::Backwards);
    //     let position = self.beginning_of_line_position(starting_point);

    //     self.move_to(position, 0, context);
    // }

    // fn move_end_of_line(&mut self, _: &MoveEndOfLine, context: &mut ViewContext<Self>) {
    //     let starting_point = self
    //         .edit_location
    //         .starting_point(SelectionDirection::Forwards);
    //     let position = self.end_of_line_position(starting_point);
    //     let preferred_x = self.preferred_x(position.clone());

    //     self.move_to(position, preferred_x, context);
    // }

    // fn move_beginning_of_word(&mut self, _: &MoveBeginningOfWord, context: &mut ViewContext<Self>) {
    //     let starting_point = match self.edit_location.clone() {
    //         EditLocation::Cursor(cursor) => cursor.position,
    //         EditLocation::Selection(selection) => selection.smallest(),
    //     };

    //     let position = self.beginning_of_word_position(starting_point);
    //     let preferred_x = self.preferred_x(position.clone());

    //     self.move_to(position, preferred_x, context);
    // }

    // fn move_end_of_word(&mut self, _: &MoveEndOfWord, context: &mut ViewContext<Self>) {
    //     let starting_point = match self.edit_location.clone() {
    //         EditLocation::Cursor(cursor) => cursor.position,
    //         EditLocation::Selection(selection) => selection.largest(),
    //     };
    //     let position = self.end_of_word_position(starting_point);
    //     let preferred_x = self.preferred_x(position.clone());

    //     self.move_to(position, preferred_x, context)
    // }

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

    // fn move_to(
    //     &mut self,
    //     position: CursorPoint,
    //     preferred_x: usize,
    //     context: &mut ViewContext<Self>,
    // ) {
    //     self.edit_location = EditLocation::Cursor(Cursor {
    //         position,
    //         preferred_x,
    //     });

    //     context.notify();
    // }

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

    // fn preferred_x(&self, position: CursorPoint) -> usize {
    //     let block = self.content.block(position.block_index);
    //     let line_index = block.line_of_offset(position.offset);

    //     return block.offset_in_line(line_index, position.offset);
    // }

    // fn left_position(&self, point: CursorPoint) -> CursorPoint {
    //     if point.block_index == 0 && point.offset == 0 {
    //         return point;
    //     }

    //     if point.offset == 0 {
    //         let new_block_index = point.block_index - 1;
    //         let block_length = self.content.block_length(new_block_index);
    //         let new_offset = if block_length == 0 {
    //             0
    //         } else {
    //             self.content.block_length(new_block_index) - 1
    //         };

    //         return CursorPoint::new(new_block_index, new_offset);
    //     }

    //     return CursorPoint::new(point.block_index, point.offset - 1);
    // }

    // fn right_position(&self, point: CursorPoint) -> CursorPoint {
    //     let block_length = self.content.block_length(point.block_index);
    //     let block_length = if block_length == 0 {
    //         0
    //     } else {
    //         self.content.block_length(point.block_index) - 1
    //     };

    //     if point.block_index == self.content.blocks().len() - 1 && point.offset == block_length {
    //         return point;
    //     }

    //     if point.offset == block_length {
    //         return CursorPoint::new(point.block_index + 1, 0);
    //     }

    //     return CursorPoint::new(point.block_index, point.offset + 1);
    // }

    // fn up_position(&self, point: CursorPoint) -> CursorPoint {
    //     let block = self.content.block(point.block_index);
    //     let line_index = block.line_of_offset(point.offset);

    //     if point.block_index == 0 && line_index == 0 {
    //         return CursorPoint::new(0, 0);
    //     }

    //     if line_index == 0 {
    //         let previous_block = self.content.block(point.block_index - 1);
    //         let previous_block_line_length = previous_block.line_length();
    //         let previous_line_start = previous_block.line_start(previous_block_line_length - 1);
    //         let previous_line_length =
    //             previous_block.length_of_line(previous_block_line_length - 1);
    //         let previous_line_end = if previous_line_start == 0 && previous_line_length == 0 {
    //             0
    //         } else {
    //             previous_line_start + previous_line_length - 1
    //         };

    //         let preferred_x = match self.edit_location.clone() {
    //             EditLocation::Cursor(cursor) => cursor.preferred_x,
    //             EditLocation::Selection(selection) => self.preferred_x(selection.start),
    //         };
    //         let preferred_offset = previous_line_start + preferred_x;

    //         let offset = std::cmp::min(previous_line_end, preferred_offset);

    //         return CursorPoint::new(point.block_index - 1, offset);
    //     }

    //     let previous_line_start = block.line_start(line_index - 1);
    //     let previous_line_length = block.length_of_line(line_index - 1);

    //     let preferred_x = match self.edit_location.clone() {
    //         EditLocation::Cursor(cursor) => cursor.preferred_x,
    //         EditLocation::Selection(selection) => self.preferred_x(selection.start),
    //     };
    //     let preferred_offset = previous_line_start + preferred_x;

    //     let offset = std::cmp::min(
    //         previous_line_start + previous_line_length - 1,
    //         preferred_offset,
    //     );

    //     return CursorPoint::new(point.block_index, offset);
    // }

    // fn down_position(&self, point: CursorPoint) -> CursorPoint {
    //     let block = self.content.block(point.block_index);
    //     let block_line_length = block.line_length();
    //     let line_index = block.line_of_offset(point.offset);

    //     // If last line in last block
    //     if point.block_index == self.content.blocks().len() - 1
    //         && line_index == block_line_length - 1
    //     {
    //         return CursorPoint::new(point.block_index, block.length());
    //     }

    //     // If last line in any block
    //     if line_index == block_line_length - 1 {
    //         let next_block_index = point.block_index + 1;
    //         let next_block = self.content.block(next_block_index);

    //         let first_line_length = next_block.length_of_line(0);
    //         let first_line_length = if first_line_length == 0 {
    //             0
    //         } else {
    //             if next_block.is_soft_wrapped_line(0) {
    //                 first_line_length - 1
    //             } else {
    //                 first_line_length
    //             }
    //         };

    //         let preferred_x = match self.edit_location.clone() {
    //             EditLocation::Cursor(cursor) => cursor.preferred_x,
    //             EditLocation::Selection(selection) => self.preferred_x(selection.start),
    //         };

    //         let offset = std::cmp::min(first_line_length, preferred_x);

    //         return CursorPoint::new(next_block_index, offset);
    //     }

    //     let next_line_start = block.line_start(line_index + 1);
    //     let next_line_length = block.length_of_line(line_index + 1);
    //     let is_soft_wrapped_line = block.is_soft_wrapped_line(line_index + 1);
    //     let modifier_value = match is_soft_wrapped_line {
    //         true => 1,
    //         false => 0,
    //     };

    //     let preferred_x = match self.edit_location.clone() {
    //         EditLocation::Cursor(cursor) => cursor.preferred_x,
    //         EditLocation::Selection(selection) => self.preferred_x(selection.start),
    //     };
    //     let preferred_offset = next_line_start + preferred_x;

    //     let offset = std::cmp::min(
    //         next_line_start + next_line_length - modifier_value,
    //         preferred_offset,
    //     );

    //     return CursorPoint::new(point.block_index, offset);
    // }

    // fn beginning_of_file_position(&self) -> CursorPoint {
    //     return CursorPoint::new(0, 0);
    // }

    // fn end_of_file_position(&self) -> CursorPoint {
    //     let block_index = self.content.blocks().len() - 1;

    //     let block = self.content.block(block_index);
    //     let offset = block.length() - 1;

    //     return CursorPoint::new(block_index, offset);
    // }

    // fn beginning_of_line_position(&self, point: CursorPoint) -> CursorPoint {
    //     let block = self.content.block(point.block_index);
    //     let current_line_index = block.line_of_offset(point.offset);
    //     let line_start = block.line_start(current_line_index);

    //     return CursorPoint::new(point.block_index, line_start);
    // }

    // fn end_of_line_position(&self, point: CursorPoint) -> CursorPoint {
    //     let block = self.content.block(point.block_index);
    //     let current_line_index = block.line_of_offset(point.offset);
    //     let line_start = block.line_start(current_line_index);
    //     let line_length = block.length_of_line(current_line_index);

    //     // If line is empty (just a newline), stay at line_start, otherwise go to last character
    //     let new_offset = if line_length == 0 {
    //         line_start
    //     } else {
    //         line_start + line_length - 1
    //     };

    //     return CursorPoint::new(point.block_index, new_offset);
    // }

    // fn beginning_of_word_position(&self, point: CursorPoint) -> CursorPoint {
    //     let mut potential_position: Option<(usize, usize)> = None;
    //     let mut current_block_index = point.block_index;
    //     let mut current_offset = point.offset;

    //     loop {
    //         let block = self.content.block(current_block_index);

    //         let position = block.previous_word_boundary(current_offset);

    //         if let Some(offset) = position {
    //             potential_position = Some((current_block_index, offset));
    //             break;
    //         }

    //         if current_block_index == 0 {
    //             break;
    //         }

    //         let previous_block_index = current_block_index - 1;
    //         let previous_block = self.content.block(previous_block_index);
    //         current_offset = previous_block.length();
    //         current_block_index = previous_block_index;
    //     }

    //     let (block_index, offset) = match potential_position {
    //         Some(position) => position,
    //         None => (0, 0),
    //     };

    //     return CursorPoint::new(block_index, offset);
    // }

    // fn end_of_word_position(&self, point: CursorPoint) -> CursorPoint {
    //     let mut potential_position: Option<(usize, usize)> = None;
    //     let mut current_block_index = point.block_index;
    //     let mut current_offset = point.offset;

    //     loop {
    //         let block = self.content.block(current_block_index);

    //         let position = block.next_word_boundary(current_offset);

    //         if let Some(offset) = position {
    //             potential_position = Some((current_block_index, offset));
    //             break;
    //         }

    //         if current_block_index == self.content.blocks().len() - 1 {
    //             break;
    //         }

    //         current_offset = 0;
    //         current_block_index = current_block_index + 1;
    //     }

    //     let (block_index, offset) = match potential_position {
    //         Some(position) => position,
    //         None => {
    //             let block_index = self.content.blocks().len() - 1;
    //             let block = self.content.block(block_index);
    //             let length = if block.length() == 0 {
    //                 0
    //             } else {
    //                 block.length() - 1
    //             };

    //             (block_index, length)
    //         }
    //     };

    //     return CursorPoint::new(block_index, offset);
    // }
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
            // .on_action(context.listener(Self::move_left))
            // .on_action(context.listener(Self::move_right))
            // .on_action(context.listener(Self::move_up))
            // .on_action(context.listener(Self::move_down))
            // .on_action(context.listener(Self::move_beginning_of_file))
            // .on_action(context.listener(Self::move_end_of_file))
            // .on_action(context.listener(Self::move_beginning_of_line))
            // .on_action(context.listener(Self::move_end_of_line))
            // .on_action(context.listener(Self::move_beginning_of_word))
            // .on_action(context.listener(Self::move_end_of_word))
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
enum LineType {
    HeadlineStart(usize),
    HeadlineNotStart,
    Regular,
}

#[derive(Debug, Clone)]
struct RenderedLine {
    shaped_line: ShapedLine,
    kind: LineType,
}

impl RenderedLine {
    pub fn new(kind: LineType, shaped_line: ShapedLine) -> RenderedLine {
        return RenderedLine { kind, shaped_line };
    }
}

struct PrepaintState {
    lines: Vec<RenderedLine>,
    // blocks: Vec<RenderedBlock>,
    // edit_location_rectangles: Option<Vec<PaintQuad>>,
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

        // let lines = context.text_system().shape_line(content.to_string(), font_size, runs)
        let mut lines: Vec<RenderedLine> = vec![];

        let text = content.to_string();
        let raw_lines: Vec<_> = text.lines().map(|s| s.to_string()).collect();
        for line in &raw_lines {
            let is_headline = line.starts_with('#');

            if is_headline {
                let runs = vec![TextRun {
                    len: line.len(),
                    font: Font {
                        weight: FontWeight::EXTRA_BOLD,
                        ..style.font()
                    },
                    color: Hsla::from(rgb(COLOR_GRAY_800)),
                    background_color: None,
                    underline: None,
                    strikethrough: None,
                }];
                let shaped_line = context
                    .text_system()
                    .shape_line(SharedString::from(line), font_size, &runs)
                    .unwrap();

                let level = line
                    .chars()
                    .take_while(|&character| character == '#')
                    .count();

                lines.push(RenderedLine::new(
                    LineType::HeadlineStart(level),
                    shaped_line,
                ));
            } else {
                let runs = vec![TextRun {
                    len: line.len(),
                    font: style.font(),
                    color: Hsla::from(rgb(COLOR_GRAY_700)),
                    background_color: None,
                    underline: None,
                    strikethrough: None,
                }];
                let shaped_line = context
                    .text_system()
                    .shape_line(SharedString::from(line), font_size, &runs)
                    .unwrap();

                lines.push(RenderedLine::new(LineType::Regular, shaped_line));
            }
        }
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
            // blocks: rendered_blocks,
            // edit_location_rectangles: Some(edit_location_rectangles),
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
        let lines = prepaint.lines.clone();

        for (index, line) in lines.iter().enumerate() {
            let offset = match line.kind {
                LineType::HeadlineStart(level) => {
                    EDITOR_HORIZONTAL_MARGIN - px(level as f32 + 1 as f32) * CHARACTER_WIDTH
                }
                LineType::HeadlineNotStart => EDITOR_HORIZONTAL_MARGIN,
                LineType::Regular => EDITOR_HORIZONTAL_MARGIN,
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

        // if focus_handle.is_focused(context) {
        //     if let Some(edit_location_rectanlges) = prepaint.edit_location_rectangles.take() {
        //         for rectangle in edit_location_rectanlges {
        //             context.paint_quad(rectangle);
        //         }
        //     }
        // }

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
