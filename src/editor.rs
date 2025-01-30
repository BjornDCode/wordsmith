use gpui::{
    div, fill, point, prelude::*, px, rgb, size, AppContext, Bounds, FocusHandle, FocusableView,
    Font, FontWeight, Hsla, PaintQuad, Point, Render, SharedString, Style, TextRun, View,
    ViewContext, WrappedLine,
};

use crate::{ExampleEditorAction, COLOR_BLUE_DARK, COLOR_GRAY_700, COLOR_GRAY_800, COLOR_PINK};

pub struct Editor {
    focus_handle: FocusHandle,
    content: SharedString,
}

impl Editor {
    pub fn new(focus_handle: FocusHandle) -> Editor {
        return Editor {
            focus_handle,
            content: "Yo".into(),
        };
    }

    fn temp(&mut self, _: &ExampleEditorAction, _context: &mut ViewContext<Self>) {
        println!("Pressed: a");
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
            .on_action(context.listener(Self::temp))
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
    lines: Option<Vec<WrappedLine>>,
    cursor: Option<PaintQuad>,
    display_map: DisplayMap,
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
        let style = context.text_style();
        let font_size = style.font_size.to_pixels(context.rem_size());

        let text = "## This is a headline\n\nThis is a paragraph with some bold text, some italic text and some mixed text.\n\n\n### Another headline\n\nYo, some more text";

        let mut spans: Vec<TextSpan> = vec![];
        let mut offset = 0;
        let mut display_map = DisplayMap {
            hidden: vec![],
            headlines: vec![],
        };

        for (index, line) in text.lines().enumerate() {
            if !line.is_empty() {
                if line.starts_with('#') {
                    let removed_count: usize = display_map
                        .hidden
                        .iter()
                        .map(|range| range.start + range.length)
                        .sum();

                    let level = line
                        .chars()
                        .take_while(|&character| character == '#')
                        .count();

                    display_map.hidden.push(HiddenDisplayMapRange {
                        start: offset,
                        length: level + 1,
                    });

                    display_map.headlines.push(HeadingLine {
                        line_index: index,
                        level,
                    });

                    spans.push(TextSpan {
                        start: offset - removed_count,
                        length: line.len() - level - 1,
                        kind: TextSpanType::Headline,
                    });
                }

                offset += line.len();
            }

            offset += 1; // Newline character
        }

        let prepared_content = apply_display_map(text, display_map.clone());
        let runs = get_text_runs_from_spans(&prepared_content, spans, style.font());

        let lines = context
            .text_system()
            .shape_text(prepared_content.into(), font_size, &runs, Some(px(480.)))
            .unwrap()
            .to_vec();

        let cursor = fill(
            Bounds::new(
                point(
                    px((bounds.left().to_f64() + 20. - 1.) as f32),
                    px((bounds.top().to_f64() + 4.) as f32),
                ),
                size(px(2.), px(16.)),
            ),
            rgb(COLOR_BLUE_DARK),
        );

        PrepaintState {
            lines: Some(lines),
            cursor: Some(cursor),
            display_map,
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
        let lines = prepaint.lines.take().unwrap().into_iter().enumerate();
        let display_map = prepaint.display_map.clone();

        for (index, line) in lines {
            let point = Point::new(
                bounds.origin.x,
                bounds.origin.y + (context.line_height() * index),
            );
            line.paint(point, context.line_height(), context).unwrap();
        }

        for headline in display_map.headlines {
            let width = px(16. * headline.level as f32);
            let rect = fill(
                Bounds::new(
                    point(
                        bounds.origin.x - width - px(16.),
                        bounds.origin.y + (context.line_height() * headline.line_index) + px(4.),
                    ),
                    size(width, px(16.)),
                ),
                rgb(COLOR_GRAY_800),
            );
            context.paint_quad(rect);
        }

        if focus_handle.is_focused(context) {
            if let Some(cursor) = prepaint.cursor.take() {
                context.paint_quad(cursor);
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct TextSpan {
    start: usize,
    length: usize,
    kind: TextSpanType,
}

#[derive(Debug, Clone, Copy)]
enum TextSpanType {
    Normal,
    Headline,
}

#[derive(Debug, Clone)]
struct DisplayMap {
    hidden: Vec<HiddenDisplayMapRange>,
    headlines: Vec<HeadingLine>,
}

#[derive(Debug, Clone, Copy)]
struct HiddenDisplayMapRange {
    start: usize,
    length: usize,
}

#[derive(Debug, Clone, Copy)]
struct HeadingLine {
    line_index: usize,
    level: usize,
}

fn get_text_runs_from_spans(content: &String, spans: Vec<TextSpan>, font: Font) -> Vec<TextRun> {
    if spans.len() == 0 {
        return vec![TextRun {
            len: content.len(),
            font: font.clone(),
            color: Hsla::from(rgb(COLOR_GRAY_700)),
            background_color: None,
            underline: None,
            strikethrough: None,
        }];
    }

    let mut normal_spans: Vec<TextSpan> = vec![];

    let mut position = 0;

    for span in &spans {
        if position < span.start {
            normal_spans.push(TextSpan {
                start: position,
                length: span.start - position,
                kind: TextSpanType::Normal,
            });
        }

        position = position.max(span.start + span.length)
    }

    if position < content.len() {
        normal_spans.push(TextSpan {
            start: position,
            length: content.len() - position,
            kind: TextSpanType::Normal,
        });
    }

    let mut all_spans: Vec<TextSpan> = spans.clone();
    all_spans.append(&mut normal_spans);
    all_spans.sort_by_key(|span| span.start);

    let mut runs: Vec<TextRun> = vec![];

    for span in all_spans {
        let run = match span.kind {
            TextSpanType::Normal => TextRun {
                len: span.length,
                font: font.clone(),
                color: Hsla::from(rgb(COLOR_GRAY_700)),
                background_color: None,
                underline: None,
                strikethrough: None,
            },
            TextSpanType::Headline => TextRun {
                len: span.length,
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

fn apply_display_map(content: &str, display_map: DisplayMap) -> String {
    let mut modified = String::from(content);

    let mut count = 0;

    for range in display_map.hidden {
        modified.drain(range.start - count..range.start + range.length - count);
        count += range.length;
    }

    return modified;
}
