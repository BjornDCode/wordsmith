use std::any::Any;
use std::cmp::Ordering;
use std::ops::Index;

use gpui::{
    div, fill, point, prelude::*, px, rgb, size, AppContext, Bounds, FocusHandle, FocusableView,
    Font, FontWeight, Hsla, PaintQuad, Pixels, Point, ShapedLine, Style, TextRun, View,
    ViewContext,
};

use crate::content::{Block, Content, Size};
use crate::content::{Render, RenderedBlock};
use crate::{
    MoveBeginningOfFile, MoveBeginningOfLine, MoveBeginningOfWord, MoveDown, MoveEndOfFile,
    MoveEndOfLine, MoveEndOfWord, MoveLeft, MoveRight, MoveUp, COLOR_BLUE_DARK, COLOR_BLUE_LIGHT,
    COLOR_BLUE_MEDIUM, COLOR_GRAY_800, COLOR_PINK,
};

const CHARACTER_WIDTH: Pixels = px(10.24);
pub const CHARACTER_COUNT_PER_LINE: usize = 50;
pub const CONTAINER_WIDTH: Pixels = px(512.); // CHARACTER_WIDTH * CHARCTER_COUNT_PER_LINE

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

#[derive(Debug, Clone)]
pub struct CursorPoint {
    pub block_index: usize,
    pub offset: usize,
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
            SelectionDirection::Left
        } else {
            SelectionDirection::Right
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

#[derive(Debug, Clone)]
enum SelectionDirection {
    Left,
    Right,
}

impl Editor {
    pub fn new(focus_handle: FocusHandle) -> Editor {
        let edit_location = EditLocation::Selection(Selection {
            end: CursorPoint {
                offset: 12,
                block_index: 0,
            },
            start: CursorPoint {
                offset: 110,
                block_index: 2,
            },
        });

        return Editor {
            focus_handle,
            content: Content::new("## This is a headline\n\nDolor elend vitae porta iaculis etiam commodo. Mus erat lacus penatibus congue ultricies. Elementum tristique sociosqu curae etiam consequat et arcu placerat est.\n\nHabitant primis praesent malesuada lorem parturient lobortis metus. Pulvinar ultrices ligula id ac quisque curae, leo est.\n\n### Another headline\n\nYo, some more text\n\n## Headline".into()),
            edit_location
        };
    }

    fn move_left(&mut self, _: &MoveLeft, context: &mut ViewContext<Self>) {
        let point = match self.edit_location.clone() {
            EditLocation::Cursor(cursor) => cursor.position,
            EditLocation::Selection(selection) => match selection.direction() {
                SelectionDirection::Left => selection.end,
                SelectionDirection::Right => selection.start,
            },
        };

        if point.offset == 0 {
            if point.block_index > 0 {
                let new_block_index = point.block_index - 1;
                let block_length = self.content.block_length(new_block_index);
                let new_offset = if block_length == 0 {
                    0
                } else {
                    self.content.block_length(new_block_index) - 1
                };
                let new_block = self.content.block(new_block_index);
                let last_line_index = new_block.line_length() - 1;
                let last_line_length = new_block.length_of_line(last_line_index);

                self.edit_location = EditLocation::Cursor(Cursor {
                    position: CursorPoint {
                        block_index: new_block_index,
                        offset: new_offset,
                    },
                    preferred_x: last_line_length,
                });
            }
        } else {
            let new_offset = point.offset - 1;
            let block = self.content.block(point.block_index);
            let line_in_block = block.line_of_offset(new_offset);
            let new_preferred_x = block.offset_in_line(line_in_block, new_offset);

            self.edit_location = EditLocation::Cursor(Cursor {
                position: CursorPoint {
                    block_index: point.block_index,
                    offset: new_offset,
                },
                preferred_x: new_preferred_x,
            });
        }

        context.notify();
    }

    fn move_right(&mut self, _: &MoveRight, context: &mut ViewContext<Self>) {
        let point = match self.edit_location.clone() {
            EditLocation::Cursor(cursor) => cursor.position,
            EditLocation::Selection(selection) => match selection.direction() {
                SelectionDirection::Left => selection.start,
                SelectionDirection::Right => selection.end,
            },
        };
        let block_length = self.content.block_length(point.block_index);

        let block_length = if block_length == 0 {
            0
        } else {
            self.content.block_length(point.block_index) - 1
        };

        if point.offset == block_length {
            let new_block_index = point.block_index + 1;

            if new_block_index < self.content.blocks().len() {
                self.edit_location = EditLocation::Cursor(Cursor {
                    position: CursorPoint {
                        block_index: new_block_index,
                        offset: 0,
                    },
                    preferred_x: 0,
                });
            }
        } else {
            let new_offset = point.offset + 1;
            let block = self.content.block(point.block_index);
            let line_in_block = block.line_of_offset(new_offset);
            let new_preferred_x = block.offset_in_line(line_in_block, new_offset);

            self.edit_location = EditLocation::Cursor(Cursor {
                position: CursorPoint {
                    block_index: point.block_index,
                    offset: new_offset,
                },
                preferred_x: new_preferred_x,
            });
        }

        context.notify();
    }

    fn move_up(&mut self, _: &MoveUp, context: &mut ViewContext<Self>) {
        let point = match self.edit_location.clone() {
            EditLocation::Cursor(cursor) => cursor.position,
            EditLocation::Selection(selection) => match selection.direction() {
                SelectionDirection::Left => selection.end,
                SelectionDirection::Right => selection.start,
            },
        };
        let current_block = self.content.block(point.block_index);
        let line_index_in_block = current_block.line_of_offset(point.offset);

        if line_index_in_block == 0 {
            if point.block_index > 0 {
                let new_block_index = point.block_index - 1;

                let previous_block = self.content.block(new_block_index);
                let previous_block_line_length = previous_block.line_length();
                let start_of_last_line_in_previous_block =
                    previous_block.line_start(previous_block_line_length - 1);

                let last_line_length =
                    previous_block.length_of_line(previous_block_line_length - 1);

                let old_preferred_x = match self.edit_location.clone() {
                    EditLocation::Cursor(cursor) => cursor.preferred_x,
                    EditLocation::Selection(selection) => match selection.direction() {
                        SelectionDirection::Left => {
                            let block = self.content.block(point.block_index);
                            let line_index = block.line_of_offset(selection.end.offset);
                            let offset_in_line =
                                block.offset_in_line(line_index, selection.end.offset);

                            offset_in_line
                        }
                        SelectionDirection::Right => {
                            let block = self.content.block(point.block_index);
                            let line_index = block.line_of_offset(selection.start.offset);
                            let offset_in_line =
                                block.offset_in_line(line_index, selection.start.offset);

                            offset_in_line
                        }
                    },
                };
                let preferred_offset = start_of_last_line_in_previous_block + old_preferred_x;

                let offset = std::cmp::min(
                    start_of_last_line_in_previous_block + last_line_length,
                    preferred_offset,
                );

                self.edit_location = EditLocation::Cursor(Cursor {
                    position: CursorPoint {
                        block_index: new_block_index,
                        offset,
                    },
                    preferred_x: old_preferred_x,
                });
            } else {
                self.edit_location = EditLocation::Cursor(Cursor {
                    position: CursorPoint {
                        block_index: 0,
                        offset: 0,
                    },
                    preferred_x: 0,
                });
            }
        } else {
            let previous_line_length = current_block.length_of_line(line_index_in_block - 1);
            let previous_line_start = current_block.line_start(line_index_in_block - 1);

            let old_preferred_x = match self.edit_location.clone() {
                EditLocation::Cursor(cursor) => cursor.preferred_x,
                EditLocation::Selection(selection) => match selection.direction() {
                    SelectionDirection::Left => {
                        let block = self.content.block(point.block_index);
                        let line_index = block.line_of_offset(selection.end.offset);
                        let offset_in_line = block.offset_in_line(line_index, selection.end.offset);

                        offset_in_line
                    }
                    SelectionDirection::Right => {
                        let block = self.content.block(point.block_index);
                        let line_index = block.line_of_offset(selection.start.offset);
                        let offset_in_line =
                            block.offset_in_line(line_index, selection.start.offset);

                        offset_in_line
                    }
                },
            };
            let preferred_offset = previous_line_start + old_preferred_x;

            let offset = std::cmp::min(
                previous_line_start + previous_line_length - 1,
                preferred_offset,
            );

            self.edit_location = EditLocation::Cursor(Cursor {
                position: CursorPoint {
                    block_index: point.block_index,
                    offset,
                },
                preferred_x: old_preferred_x,
            });
        }

        context.notify()
    }

    fn move_down(&mut self, _: &MoveDown, context: &mut ViewContext<Self>) {
        let point = match self.edit_location.clone() {
            EditLocation::Cursor(cursor) => cursor.position,
            EditLocation::Selection(selection) => match selection.direction() {
                SelectionDirection::Left => selection.start,
                SelectionDirection::Right => selection.end,
            },
        };
        let block = self.content.block(point.block_index);
        let block_line_length = block.line_length();
        let line_index_in_block = block.line_of_offset(point.offset);

        if line_index_in_block == block_line_length - 1 {
            if point.block_index < self.content.blocks().len() - 1 {
                let next_block_index = point.block_index + 1;
                let next_block = self.content.block(next_block_index);

                let first_line_length = next_block.length_of_line(0);
                let first_line_length = if first_line_length == 0 {
                    0
                } else {
                    if next_block.is_soft_wrapped_line(0) {
                        first_line_length - 1
                    } else {
                        first_line_length
                    }
                };

                let preferred_offset = match self.edit_location.clone() {
                    EditLocation::Cursor(cursor) => cursor.preferred_x,
                    EditLocation::Selection(selection) => match selection.direction() {
                        SelectionDirection::Left => {
                            let block = self.content.block(point.block_index);
                            let line_index = block.line_of_offset(selection.end.offset);
                            let offset_in_line =
                                block.offset_in_line(line_index, selection.end.offset);

                            offset_in_line
                        }
                        SelectionDirection::Right => {
                            let block = self.content.block(point.block_index);
                            let line_index = block.line_of_offset(selection.start.offset);
                            let offset_in_line =
                                block.offset_in_line(line_index, selection.start.offset);

                            offset_in_line
                        }
                    },
                };

                let offset = std::cmp::min(first_line_length, preferred_offset);

                self.edit_location = EditLocation::Cursor(Cursor {
                    position: CursorPoint {
                        block_index: next_block_index,
                        offset,
                    },
                    preferred_x: preferred_offset,
                });
            } else {
                let last_last_index = block.line_length() - 1;
                let last_line_length = block.length_of_line(last_last_index);
                self.edit_location = EditLocation::Cursor(Cursor {
                    position: CursorPoint {
                        block_index: point.block_index,
                        offset: block.length() - 1,
                    },
                    preferred_x: last_line_length,
                });
            }
        } else {
            let next_line_start = block.line_start(line_index_in_block + 1);
            let next_line_length = block.length_of_line(line_index_in_block + 1);
            let is_soft_wrapped_line = block.is_soft_wrapped_line(line_index_in_block + 1);
            let modifier_value = match is_soft_wrapped_line {
                true => 1,
                false => 0,
            };

            let old_preferred_x = match self.edit_location.clone() {
                EditLocation::Cursor(cursor) => cursor.preferred_x,
                EditLocation::Selection(_selection) => {
                    let block = self.content.block(point.block_index);
                    let line_index = block.line_of_offset(point.offset);
                    let offset_in_line = block.offset_in_line(line_index, point.offset);

                    offset_in_line
                }
            };
            let preferred_offset = next_line_start + old_preferred_x;

            let offset = std::cmp::min(
                next_line_start + next_line_length - modifier_value,
                preferred_offset,
            );

            self.edit_location = EditLocation::Cursor(Cursor {
                position: CursorPoint {
                    block_index: point.block_index,
                    offset,
                },
                preferred_x: old_preferred_x,
            });
        }

        context.notify();
    }

    fn move_beginning_of_file(&mut self, _: &MoveBeginningOfFile, context: &mut ViewContext<Self>) {
        self.edit_location = EditLocation::Cursor(Cursor {
            position: CursorPoint {
                block_index: 0,
                offset: 0,
            },
            preferred_x: 0,
        });

        context.notify();
    }

    fn move_end_of_file(&mut self, _: &MoveEndOfFile, context: &mut ViewContext<Self>) {
        let last_block_index = self.content.blocks().len() - 1;
        let block = self.content.block(last_block_index);
        let last_line_index = block.line_length() - 1;
        let last_line_length = block.length_of_line(last_line_index);

        self.edit_location = EditLocation::Cursor(Cursor {
            position: CursorPoint {
                block_index: last_block_index,
                offset: block.length() - 1,
            },
            preferred_x: last_line_length,
        });

        context.notify();
    }

    fn move_beginning_of_line(&mut self, _: &MoveBeginningOfLine, context: &mut ViewContext<Self>) {
        let point = match self.edit_location.clone() {
            EditLocation::Cursor(cursor) => cursor.position,
            EditLocation::Selection(selection) => match selection.direction() {
                SelectionDirection::Left => selection.end,
                SelectionDirection::Right => selection.start,
            },
        };

        let block = self.content.block(point.block_index);
        let current_line_index = block.line_of_offset(point.offset);
        let line_start = block.line_start(current_line_index);

        self.edit_location = EditLocation::Cursor(Cursor {
            position: CursorPoint {
                block_index: point.block_index,
                offset: line_start,
            },
            preferred_x: 0,
        });

        context.notify();
    }

    fn move_end_of_line(&mut self, _: &MoveEndOfLine, context: &mut ViewContext<Self>) {
        let point = match self.edit_location.clone() {
            EditLocation::Cursor(cursor) => cursor.position,
            EditLocation::Selection(selection) => match selection.direction() {
                SelectionDirection::Left => selection.start,
                SelectionDirection::Right => selection.end,
            },
        };

        let block = self.content.block(point.block_index);
        let current_line_index = block.line_of_offset(point.offset);
        let line_start = block.line_start(current_line_index);
        let line_length = block.length_of_line(current_line_index);
        let new_preferred_x = if line_length == 0 { 0 } else { line_length - 1 };

        // If line is empty (just a newline), stay at line_start, otherwise go to last character
        let new_offset = if line_length == 0 {
            line_start
        } else {
            line_start + line_length - 1
        };

        self.edit_location = EditLocation::Cursor(Cursor {
            position: CursorPoint {
                block_index: point.block_index,
                offset: new_offset,
            },
            preferred_x: new_preferred_x,
        });

        context.notify();
    }

    fn move_beginning_of_word(&mut self, _: &MoveBeginningOfWord, context: &mut ViewContext<Self>) {
        let point = match self.edit_location.clone() {
            EditLocation::Cursor(cursor) => cursor.position,
            EditLocation::Selection(selection) => selection.smallest(),
        };

        let mut potential_position: Option<(usize, usize)> = None;
        let mut current_block_index = point.block_index;
        let mut current_offset = point.offset;

        loop {
            let block = self.content.block(current_block_index);

            let position = block.previous_word_boundary(current_offset);

            if let Some(offset) = position {
                potential_position = Some((current_block_index, offset));
                break;
            }

            if current_block_index == 0 {
                break;
            }

            let previous_block_index = current_block_index - 1;
            let previous_block = self.content.block(previous_block_index);
            current_offset = previous_block.length();
            current_block_index = previous_block_index;
        }

        let (block_index, offset) = match potential_position {
            Some(position) => position,
            None => (0, 0),
        };

        let block = self.content.block(block_index);
        let line_of_offset = block.line_of_offset(offset);
        let line_length = block.length_of_line(line_of_offset);
        let preferred_x = if line_length == 0 { 0 } else { line_length - 1 };

        self.edit_location = EditLocation::Cursor(Cursor {
            position: CursorPoint {
                block_index,
                offset,
            },
            preferred_x,
        });

        context.notify();
    }

    fn move_end_of_word(&mut self, _: &MoveEndOfWord, context: &mut ViewContext<Self>) {
        let point = match self.edit_location.clone() {
            EditLocation::Cursor(cursor) => cursor.position,
            EditLocation::Selection(selection) => selection.largest(),
        };

        let mut potential_position: Option<(usize, usize)> = None;
        let mut current_block_index = point.block_index;
        let mut current_offset = point.offset;

        loop {
            let block = self.content.block(current_block_index);

            let position = block.next_word_boundary(current_offset);

            if let Some(offset) = position {
                potential_position = Some((current_block_index, offset));
                break;
            }

            if current_block_index == self.content.blocks().len() - 1 {
                break;
            }

            current_offset = 0;
            current_block_index = current_block_index + 1;
        }

        let (block_index, offset) = match potential_position {
            Some(position) => position,
            None => {
                let block_index = self.content.blocks().len() - 1;
                let block = self.content.block(block_index);
                let length = if block.length() == 0 {
                    0
                } else {
                    block.length() - 1
                };

                (block_index, length)
            }
        };

        let block = self.content.block(block_index);
        let line_of_offset = block.line_of_offset(offset);
        let line_length = block.length_of_line(line_of_offset);
        let preferred_x = if line_length == 0 { 0 } else { line_length - 1 };

        self.edit_location = EditLocation::Cursor(Cursor {
            position: CursorPoint {
                block_index,
                offset,
            },
            preferred_x,
        });

        context.notify();
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
            .pt_8()
            .group("editor-container")
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

struct PrepaintState {
    blocks: Vec<RenderedBlock>,
    edit_location_rectangles: Option<Vec<PaintQuad>>,
    headline_markers: Vec<RenderedHeadlineMarker>,
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

        let blocks = content.blocks();

        let mut headline_markers = vec![];

        for block in &blocks {
            if let Block::Headline(headline) = block {
                let width = px(16. * headline.level() as f32);
                let content = "#".repeat(headline.level()) + " ";
                let runs = vec![TextRun {
                    len: content.len(),
                    font: Font {
                        weight: FontWeight::EXTRA_BOLD,
                        ..style.font()
                    },
                    color: Hsla::from(rgb(COLOR_GRAY_800)),
                    background_color: None,
                    underline: None,
                    strikethrough: None,
                }];
                let shaped_text = context
                    .text_system()
                    .shape_line(content.into(), font_size, &runs)
                    .unwrap();
                let origin = point(
                    bounds.origin.x - width,
                    bounds.origin.y + (context.line_height() * block.line_index()),
                );

                headline_markers.push(RenderedHeadlineMarker {
                    shaped_text,
                    origin,
                });
            }
        }

        let rendered_blocks: Vec<RenderedBlock> = blocks
            .into_iter()
            .map(|mut block| block.render(context.text_system(), style.font(), font_size))
            .collect();

        let edit_location_rectangles = match input.edit_location.clone() {
            EditLocation::Cursor(caret) => {
                let position = content.cursor_position(caret.position);

                let rectangles = vec![fill(
                    Bounds::new(
                        point(
                            bounds.left() + px(position.x as f32) * CHARACTER_WIDTH - px(1.),
                            bounds.top() + context.line_height() * px(position.y as f32) + px(4.),
                        ),
                        size(px(2.), px(16.)),
                    ),
                    rgb(COLOR_BLUE_DARK),
                )];

                rectangles
            }
            EditLocation::Selection(selection) => {
                let mut rectangles = vec![];
                let smallest_point = std::cmp::min(selection.start.clone(), selection.end.clone());
                let largest_point = std::cmp::max(selection.start.clone(), selection.end.clone());
                let block_indexes = smallest_point.block_index..largest_point.block_index + 1;

                for block_index in block_indexes.clone() {
                    let block_start_line_index = content.block_start(block_index);
                    let block = content.block(block_index);
                    let min = if block_index == block_indexes.start {
                        smallest_point.offset
                    } else {
                        0
                    };
                    let max = if block_index == block_indexes.end - 1 {
                        largest_point.offset
                    } else {
                        block.length()
                    };
                    let line_range = block.line_range(min, max);

                    for line_index in line_range.clone() {
                        let start = if line_index == line_range.start {
                            block.offset_in_line(line_index, min)
                        } else {
                            0
                        };
                        let end = if line_index == line_range.end - 1 {
                            let offset = if block_index == 0 {
                                block.offset_in_line(line_index, min)
                            } else {
                                block.offset_in_line(line_index, max)
                            };

                            offset
                        } else {
                            CHARACTER_COUNT_PER_LINE - start
                        };

                        let left = bounds.left() + px(start as f32) * CHARACTER_WIDTH - px(1.);
                        let top = bounds.top()
                            + px(block_start_line_index as f32) * context.line_height()
                            + px(line_index as f32) * context.line_height();
                        let width = px(end as f32) * CHARACTER_WIDTH + px(2.);

                        let bounds =
                            Bounds::new(point(left, top), size(width, context.line_height()));
                        let rectangle = fill(bounds, rgb(COLOR_BLUE_MEDIUM));

                        rectangles.push(rectangle);
                    }
                }

                rectangles
            }
        };

        PrepaintState {
            blocks: rendered_blocks,
            edit_location_rectangles: Some(edit_location_rectangles),
            headline_markers,
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
        let blocks = prepaint.blocks.clone().into_iter();
        let headline_markers = prepaint.headline_markers.clone();

        if focus_handle.is_focused(context) {
            if let Some(edit_location_rectanlges) = prepaint.edit_location_rectangles.take() {
                for rectangle in edit_location_rectanlges {
                    context.paint_quad(rectangle);
                }
            }
        }

        for block in blocks {
            // The reason we are not just looping over lines directly is that there seem to be a rogue newline at the end
            // So this is a hacky way to avoid that
            // Should probably fix that issue properly at some point

            let mut line_count = 0;

            for index in 0..block.line_length {
                let line = &block.lines.index(index);

                let point = Point::new(
                    bounds.origin.x,
                    bounds.origin.y
                        + (context.line_height() * px(block.line_index as f32 + line_count as f32)),
                );

                line.paint(point, context.line_height(), context).unwrap();

                line_count += 1;
            }
        }

        for marker in headline_markers {
            marker.render(context);
        }
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
