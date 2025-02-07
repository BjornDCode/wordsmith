use std::ops::Index;

use gpui::{
    div, fill, point, prelude::*, px, rgb, size, AppContext, Bounds, FocusHandle, FocusableView,
    PaintQuad, Pixels, Point, Style, View, ViewContext,
};

use crate::content::{Block, Content, Size};
use crate::content::{Render, RenderedBlock};
use crate::{MoveLeft, MoveRight, MoveUp, COLOR_BLUE_DARK, COLOR_GRAY_800, COLOR_PINK};

const CHARACTER_WIDTH: Pixels = px(10.24);
pub const CHARACTER_COUNT_PER_LINE: usize = 50;
pub const CONTAINER_WIDTH: Pixels = px(512.); // CHARACTER_WIDTH * CHARCTER_COUNT_PER_LINE

pub struct Editor {
    focus_handle: FocusHandle,
    content: Content,
    cursor_position: CursorPosition,
}

struct CursorPosition {
    block_index: usize,
    offset: usize,
}

impl Editor {
    pub fn new(focus_handle: FocusHandle) -> Editor {
        return Editor {
            focus_handle,
            content: Content::new("## This is a headline\n\nThis is a paragraph with some bold text, some italic text and some mixed text. This is a paragraph with some bold text, some italic text and some mixed text.\n\nThis is a paragraph with some bold text, some italic text and some mixed text.\n\n### Another headline\n\nYo, some more text\n\n## Headline".into()),
            cursor_position: CursorPosition { offset: 135, block_index: 2 }
        };
    }

    fn move_left(&mut self, _: &MoveLeft, context: &mut ViewContext<Self>) {
        if self.cursor_position.offset == 0 {
            if self.cursor_position.block_index > 0 {
                let new_y = self.cursor_position.block_index - 1;

                self.cursor_position.block_index = new_y;
                self.cursor_position.offset = self.content.block_length(new_y) - 1;
            }
        } else {
            self.cursor_position.offset -= 1;
        }

        context.notify();
    }

    fn move_right(&mut self, _: &MoveRight, context: &mut ViewContext<Self>) {
        let block_length = self.content.block_length(self.cursor_position.block_index) - 1;

        if self.cursor_position.offset == block_length {
            let new_y = self.cursor_position.block_index + 1;

            if new_y < self.content.blocks().len() {
                self.cursor_position.block_index += 1;
                self.cursor_position.offset = 0;
            }
        } else {
            self.cursor_position.offset += 1;
        }

        context.notify();
    }

    fn move_up(&mut self, _: &MoveUp, context: &mut ViewContext<Self>) {
        let current_block = self.content.block(self.cursor_position.block_index);
        let line_index_in_block = current_block.line_of_offset(self.cursor_position.offset);

        if line_index_in_block == 0 {
            if self.cursor_position.block_index > 0 {
                let offset_in_current_line =
                    current_block.offset_in_line(line_index_in_block, self.cursor_position.offset);
                let new_block_index = self.cursor_position.block_index - 1;
                self.cursor_position.block_index = new_block_index;

                let previous_block = self.content.block(new_block_index);
                let previous_block_line_length = previous_block.line_length();
                let length_of_last_line_in_prevous_block =
                    previous_block.length_of_line(previous_block_line_length - 1);

                let offset =
                    std::cmp::min(length_of_last_line_in_prevous_block, offset_in_current_line);

                self.cursor_position.offset = offset;
            } else {
                self.cursor_position.offset = 0;
            }
        } else {
            let offset_in_current_line =
                current_block.offset_in_line(line_index_in_block, self.cursor_position.offset);

            let previous_line_length = current_block.length_of_line(line_index_in_block - 1);
            let previous_line_start = current_block.line_start(line_index_in_block - 1);
            let offset = std::cmp::min(
                previous_line_start + previous_line_length - 1,
                previous_line_start + offset_in_current_line,
            );

            self.cursor_position.offset = offset;
        }

        context.notify()
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
    cursor: Option<PaintQuad>,
    headline_rectangles: Vec<PaintQuad>,
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

        let mut headline_rectangles = vec![];

        for block in &blocks {
            if let Block::Headline(headline) = block {
                let width = px(16. * headline.level() as f32);

                let rectangle = fill(
                    Bounds::new(
                        point(
                            bounds.origin.x - width - px(16.),
                            bounds.origin.y + (context.line_height() * block.line_index()) + px(4.),
                        ),
                        size(width, px(16.)),
                    ),
                    rgb(COLOR_GRAY_800),
                );

                headline_rectangles.push(rectangle);
            }
        }

        let rendered_blocks: Vec<RenderedBlock> = blocks
            .into_iter()
            .map(|mut block| block.render(context.text_system(), style.font(), font_size))
            .collect();

        let cursor_position = content.cursor_position(
            input.cursor_position.block_index,
            input.cursor_position.offset,
        );

        let cursor = fill(
            Bounds::new(
                point(
                    bounds.left() + px(cursor_position.x as f32) * CHARACTER_WIDTH - px(1.),
                    bounds.top() + context.line_height() * px(cursor_position.y as f32) + px(4.),
                ),
                size(px(2.), px(16.)),
            ),
            rgb(COLOR_BLUE_DARK),
        );

        PrepaintState {
            blocks: rendered_blocks,
            cursor: Some(cursor),
            headline_rectangles,
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
        let headline_rectangles = prepaint.headline_rectangles.clone();

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

        for rectangle in headline_rectangles {
            context.paint_quad(rectangle);
        }

        if focus_handle.is_focused(context) {
            if let Some(cursor) = prepaint.cursor.take() {
                context.paint_quad(cursor);
            }
        }
    }
}
