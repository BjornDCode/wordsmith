use gpui::{
    div, fill, hsla, point, prelude::*, px, rgb, size, AppContext, Bounds, FocusHandle,
    FocusableView, Font, FontStyle, FontWeight, Hsla, PaintQuad, Point, Render, ShapedLine,
    SharedString, StrikethroughStyle, Style, TextRun, TextStyle, UnderlineStyle, View, ViewContext,
    WrappedLine,
};

use crate::{
    ExampleEditorAction, COLOR_BLUE_DARK, COLOR_BLUE_MEDIUM, COLOR_GRAY_700, COLOR_GRAY_800,
    COLOR_PINK,
};

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

    fn temp(&mut self, _: &ExampleEditorAction, context: &mut ViewContext<Self>) {
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
}

impl Element for EditorElement {
    type RequestLayoutState = ();
    type PrepaintState = PrepaintState;

    fn id(&self) -> Option<gpui::ElementId> {
        None
    }

    fn request_layout(
        &mut self,
        id: Option<&gpui::GlobalElementId>,
        context: &mut gpui::WindowContext,
    ) -> (gpui::LayoutId, Self::RequestLayoutState) {
        let style = Style::default();

        (context.request_layout(style, []), ())
    }

    fn prepaint(
        &mut self,
        id: Option<&gpui::GlobalElementId>,
        bounds: gpui::Bounds<gpui::Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        context: &mut gpui::WindowContext,
    ) -> Self::PrepaintState {
        let style = context.text_style();
        let font_size = style.font_size.to_pixels(context.rem_size());

        let text = "This is a headline\n\nThis is a paragraph with some bold text, some italic text and some mixed text.";

        let nodes = vec![
            Node::Headline(Headline {
                start: 0,
                end: 18,
                decorations: vec![Span {
                    start: 10,
                    end: 18,
                    weight: FontWeight::NORMAL,
                    style: FontStyle::Italic,
                }],
            }),
            Node::Paragraph(Paragraph {
                start: 18,
                end: 100,
                decorations: vec![
                    Span {
                        start: 50,
                        end: 54,
                        weight: FontWeight::EXTRA_BOLD,
                        style: FontStyle::Normal,
                    },
                    Span {
                        start: 66,
                        end: 72,
                        weight: FontWeight::NORMAL,
                        style: FontStyle::Italic,
                    },
                    Span {
                        start: 82,
                        end: 87,
                        weight: FontWeight::EXTRA_BOLD,
                        style: FontStyle::Normal,
                    },
                    Span {
                        start: 87,
                        end: 92,
                        weight: FontWeight::EXTRA_BOLD,
                        style: FontStyle::Italic,
                    },
                ],
            }),
        ];

        let runs = get_text_runs(nodes, style.font());

        let lines = context
            .text_system()
            .shape_text(text.into(), font_size, &runs, Some(px(480.)))
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
        }
    }

    fn paint(
        &mut self,
        id: Option<&gpui::GlobalElementId>,
        bounds: gpui::Bounds<gpui::Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        context: &mut gpui::WindowContext,
    ) {
        let focus_handle = self.input.read(context).focus_handle.clone();
        let lines = prepaint.lines.take().unwrap().into_iter().enumerate();

        for (index, line) in lines {
            let point = Point::new(
                bounds.origin.x,
                bounds.origin.y + (context.line_height() * index),
            );
            line.paint(point, context.line_height(), context).unwrap();
        }

        if focus_handle.is_focused(context) {
            if let Some(cursor) = prepaint.cursor.take() {
                context.paint_quad(cursor);
            }
        }
    }
}

enum Node {
    Headline(Headline),
    Paragraph(Paragraph),
}

struct Headline {
    start: usize,
    end: usize,
    decorations: Vec<Span>,
}

struct Paragraph {
    start: usize,
    end: usize,
    decorations: Vec<Span>,
}

struct Span {
    start: usize,
    end: usize,
    weight: FontWeight,
    style: FontStyle,
}

fn get_text_runs_for_headline(node: Headline, font: Font) -> Vec<TextRun> {
    let mut runs = Vec::new();
    let mut current_pos = node.start;

    // Sort decorations by start position for sequential processing
    let mut decorations = node.decorations;
    decorations.sort_by_key(|span| span.start);

    for span in decorations {
        // If there's a gap before the decoration, add a base run
        if current_pos < span.start {
            runs.push(TextRun {
                len: span.start - current_pos,
                font: Font {
                    weight: FontWeight::EXTRA_BOLD,
                    ..font.clone()
                },
                color: Hsla::from(rgb(COLOR_GRAY_800)),
                background_color: None,
                underline: None,
                strikethrough: None,
            });
        }

        // Add the decorated run
        runs.push(TextRun {
            len: span.end - span.start,
            font: Font {
                style: span.style,
                weight: FontWeight::EXTRA_BOLD,
                ..font.clone()
            },
            color: Hsla::from(rgb(COLOR_GRAY_800)),
            background_color: None,
            underline: None,
            strikethrough: None,
        });

        current_pos = span.end;
    }

    // If there's remaining text after the last decoration, add a final base run
    if current_pos < node.end {
        runs.push(TextRun {
            len: node.end - current_pos,
            font: Font {
                weight: FontWeight::EXTRA_BOLD,
                ..font
            },
            color: Hsla::from(rgb(COLOR_GRAY_800)),
            background_color: None,
            underline: None,
            strikethrough: None,
        });
    }

    runs
}

fn get_text_runs_for_paragraph(node: Paragraph, font: Font) -> Vec<TextRun> {
    let mut runs = Vec::new();
    let mut current_pos = node.start;

    // Sort decorations by start position for sequential processing
    let mut decorations = node.decorations;
    decorations.sort_by_key(|span| span.start);

    for span in decorations {
        // If there's a gap before the decoration, add a base run
        if current_pos < span.start {
            runs.push(TextRun {
                len: span.start - current_pos,
                font: font.clone(),
                color: Hsla::from(rgb(COLOR_GRAY_700)),
                background_color: None,
                underline: None,
                strikethrough: None,
            });
        }

        // Add the decorated run
        runs.push(TextRun {
            len: span.end - span.start,
            font: Font {
                weight: span.weight,
                style: span.style,
                ..font.clone()
            },
            color: Hsla::from(rgb(COLOR_GRAY_700)),
            background_color: None,
            underline: None,
            strikethrough: None,
        });

        current_pos = span.end;
    }

    // If there's remaining text after the last decoration, add a final base run
    if current_pos < node.end {
        runs.push(TextRun {
            len: node.end - current_pos,
            font: font.clone(),
            color: Hsla::from(rgb(COLOR_GRAY_700)),
            background_color: None,
            underline: None,
            strikethrough: None,
        });
    }

    runs
}

fn get_text_runs_for_node(node: Node, font: Font) -> Vec<TextRun> {
    match node {
        Node::Headline(node) => get_text_runs_for_headline(node, font),
        Node::Paragraph(node) => get_text_runs_for_paragraph(node, font),
    }
}

fn get_text_runs(nodes: Vec<Node>, font: Font) -> Vec<TextRun> {
    let mut runs: Vec<TextRun> = vec![];

    for node in nodes {
        let node_runs = get_text_runs_for_node(node, font.clone());

        runs.extend(node_runs);
    }

    return runs;
}
