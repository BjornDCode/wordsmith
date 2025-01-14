use gpui::{div, prelude::*, rgb, AppContext, WindowOptions};

fn main() {
    gpui::App::new().run(|context: &mut AppContext| {
        context
            .open_window(WindowOptions::default(), |context| {
                context.new_view(|_cx| Wordsmith::new())
            })
            .unwrap();

        context.activate(true);
    });
}

struct Wordsmith {}

impl Wordsmith {
    pub fn new() -> Wordsmith {
        Wordsmith {}
    }
}

impl Render for Wordsmith {
    fn render(&mut self, _context: &mut gpui::ViewContext<Self>) -> impl gpui::IntoElement {
        div().text_color(rgb(0xffffff)).child("Yo")
    }
}
