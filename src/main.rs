use std::{fs, path::PathBuf};

use gpui::{
    actions, div, prelude::*, px, rems, rgb, size, svg, AppContext, AssetSource, Bounds,
    FocusHandle, FocusableView, KeyBinding, SharedString, ViewContext, WindowBounds, WindowOptions,
};

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

actions!(app, [Quit, ToggleSidebar]);

fn main() {
    gpui::App::new()
        .with_assets(Assets {
            base: PathBuf::from("resources"),
        })
        .run(|context: &mut AppContext| {
            let bounds = Bounds::centered(None, size(px(800.), px(600.)), context);

            context.bind_keys([
                KeyBinding::new("cmd-q", Quit, None),
                KeyBinding::new("cmd-b", ToggleSidebar, None),
            ]);

            context.on_action(|_: &Quit, context| context.quit());

            let window = context
                .open_window(
                    WindowOptions {
                        window_bounds: Some(WindowBounds::Windowed(bounds)),
                        ..Default::default()
                    },
                    |context| context.new_view(|context| Wordsmith::new(context.focus_handle())),
                )
                .unwrap();

            context
                .on_keyboard_layout_change({
                    move |context| {
                        window.update(context, |_, context| context.notify()).ok();
                    }
                })
                .detach();

            window
                .update(context, |view, context| {
                    context.focus_self();
                    context.activate(true);
                })
                .unwrap();
        });
}

struct Assets {
    base: PathBuf,
}

impl AssetSource for Assets {
    fn load(&self, path: &str) -> gpui::Result<Option<std::borrow::Cow<'static, [u8]>>> {
        fs::read(self.base.join(path))
            .map(|data| Some(std::borrow::Cow::Owned(data)))
            .map_err(|err| err.into())
    }

    fn list(&self, path: &str) -> gpui::Result<Vec<gpui::SharedString>> {
        fs::read_dir(self.base.join(path))
            .map(|entries| {
                entries
                    .filter_map(|entry| {
                        entry
                            .ok()
                            .and_then(|entry| entry.file_name().into_string().ok())
                            .map(SharedString::from)
                    })
                    .collect()
            })
            .map_err(|err| err.into())
    }
}

struct Wordsmith {
    focus_handle: FocusHandle,
    show_sidebar: bool,
}

impl Wordsmith {
    pub fn new(focus_handle: FocusHandle) -> Wordsmith {
        Wordsmith {
            focus_handle,
            show_sidebar: true,
        }
    }

    fn toggle_sidebar(&mut self, _: &ToggleSidebar, context: &mut ViewContext<Self>) {
        self.show_sidebar = !self.show_sidebar;

        context.notify();
    }
}

impl FocusableView for Wordsmith {
    fn focus_handle(&self, _context: &AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for Wordsmith {
    fn render(&mut self, context: &mut gpui::ViewContext<Self>) -> impl gpui::IntoElement {
        let children = if self.show_sidebar {
            vec![main_content(), sidebar()]
        } else {
            vec![main_content()]
        };

        div()
            .flex()
            .flex_row()
            .track_focus(&self.focus_handle(context))
            .on_action(context.listener(Self::toggle_sidebar))
            .bg(rgb(COLOR_WHITE))
            .size_full()
            .font_family("MonoLisa")
            .text_color(rgb(COLOR_BLACK))
            .children(children)
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
        .children(vec![mode_selector()])
}

fn mode_selector() -> gpui::Div {
    div().flex().flex_row().gap_2().children(vec![
        radio_button("Outline", "icons/outline.svg"),
        radio_button("Write", "icons/write.svg"),
        radio_button("Edit", "icons/edit.svg"),
    ])
}

fn radio_button(label: &'static str, icon: &'static str) -> gpui::Div {
    div().flex().flex_1().flex_col().gap_1().children(vec![
        div()
            .flex()
            .justify_center()
            .bg(rgb(COLOR_GRAY_100))
            .py_1()
            .border_1()
            .border_color(rgb(COLOR_GRAY_200))
            .rounded(px(3.))
            .hover(|this| this.bg(rgb(COLOR_GRAY_200)))
            .group("button")
            .child(
                svg()
                    .path(icon)
                    .size_6()
                    .text_color(rgb(COLOR_GRAY_500))
                    .group_hover("button", |this| this.text_color(rgb(COLOR_GRAY_600))),
            ),
        div()
            .flex()
            .justify_center()
            .text_color(rgb(COLOR_GRAY_600))
            .text_size(px(8.))
            .child(label),
    ])
}
