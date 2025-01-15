use gpui::{
    div, prelude::*, px, rgb, AppContext, FocusHandle, FocusableView, Render, SharedString,
    ViewContext,
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
                div()
                    .bg(rgb(COLOR_PINK))
                    .w(px(480.))
                    .child(self.content.clone())
                    .when(is_focused, |this| this.child("CURSOR")),
            )
    }
}

// impl ViewInputHandler for Editor {}

struct EditorElement {}
