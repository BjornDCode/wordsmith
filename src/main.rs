mod buffer;
mod content;
mod editor;
mod text;

use std::{fs, path::PathBuf};

use buffer::Buffer;
use editor::Editor;
use gpui::{
    actions, div, img, impl_actions, prelude::*, px, rems, rgb, size, svg, AppContext, AssetSource,
    Bounds, FocusHandle, FocusableView, KeyBinding, Menu, MenuItem, MouseButton, SharedString,
    View, ViewContext, WindowBounds, WindowOptions,
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

const COLOR_BLUE_LIGHT: u32 = 0xe0f2fe;
const COLOR_BLUE_MEDIUM: u32 = 0x7dd3fc;
const COLOR_BLUE_DARK: u32 = 0x0ea5e9;

actions!(
    app,
    [
        // App
        Quit,
        ToggleSidebar,
        // Editor
        MoveLeft,
        MoveRight,
        MoveUp,
        MoveDown,
        MoveBeginningOfFile,
        MoveEndOfFile,
        MoveBeginningOfLine,
        MoveEndOfLine,
        MoveBeginningOfWord,
        MoveEndOfWord,
        SelectLeft,
        SelectRight,
        SelectUp,
        SelectDown,
        SelectBeginningOfFile,
        SelectEndOfFile,
        SelectBeginningOfLine,
        SelectEndOfLine,
        SelectBeginningOfWord,
        SelectEndOfWord,
        SelectAll,
        RemoveSelection,
        Backspace,
        Enter,
        // Clipboard
        Copy,
        Cut,
        Paste,
        // File
        Save,
    ]
);
impl_actions!(app, [SetMode]);

#[derive(Clone, Default, PartialEq, serde::Deserialize, schemars::JsonSchema)]
struct SetMode {
    mode: Mode,
}

impl SetMode {
    pub fn mode(mode: Mode) -> SetMode {
        SetMode { mode }
    }
}

fn main() {
    let executable_path = std::env::current_exe().unwrap();
    let executable_dir = executable_path.parent().unwrap();

    // Try multiple potential resource paths
    let resource_paths = vec![
        // Dev environment: resources are in the project root
        PathBuf::from("resources"),
        // Bundled app: resources are next to the executable
        executable_dir.join("resources"),
        // macOS app bundle: resources are in the Resources directory
        executable_dir.join("../Resources/resources"),
    ];

    // Find the first path that exists
    let resources_path = resource_paths
        .into_iter()
        .find(|path| path.exists())
        .unwrap_or_else(|| PathBuf::from("resources"));

    println!("Using resources path: {:?}", resources_path);

    gpui::App::new()
        .with_assets(Assets {
            base: resources_path,
        })
        .run(|context: &mut AppContext| {
            let bounds = Bounds::centered(None, size(px(1024.), px(768.)), context);

            context.bind_keys([
                KeyBinding::new("cmd-q", Quit, None),
                KeyBinding::new("cmd-b", ToggleSidebar, None),
                // KeyBinding::new("cmd-1", SetMode::mode(Mode::Outline), None),
                KeyBinding::new("cmd-2", SetMode::mode(Mode::Write), None),
                // KeyBinding::new("cmd-3", SetMode::mode(Mode::Edit), None),
                KeyBinding::new("left", MoveLeft, "editor".into()),
                KeyBinding::new("right", MoveRight, "editor".into()),
                KeyBinding::new("up", MoveUp, "editor".into()),
                KeyBinding::new("down", MoveDown, "editor".into()),
                KeyBinding::new("cmd-up", MoveBeginningOfFile, "editor".into()),
                KeyBinding::new("cmd-down", MoveEndOfFile, "editor".into()),
                KeyBinding::new("cmd-left", MoveBeginningOfLine, "editor".into()),
                KeyBinding::new("cmd-right", MoveEndOfLine, "editor".into()),
                KeyBinding::new("alt-left", MoveBeginningOfWord, "editor".into()),
                KeyBinding::new("alt-right", MoveEndOfWord, "editor".into()),
                KeyBinding::new("shift-left", SelectLeft, "editor".into()),
                KeyBinding::new("shift-right", SelectRight, "editor".into()),
                KeyBinding::new("shift-up", SelectUp, "editor".into()),
                KeyBinding::new("shift-down", SelectDown, "editor".into()),
                KeyBinding::new("cmd-shift-up", SelectBeginningOfFile, "editor".into()),
                KeyBinding::new("cmd-shift-down", SelectEndOfFile, "editor".into()),
                KeyBinding::new("cmd-shift-left", SelectBeginningOfLine, "editor".into()),
                KeyBinding::new("cmd-shift-right", SelectEndOfLine, "editor".into()),
                KeyBinding::new("alt-shift-left", SelectBeginningOfWord, "editor".into()),
                KeyBinding::new("alt-shift-right", SelectEndOfWord, "editor".into()),
                KeyBinding::new("cmd-a", SelectAll, "editor".into()),
                KeyBinding::new("escape", RemoveSelection, "editor".into()),
                KeyBinding::new("backspace", Backspace, "editor".into()),
                KeyBinding::new("enter", Enter, "editor".into()),
                KeyBinding::new("cmd-s", Save, "editor".into()),
                KeyBinding::new("cmd-c", Copy, "editor".into()),
                KeyBinding::new("cmd-x", Cut, "editor".into()),
                KeyBinding::new("cmd-v", Paste, "editor".into()),
            ]);

            context.on_action(|_: &Quit, context| context.quit());

            context.set_menus(vec![
                Menu {
                    name: "Wordsmith".into(),
                    items: vec![MenuItem::action("Quit", Quit)],
                },
                Menu {
                    name: "File".into(),
                    items: vec![MenuItem::action("Save", Save)],
                },
            ]);

            let window = context
                .open_window(
                    WindowOptions {
                        window_bounds: Some(WindowBounds::Windowed(bounds)),
                        ..Default::default()
                    },
                    |context| {
                        let editor = context.new_view(|context| {
                            Editor::new(Buffer::empty(), context.focus_handle())
                        });

                        context.new_view(|context| Wordsmith::new(context.focus_handle(), editor))
                    },
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
                    context.focus_view(&view.editor);
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
    mode: Mode,
    editor: View<Editor>,
}

impl Wordsmith {
    pub fn new(focus_handle: FocusHandle, editor: View<Editor>) -> Wordsmith {
        Wordsmith {
            focus_handle,
            show_sidebar: true,
            mode: Mode::Write,
            editor,
        }
    }

    fn toggle_sidebar(&mut self, _: &ToggleSidebar, context: &mut ViewContext<Self>) {
        self.show_sidebar = !self.show_sidebar;

        context.notify();
    }

    fn set_mode(&mut self, event: &SetMode, context: &mut ViewContext<Self>) {
        self.mode = event.mode.clone();

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
            vec![
                main_content(self.editor.clone()),
                sidebar(self.mode.clone()),
            ]
        } else {
            vec![main_content(self.editor.clone())]
        };

        div()
            .flex()
            .flex_row()
            .track_focus(&self.focus_handle(context))
            .on_action(context.listener(Self::toggle_sidebar))
            .on_action(context.listener(Self::set_mode))
            .bg(rgb(COLOR_WHITE))
            .size_full()
            .font_family("MonoLisa")
            .text_color(rgb(COLOR_BLACK))
            .children(children)
    }
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, schemars::JsonSchema)]
enum Mode {
    Outline,
    Write,
    Edit,
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Write
    }
}

fn main_content(editor: View<Editor>) -> gpui::Div {
    div().flex().justify_center().flex_1().child(editor)
}

fn sidebar(mode: Mode) -> gpui::Div {
    div()
        .w(rems(15.))
        .border_l_1()
        .border_color(rgb(COLOR_GRAY_100))
        .p(rems(1.))
        .children(vec![mode_selector(mode)])
}

fn mode_selector(mode: Mode) -> gpui::Div {
    div().flex().flex_row().gap_2().children(vec![
        radio_button(
            "Outline",
            "icons/outline.svg",
            mode == Mode::Outline,
            true,
            Mode::Outline,
        ),
        radio_button(
            "Write",
            "icons/write.svg",
            mode == Mode::Write,
            false,
            Mode::Write,
        ),
        radio_button(
            "Edit",
            "icons/edit.svg",
            mode == Mode::Edit,
            true,
            Mode::Edit,
        ),
    ])
}

fn radio_button(
    label: &'static str,
    icon: &'static str,
    active: bool,
    disabled: bool,
    mode: Mode,
) -> gpui::Div {
    div()
        .flex()
        .flex_1()
        .flex_col()
        .gap_1()
        .relative()
        .children(vec![
            div()
                .flex()
                .justify_center()
                .py_1()
                .border_1()
                .when(disabled, |this| {
                    this.border_color(rgb(COLOR_GRAY_100))
                        .bg(rgb(COLOR_GRAY_50))
                })
                .when(!disabled, |this| {
                    this.when(active, |this| {
                        this.border_color(rgb(COLOR_BLUE_MEDIUM))
                            .bg(rgb(COLOR_BLUE_LIGHT))
                            .group("active-button")
                    })
                    .when(!active, |this| {
                        this.border_color(rgb(COLOR_GRAY_200))
                            .bg(rgb(COLOR_GRAY_100))
                            .hover(|this| this.bg(rgb(COLOR_GRAY_200)))
                    })
                })
                .group("button")
                .rounded(px(3.))
                .child(
                    svg()
                        .path(icon)
                        .size_6()
                        .when(disabled, |this| this.text_color(rgb(COLOR_GRAY_300)))
                        .when(!disabled, |this| {
                            this.when(active, |this| this.text_color(rgb(COLOR_BLUE_DARK)))
                                .when(!active, |this| {
                                    this.text_color(rgb(COLOR_GRAY_500))
                                        .group_hover("button", |this| {
                                            this.text_color(rgb(COLOR_GRAY_600))
                                        })
                                })
                        }),
                )
                .when(!disabled, |this| {
                    this.on_mouse_up(MouseButton::Left, move |_event, context| {
                        context.dispatch_action(Box::new(SetMode::mode(mode.clone())));
                    })
                }),
            div()
                .flex()
                .justify_center()
                .when(disabled, |this| this.text_color(rgb(COLOR_GRAY_300)))
                .when(!disabled, |this| {
                    this.when(active, |this| this.text_color(rgb(COLOR_BLUE_DARK)))
                        .when(!active, |this| this.text_color(rgb(COLOR_GRAY_600)))
                })
                .text_size(px(8.))
                .child(label),
            div()
                .absolute()
                .top(px(-4.))
                .left(px(-4.))
                .child(img("images/wip.png").w(px(21.)).h(px(12.)))
                .when(!disabled, |this| this.invisible()),
        ])
}
