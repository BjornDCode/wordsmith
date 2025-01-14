use gpui::{div, prelude::*, rems, rgb, AppContext, WindowOptions};

const COLOR_WHITE: u32 = 0xffffff;
const COLOR_BLACK: u32 = 0x000000;
const COLOR_PINK: u32 = 0xfce7f3;

const COLOR_GRAY_50: u32 = 0xf9fafb;
const COLOR_GRAY_100: u32 = 0xf3f4f6;
const COLOR_GRAY_200: u32 = 0xe5e7eb;
const COLOR_GRAY_300: u32 = 0xd1d5db;
const COLOR_GRAY_400: u32 = 0x9ca3af;
const COLOR_GRAY_500: u32 = 0x6b7280;
const COLOR_GRAY_600: u32 = 0x4b5563;
const COLOR_GRAY_700: u32 = 0x374151;
const COLOR_GRAY_800: u32 = 0x1f2937;
const COLOR_GRAY_900: u32 = 0x111827;
const COLOR_GRAY_950: u32 = 0x030712;

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
        div()
            .flex()
            .flex_row()
            .bg(rgb(COLOR_WHITE))
            .size_full()
            .text_color(rgb(COLOR_BLACK))
            .children(vec![main_content(), sidebar()])
    }
}

fn main_content() -> gpui::Div {
    div().flex_1().child("Main")
}

fn sidebar() -> gpui::Div {
    div()
        .w(rems(15.))
        .border_l_1()
        .border_color(rgb(COLOR_GRAY_100))
        .p(rems(1.))
        .child("Sidebar")
}
