use std::ops::Index;

use gpui::{
    div, fill, point, prelude::*, px, rgb, size, AppContext, Bounds, FocusHandle, FocusableView,
    PaintQuad, Pixels, Point, Style, View, ViewContext, WrappedLine,
};

use crate::{
    text::{Block, Render, Size, Text},
    MoveLeft, MoveRight, COLOR_BLUE_DARK, COLOR_GRAY_800, COLOR_PINK,
};

pub struct Editor {
    focus_handle: FocusHandle,
    content: Text,
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
            content: Text::new("## This is a headline\n\nThis is a paragraph with some bold text, some italic text and some mixed text. This is a paragraph with some bold text, some italic text and some mixed text.\n\nThis is a paragraph with some bold text, some italic text and some mixed text.\n\n### Another headline\n\nYo, some more text\n\n## Headline"),
            cursor_position: CursorPosition { offset: 42, block_index: 4 }
        };
    }

    fn move_left(&mut self, _: &MoveLeft, context: &mut ViewContext<Self>) {
        if self.cursor_position.offset == 0 {
            if self.cursor_position.block_index > 0 {
                let new_y = self.cursor_position.block_index - 1;

                self.cursor_position.block_index = new_y;
                self.cursor_position.offset = self.content.get_block_length(new_y);
            }
        } else {
            self.cursor_position.offset -= 1;
        }

        context.notify();
    }

    fn move_right(&mut self, _: &MoveRight, context: &mut ViewContext<Self>) {
        let block_length = self
            .content
            .get_block_length(self.cursor_position.block_index);

        if self.cursor_position.offset == block_length {
            if self.cursor_position.block_index + 1 < self.content.blocks().len() {
                self.cursor_position.block_index += 1;
                self.cursor_position.offset = 0;
            }
        } else {
            self.cursor_position.offset += 1;
        }

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
            .pt_8()
            .group("editor-container")
            .child(
                div()
                    .bg(rgb(COLOR_PINK))
                    .w(px(480.))
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
    blocks: Option<Vec<WrappedLine>>,
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

        let mut blocks = content.blocks();

        let mut headline_rectangles = vec![];
        let mut line_count = 0;

        for block in &mut blocks {
            if let Block::Headline(headline) = block {
                let width = px(16. * headline.level.length() as f32);

                let rect = fill(
                    Bounds::new(
                        point(
                            bounds.origin.x - width - px(16.),
                            bounds.origin.y + (context.line_height() * line_count) + px(4.),
                        ),
                        size(width, px(16.)),
                    ),
                    rgb(COLOR_GRAY_800),
                );

                headline_rectangles.push(rect);
            }

            line_count += block.line_length(context.text_system(), style.font(), font_size);
        }

        let content = blocks
            .iter_mut()
            .flat_map(|block| block.render(context.text_system(), style.font(), font_size))
            .collect();

        let shaped_blocks = blocks
            .get_mut(input.cursor_position.block_index)
            .unwrap()
            .render(context.text_system(), style.font(), font_size);
        let shaped_block = shaped_blocks.index(0);
        // let cursor_position = shaped_block
        //     .position_for_index(input.cursor_position.offset, context.line_height())
        //     .unwrap();
        let cursor_position = position_for_index(
            shaped_block,
            input.cursor_position.offset,
            context.line_height(),
        )
        .unwrap();

        // println!("{:?}", cursor_position);

        let mut cursor_line_index = 0;

        for (index, block) in blocks.iter_mut().enumerate() {
            if index < input.cursor_position.block_index {
                cursor_line_index +=
                    block.line_length(context.text_system(), style.font(), font_size);
            }
        }

        let cursor = fill(
            Bounds::new(
                point(
                    bounds.left() + cursor_position.x - px(1.),
                    bounds.top()
                        + context.line_height() * cursor_line_index
                        + cursor_position.y
                        + px(4.),
                ),
                size(px(2.), px(16.)),
            ),
            rgb(COLOR_BLUE_DARK),
        );

        PrepaintState {
            blocks: Some(content),
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
        let blocks = prepaint.blocks.take().unwrap().into_iter();
        let headline_rectangles = prepaint.headline_rectangles.clone();

        let mut line_count = 0;
        for block in blocks {
            let line_length = block.wrap_boundaries.len() + 1;
            let point = Point::new(
                bounds.origin.x,
                bounds.origin.y + (context.line_height() * px(line_count as f32)),
            );
            block.paint(point, context.line_height(), context).unwrap();

            line_count += line_length;
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

fn position_for_index(
    line: &WrappedLine,
    index: usize,
    line_height: Pixels,
) -> Option<Point<Pixels>> {
    let mut line_start_index = 0;
    let line_end_indices = line
        .wrap_boundaries
        .iter()
        .map(|wrap_boundary| {
            let run = &line.unwrapped_layout.runs[wrap_boundary.run_ix];
            let glyph = &run.glyphs[wrap_boundary.glyph_ix];
            glyph.index
        })
        .chain([line.len()])
        .enumerate();

    for (i, line_end_index) in line_end_indices {
        let line_y = i as f32 * line_height;
        if index < line_start_index {
            break;
        } else if index > line_end_index {
            line_start_index = line_end_index;

            continue;
        } else if index == line_end_index && index < line.len() {
            // This condition is the patch to the original position_for_index
            // We are basically checking whether the index is at the end of a soft-wrapped line

            line_start_index = line_end_index;

            continue;
        } else {
            let line_start_x = line.unwrapped_layout.x_for_index(line_start_index);
            let x = line.unwrapped_layout.x_for_index(index) - line_start_x;

            return Some(point(x, line_y));
        }
    }

    None
}
