use gpui::{
    div, prelude::*, px, rgb, AppContext, FocusHandle, FocusableView, Render, SharedString,
};

use crate::COLOR_PINK;

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
}

impl FocusableView for Editor {
    fn focus_handle(&self, _context: &AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for Editor {
    fn render(&mut self, cx: &mut gpui::ViewContext<Self>) -> impl gpui::IntoElement {
        div().pt_8().child(
            div()
                .bg(rgb(COLOR_PINK))
                .w(px(480.))
                .child(self.content.clone()),
        )
    }
}

// impl ViewInputHandler for Editor {}

struct EditorElement {}
