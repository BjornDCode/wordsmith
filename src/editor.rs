use std::ops::Index;

use gpui::{
    div, fill, point, prelude::*, px, rgb, size, AppContext, Bounds, FocusHandle, FocusableView,
    Font, FontWeight, Hsla, PaintQuad, Point, Render, Style, TextRun, View, ViewContext,
    WrappedLine,
};

use crate::{
    text::{Block, Text},
    MoveLeft, COLOR_BLUE_DARK, COLOR_GRAY_700, COLOR_GRAY_800, COLOR_PINK,
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
            content: Text::new("## This is a headline\n\nThis is a paragraph with some bold text, some italic text and some mixed text.\n\n\n### Another headline\n\nYo, some more text"),
            cursor_position: CursorPosition { offset: 55, block_index: 2 }
        };
    }

    fn move_left(&mut self, _: &MoveLeft, context: &mut ViewContext<Self>) {
        if self.cursor_position.offset == 0 {
            if self.cursor_position.block_index == 0 {
                self.cursor_position = CursorPosition {
                    block_index: 0,
                    offset: 0,
                };
            } else {
                let new_y = self.cursor_position.block_index - 1;

                self.cursor_position.block_index = new_y;
                self.cursor_position.offset = self.content.get_line_length(new_y);
            }
        } else {
            self.cursor_position.offset -= 1;
        }

        context.notify();
    }
}

impl FocusableView for Editor {
    fn focus_handle(&self, _context: &AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for Editor {
    fn render(&mut self, context: &mut gpui::ViewContext<Self>) -> impl gpui::IntoElement {
        div()
            .track_focus(&self.focus_handle(context))
            .key_context("editor")
            .on_action(context.listener(Self::move_left))
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

        let blocks = content.blocks();
        let display_content = content.to_display_content();

        let runs = get_text_runs_from_blocks(&blocks, style.font().clone());

        let mut headline_rectangles = vec![];

        for block in blocks {
            if let Block::Headline(headline) = block {
                let width = px(16. * headline.level.length() as f32);

                let rect = fill(
                    Bounds::new(
                        point(
                            bounds.origin.x - width - px(16.),
                            bounds.origin.y
                                + (context.line_height() * headline.original_line_index)
                                + px(4.),
                        ),
                        size(width, px(16.)),
                    ),
                    rgb(COLOR_GRAY_800),
                );

                headline_rectangles.push(rect);
            }
        }

        let blocks = context
            .text_system()
            .shape_text(display_content.into(), font_size, &runs, Some(px(480.)))
            .unwrap()
            .to_vec();

        let block = blocks.index(input.cursor_position.block_index);
        let cursor_position = block
            .position_for_index(input.cursor_position.offset, context.line_height())
            .unwrap();

        let cursor = fill(
            Bounds::new(
                point(
                    bounds.left() + cursor_position.x - px(1.),
                    bounds.top()
                        + context.line_height() * input.cursor_position.block_index
                        + cursor_position.y
                        + px(4.),
                ),
                size(px(2.), px(16.)),
            ),
            rgb(COLOR_BLUE_DARK),
        );

        PrepaintState {
            blocks: Some(blocks),
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
        let blocks = prepaint.blocks.take().unwrap().into_iter().enumerate();
        let headline_rectangles = prepaint.headline_rectangles.clone();

        for (index, block) in blocks {
            let point = Point::new(
                bounds.origin.x,
                bounds.origin.y + (context.line_height() * index),
            );
            block.paint(point, context.line_height(), context).unwrap();
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

fn get_text_runs_from_blocks(blocks: &Vec<Block>, font: Font) -> Vec<TextRun> {
    let mut runs: Vec<TextRun> = vec![];

    for block in blocks {
        let run = match block {
            Block::Newline => TextRun {
                len: 1,
                font: font.clone(),
                color: Hsla::from(rgb(COLOR_GRAY_700)),
                background_color: None,
                underline: None,
                strikethrough: None,
            },
            Block::Paragraph(block) => TextRun {
                len: block.length,
                font: font.clone(),
                color: Hsla::from(rgb(COLOR_GRAY_700)),
                background_color: None,
                underline: None,
                strikethrough: None,
            },
            Block::Headline(block) => TextRun {
                len: block.length - block.level.length() - 1,
                font: Font {
                    weight: FontWeight::EXTRA_BOLD,
                    ..font.clone()
                },
                color: Hsla::from(rgb(COLOR_GRAY_800)),
                background_color: None,
                underline: None,
                strikethrough: None,
            },
        };

        runs.push(run);
    }

    return runs;
}
