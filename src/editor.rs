use gpui::{
    div, hsla, prelude::*, px, rgb, AppContext, FocusHandle, FocusableView, Font, FontStyle,
    FontWeight, Render, ShapedLine, SharedString, StrikethroughStyle, Style, TextRun, TextStyle,
    UnderlineStyle, View, ViewContext,
};

use crate::{ExampleEditorAction, COLOR_PINK};

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
        let is_focused = self.focus_handle.is_focused(context);

        div()
            .track_focus(&self.focus_handle(context))
            .key_context("editor")
            .on_action(context.listener(Self::temp))
            .pt_8()
            .group("editor-container")
            .child(
                div().bg(rgb(COLOR_PINK)).w(px(480.)).child(EditorElement {
                    input: context.view().clone(),
                }), // .when(is_focused, |this| this.child("CURSOR")),
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
    line: Option<ShapedLine>,
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
        let mut style = Style::default();

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

        let text = "Normal underline background strike bold italic";

        let runs = vec![
            TextRun {
                len: 7,
                font: style.font(),
                color: style.color,
                background_color: None,
                underline: None,
                strikethrough: None,
            },
            TextRun {
                len: 9,
                font: style.font(),
                color: style.color,
                background_color: None,
                underline: Some(UnderlineStyle {
                    color: Some(hsla(0., 0., 0., 0.4)),
                    thickness: px(2.),
                    wavy: true,
                }),
                strikethrough: None,
            },
            TextRun {
                len: 1,
                font: style.font(),
                color: style.color,
                background_color: None,
                underline: None,
                strikethrough: None,
            },
            TextRun {
                len: 10,
                font: style.font(),
                color: style.color,
                background_color: Some(hsla(0., 0., 0., 0.2)),
                underline: None,
                strikethrough: None,
            },
            TextRun {
                len: 1,
                font: style.font(),
                color: style.color,
                background_color: None,
                underline: None,
                strikethrough: None,
            },
            TextRun {
                len: 6,
                font: style.font(),
                color: style.color,
                background_color: None,
                underline: None,
                strikethrough: Some(StrikethroughStyle {
                    thickness: px(2.),
                    color: Some(hsla(0., 0., 0., 0.8)),
                }),
            },
            TextRun {
                len: 1,
                font: style.font(),
                color: style.color,
                background_color: None,
                underline: None,
                strikethrough: None,
            },
            TextRun {
                len: 4,
                font: Font {
                    weight: FontWeight::EXTRA_BOLD,
                    ..style.font()
                },
                color: style.color,
                background_color: None,
                underline: None,
                strikethrough: None,
            },
            TextRun {
                len: 1,
                font: style.font(),
                color: style.color,
                background_color: None,
                underline: None,
                strikethrough: None,
            },
            TextRun {
                len: 6,
                font: Font {
                    style: FontStyle::Italic,
                    ..style.font()
                },
                color: style.color,
                background_color: None,
                underline: None,
                strikethrough: None,
            },
        ];

        let line = context
            .text_system()
            .shape_line(text.into(), font_size, &runs)
            .unwrap();

        PrepaintState { line: Some(line) }
    }

    fn paint(
        &mut self,
        id: Option<&gpui::GlobalElementId>,
        bounds: gpui::Bounds<gpui::Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        context: &mut gpui::WindowContext,
    ) {
        let line = prepaint.line.take().unwrap();

        line.paint(bounds.origin, context.line_height(), context)
            .unwrap();
    }
}
