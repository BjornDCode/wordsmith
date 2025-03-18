use std::{cmp::Ordering, env, ops::Range, path::PathBuf};

use gpui::{
    div, fill, point, prelude::*, px, rgb, size, AppContext, Bounds, ClipboardItem,
    ElementInputHandler, FocusHandle, FocusableView, Font, FontWeight, Hsla, PaintQuad,
    PathPromptOptions, Pixels, Point, PromptLevel, ScrollHandle, ShapedLine, SharedString, Style,
    TextRun, View, ViewContext, ViewInputHandler,
};

use crate::{
    buffer::Buffer,
    content::{Content, Line, LineType},
    text::WrappedText,
    Backspace, Copy, Cut, Enter, MoveBeginningOfFile, MoveBeginningOfLine, MoveBeginningOfWord,
    MoveDown, MoveEndOfFile, MoveEndOfLine, MoveEndOfWord, MoveLeft, MoveRight, MoveUp, NewFile,
    OpenFile, Paste, RemoveSelection, Save, SaveAs, SelectAll, SelectBeginningOfFile,
    SelectBeginningOfLine, SelectBeginningOfWord, SelectDown, SelectEndOfFile, SelectEndOfLine,
    SelectEndOfWord, SelectLeft, SelectRight, SelectUp, SetBuffer, COLOR_BLUE_DARK,
    COLOR_BLUE_LIGHT, COLOR_BLUE_MEDIUM, COLOR_GRAY_300, COLOR_GRAY_400, COLOR_GRAY_700,
    COLOR_GRAY_800, COLOR_PINK,
};

const CHARACTER_WIDTH: Pixels = px(10.24);
const LINE_HEIGHT: Pixels = px(24.);
pub const CHARACTER_COUNT_PER_LINE: usize = 50;
const EDITOR_HORIZONTAL_MARGIN: Pixels = px(71.68); // 7 (6 headline markers + 1 space) * CHARACTERWIDTH;
const EDITOR_VERTICAL_MARGIN: Pixels = px(32.);
const EDITOR_BASE_WIDTH: Pixels = px(512.);
pub const CONTAINER_WIDTH: Pixels = px(655.36); // Base width + Margin * 2

pub struct Editor {
    buffer: Buffer,
    edit_location: EditLocation,
    focus_handle: FocusHandle,
    scroll_handle: ScrollHandle,
}

#[derive(Debug, Clone)]
enum EditLocation {
    Cursor(Cursor),
    Selection(Selection),
}

#[derive(Debug, Clone)]
pub struct EditorPosition {
    pub x: isize,
    pub y: usize,
}

impl EditorPosition {
    pub fn new(y: usize, x: isize) -> EditorPosition {
        return EditorPosition { x, y };
    }
}

impl PartialEq for EditorPosition {
    fn eq(&self, other: &Self) -> bool {
        return self.x == other.x && self.y == other.y;
    }
}

impl Eq for EditorPosition {}

impl Ord for EditorPosition {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialOrd for EditorPosition {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.y == other.y {
            if self.x < other.x {
                return Some(Ordering::Less);
            }

            if self.x > other.x {
                return Some(Ordering::Greater);
            }

            return Some(Ordering::Equal);
        }

        if self.y < other.y {
            return Some(Ordering::Less);
        }

        if self.y > other.y {
            return Some(Ordering::Greater);
        }

        return Some(Ordering::Equal);
    }
}

#[derive(Debug, Clone)]
struct Cursor {
    position: EditorPosition,
    preferred_x: isize,
}

impl Cursor {
    pub fn new(y: usize, x: isize, preferred_x: isize) -> Cursor {
        return Cursor {
            position: EditorPosition::new(y, x),
            preferred_x,
        };
    }
}

#[derive(Debug, Clone)]
struct Selection {
    start: EditorPosition,
    end: EditorPosition,
}

impl Selection {
    pub fn new(start: EditorPosition, end: EditorPosition) -> Selection {
        return Selection { start, end };
    }
}

impl Selection {
    pub fn direction(&self) -> SelectionDirection {
        if self.end < self.start {
            SelectionDirection::Backwards
        } else {
            SelectionDirection::Forwards
        }
    }

    pub fn smallest(&self) -> EditorPosition {
        if self.start < self.end {
            return self.start.clone();
        } else {
            return self.end.clone();
        }
    }

    pub fn largest(&self) -> EditorPosition {
        if self.start > self.end {
            return self.start.clone();
        } else {
            return self.end.clone();
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SelectionDirection {
    Backwards,
    Forwards,
}

impl Editor {
    pub fn new(buffer: Buffer, focus_handle: FocusHandle) -> Editor {
        let edit_location = EditLocation::Cursor(Cursor::new(0, 0, 0));
        // let edit_location = EditLocation::Selection(Selection::new(
        //     EditorPosition::new(4, 4),
        //     EditorPosition::new(4, 8),
        // ));

        return Editor {
            buffer,
            edit_location,
            focus_handle,
            scroll_handle: ScrollHandle::new(),
        };
    }

    fn set_buffer(&mut self, action: &SetBuffer, context: &mut ViewContext<Self>) {
        let buffer = Buffer::from_path(action.path.clone());

        self.buffer = buffer;

        context.notify();
    }

    fn new_file(&mut self, _: &NewFile, context: &mut ViewContext<Self>) {
        let buffer = Buffer::empty();

        self.buffer = buffer;

        context.notify();
    }

    fn open_file(&mut self, _: &OpenFile, context: &mut ViewContext<Self>) {
        if !self.buffer.pristine() {
            self.prompt_to_save_before_open(context);
        } else {
            self.prompt_to_open_file(context);
        }
    }

    fn prompt_to_save_before_open(&self, context: &mut ViewContext<Self>) {
        let prompt = context.prompt(
            PromptLevel::Warning,
            "Do you want to save the file?",
            None,
            &["Save", "Don't save", "Cancel"],
        );

        // Create a clone of self that we can move into the closure
        let editor = context.view().clone();

        context
            .spawn(move |_, mut context| async move {
                let answer = prompt.await.ok();

                match answer {
                    // Save and then open another file
                    Some(0) => {
                        // First save the file
                        context
                            .update_view(&editor, |editor, cx| {
                                // If the buffer has no file, prompt for a save location first
                                if !editor.buffer.has_file() {
                                    // Use our callback-based method to save first, then open a new file
                                    editor.prompt_to_save_file_with_callback(
                                        cx,
                                        Some(|editor: &mut Editor, cx: &mut ViewContext<Self>| {
                                            editor.prompt_to_open_file(cx)
                                        }),
                                    );
                                } else {
                                    // Otherwise save directly to the existing file
                                    match editor.buffer.save() {
                                        Ok(_) => {
                                            cx.notify();
                                            // Then open a new file
                                            editor.prompt_to_open_file(cx);
                                        }
                                        Err(err) => {
                                            // This shouldn't happen since we checked has_file(), but handle it anyway
                                            let error_message =
                                                format!("Failed to save file: {:?}", err);
                                            let error_prompt = cx.prompt(
                                                PromptLevel::Critical,
                                                &error_message,
                                                None,
                                                &["OK"],
                                            );

                                            cx.foreground_executor()
                                                .spawn(async move {
                                                    error_prompt.await.ok();
                                                })
                                                .detach();
                                        }
                                    }
                                }
                            })
                            .ok();
                    }
                    // Don't save but open another file
                    Some(1) => {
                        context
                            .update_view(&editor, |editor, cx| {
                                editor.prompt_to_open_file(cx);
                            })
                            .ok();
                    }
                    // Cancel
                    _ => {}
                };
            })
            .detach();
    }

    fn prompt_to_save_file(&self, context: &mut ViewContext<Self>) {
        self.prompt_to_save_file_with_callback::<fn(&mut Editor, &mut ViewContext<Self>)>(
            context, None,
        );
    }

    fn prompt_to_save_file_with_callback<F>(
        &self,
        context: &mut ViewContext<Self>,
        callback: Option<F>,
    ) where
        F: FnOnce(&mut Editor, &mut ViewContext<Self>) + Send + 'static,
    {
        let path = get_documents_folder_path();
        let result = context.prompt_for_new_path(path.as_path());

        let editor = context.view().clone();

        context
            .spawn(move |_, mut context| async move {
                let result = result.await;
                // Handle the result of the prompt
                if let Ok(Ok(result)) = result {
                    if let Some(path) = result {
                        // Check if the file has a .md extension
                        if path.extension().is_none() || path.extension().unwrap() != "md" {
                            // If not, show an error
                            let error_prompt = context.prompt(
                                PromptLevel::Critical,
                                "File must have a .md extension",
                                None,
                                &["OK"],
                            );

                            context
                                .foreground_executor()
                                .spawn(async move {
                                    error_prompt.await.ok();
                                })
                                .detach();
                        } else {
                            context
                                .update_view(&editor, |editor, cx| {
                                    // Use the set_file method to associate a file with the buffer and save
                                    match editor.buffer.set_file(path.clone()) {
                                        Ok(_) => {
                                            // File was created and saved successfully
                                            cx.notify();

                                            // Execute callback if provided
                                            if let Some(callback) = callback {
                                                callback(editor, cx);
                                            }
                                        }
                                        Err(error) => {
                                            // Show an error if file creation failed
                                            let error_message =
                                                format!("Failed to save file: {:?}", error);
                                            let error_prompt = cx.prompt(
                                                PromptLevel::Critical,
                                                &error_message,
                                                None,
                                                &["OK"],
                                            );

                                            cx.foreground_executor()
                                                .spawn(async move {
                                                    error_prompt.await.ok();
                                                })
                                                .detach();
                                        }
                                    }
                                })
                                .ok();
                        }
                    }
                }
            })
            .detach();
    }
    fn prompt_to_open_file(&self, context: &mut ViewContext<Self>) {
        let paths = context.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
        });

        context
            .spawn(|_, mut context| async move {
                let result = paths.await.unwrap().unwrap();

                if let Some(paths) = result {
                    let path = paths.first().unwrap();

                    if path.extension().is_none() || path.extension().unwrap() != "md" {
                        let prompt = context.prompt(
                            PromptLevel::Critical,
                            "Can only open .md files",
                            None,
                            &["OK"],
                        );

                        context
                            .foreground_executor()
                            .spawn(async {
                                prompt.await.ok();
                            })
                            .detach();
                    } else {
                        // Only dispatch the SetBuffer action if the file has a .md extension
                        context
                            .update(|context| {
                                context.dispatch_action(Box::new(SetBuffer::new(path.clone())));
                            })
                            .unwrap();
                    }
                }
            })
            .detach();
    }

    fn move_left(&mut self, _: &MoveLeft, context: &mut ViewContext<Self>) {
        let starting_point = match self.edit_location.clone() {
            EditLocation::Cursor(cursor) => cursor.position,
            EditLocation::Selection(selection) => selection.smallest(),
        };

        let position = self.left_position(starting_point.clone());

        self.move_to(position.clone(), position.x, context);
    }

    fn move_right(&mut self, _: &MoveRight, context: &mut ViewContext<Self>) {
        let starting_point = match self.edit_location.clone() {
            EditLocation::Cursor(cursor) => cursor.position,
            EditLocation::Selection(selection) => selection.largest(),
        };

        let position = self.right_position(starting_point.clone());

        self.move_to(position.clone(), position.x, context);
    }

    fn move_up(&mut self, _: &MoveUp, context: &mut ViewContext<Self>) {
        let starting_point = match self.edit_location.clone() {
            EditLocation::Cursor(cursor) => cursor.position,
            EditLocation::Selection(selection) => selection.smallest(),
        };
        let preferred_x = match self.edit_location.clone() {
            EditLocation::Cursor(cursor) => cursor.preferred_x,
            EditLocation::Selection(selection) => selection.smallest().x,
        };

        let position = self.up_position(starting_point.clone(), preferred_x);

        let preferred_x = match self.edit_location.clone() {
            EditLocation::Cursor(cursor) => cursor.preferred_x,
            EditLocation::Selection(_selection) => position.x,
        };

        self.move_to(position, preferred_x, context);
    }

    fn move_down(&mut self, _: &MoveDown, context: &mut ViewContext<Self>) {
        let starting_point = match self.edit_location.clone() {
            EditLocation::Cursor(cursor) => cursor.position,
            EditLocation::Selection(selection) => selection.largest(),
        };
        let preferred_x = match self.edit_location.clone() {
            EditLocation::Cursor(cursor) => cursor.preferred_x,
            EditLocation::Selection(selection) => selection.largest().x,
        };

        let position = self.down_position(starting_point.clone(), preferred_x);
        let preferred_x = match self.edit_location.clone() {
            EditLocation::Cursor(cursor) => cursor.preferred_x,
            EditLocation::Selection(_selection) => position.x,
        };

        self.move_to(position, preferred_x, context);
    }

    fn move_beginning_of_file(&mut self, _: &MoveBeginningOfFile, context: &mut ViewContext<Self>) {
        let position = self.beginning_of_file_position();

        self.move_to(position.clone(), position.x, context);
    }

    fn move_end_of_file(&mut self, _: &MoveEndOfFile, context: &mut ViewContext<Self>) {
        let position = self.end_of_file_position();

        let line = self.buffer.line(position.y);

        self.move_to(position, line.end(), context);
    }

    fn move_beginning_of_line(&mut self, _: &MoveBeginningOfLine, context: &mut ViewContext<Self>) {
        let starting_point = match self.edit_location.clone() {
            EditLocation::Cursor(cursor) => cursor.position,
            EditLocation::Selection(selection) => match selection.direction() {
                SelectionDirection::Backwards => selection.smallest(),
                SelectionDirection::Forwards => selection.largest(),
            },
        };
        let position = self.beginning_of_line_position(starting_point);

        self.move_to(position.clone(), position.x, context);
    }

    fn move_end_of_line(&mut self, _: &MoveEndOfLine, context: &mut ViewContext<Self>) {
        let starting_point = match self.edit_location.clone() {
            EditLocation::Cursor(cursor) => cursor.position,
            EditLocation::Selection(selection) => match selection.direction() {
                SelectionDirection::Backwards => selection.smallest(),
                SelectionDirection::Forwards => selection.largest(),
            },
        };
        let position = self.end_of_line_position(starting_point);

        self.move_to(position.clone(), position.x, context);
    }

    fn move_beginning_of_word(&mut self, _: &MoveBeginningOfWord, context: &mut ViewContext<Self>) {
        let starting_point = match self.edit_location.clone() {
            EditLocation::Cursor(cursor) => cursor.position,
            EditLocation::Selection(selection) => selection.smallest(),
        };
        let position = self.beginning_of_word_position(starting_point);

        self.move_to(position.clone(), position.x, context);
    }

    fn move_end_of_word(&mut self, _: &MoveEndOfWord, context: &mut ViewContext<Self>) {
        let starting_point = match self.edit_location.clone() {
            EditLocation::Cursor(cursor) => cursor.position,
            EditLocation::Selection(selection) => selection.largest(),
        };
        let position = self.end_of_word_position(starting_point);

        self.move_to(position.clone(), position.x, context);
    }

    fn select_left(&mut self, _: &SelectLeft, context: &mut ViewContext<Self>) {
        match self.edit_location.clone() {
            EditLocation::Cursor(cursor) => self.select(
                cursor.position.clone(),
                self.left_position(cursor.position),
                context,
            ),
            EditLocation::Selection(selection) => {
                self.select(selection.start, self.left_position(selection.end), context)
            }
        }
    }

    fn select_right(&mut self, _: &SelectRight, context: &mut ViewContext<Self>) {
        match self.edit_location.clone() {
            EditLocation::Cursor(cursor) => self.select(
                cursor.position.clone(),
                self.right_position(cursor.position),
                context,
            ),
            EditLocation::Selection(selection) => {
                self.select(selection.start, self.right_position(selection.end), context)
            }
        }
    }

    fn select_up(&mut self, _: &SelectUp, context: &mut ViewContext<Self>) {
        match self.edit_location.clone() {
            EditLocation::Cursor(cursor) => {
                self.select_to(
                    self.up_position(cursor.position.clone(), cursor.position.x),
                    context,
                );
            }
            EditLocation::Selection(selection) => match selection.direction() {
                SelectionDirection::Backwards => self.select_to(
                    self.up_position(selection.end.clone(), selection.start.x),
                    context,
                ),
                SelectionDirection::Forwards => self.select(
                    selection.start,
                    self.up_position(selection.end.clone(), selection.end.x),
                    context,
                ),
            },
        };
    }

    fn select_down(&mut self, _: &SelectDown, context: &mut ViewContext<Self>) {
        match self.edit_location.clone() {
            EditLocation::Cursor(cursor) => {
                self.select_to(
                    self.down_position(cursor.position.clone(), cursor.position.x),
                    context,
                );
            }
            EditLocation::Selection(selection) => match selection.direction() {
                SelectionDirection::Backwards => self.select(
                    selection.start,
                    self.down_position(selection.end.clone(), selection.end.x),
                    context,
                ),
                SelectionDirection::Forwards => self.select_to(
                    self.down_position(selection.end.clone(), selection.start.x),
                    context,
                ),
            },
        };
    }

    fn select_beginning_of_file(
        &mut self,
        _: &SelectBeginningOfFile,
        context: &mut ViewContext<Self>,
    ) {
        let starting_point = match self.edit_location.clone() {
            EditLocation::Cursor(cursor) => cursor.position,
            EditLocation::Selection(selection) => selection.start,
        };

        self.select(starting_point, self.beginning_of_file_position(), context);
    }

    fn select_end_of_file(&mut self, _: &SelectEndOfFile, context: &mut ViewContext<Self>) {
        let starting_point = match self.edit_location.clone() {
            EditLocation::Cursor(cursor) => cursor.position,
            EditLocation::Selection(selection) => selection.start,
        };

        self.select(starting_point, self.end_of_file_position(), context);
    }

    fn select_beginning_of_line(
        &mut self,
        _: &SelectBeginningOfLine,
        context: &mut ViewContext<Self>,
    ) {
        match self.edit_location.clone() {
            EditLocation::Cursor(cursor) => self.select(
                cursor.position.clone(),
                self.beginning_of_line_position(cursor.position),
                context,
            ),
            EditLocation::Selection(selection) => match selection.direction() {
                SelectionDirection::Backwards => {
                    self.select_to(self.beginning_of_line_position(selection.end), context)
                }
                SelectionDirection::Forwards => self.select(
                    selection.start,
                    self.beginning_of_line_position(selection.end),
                    context,
                ),
            },
        };
    }

    fn select_end_of_line(&mut self, _: &SelectEndOfLine, context: &mut ViewContext<Self>) {
        match self.edit_location.clone() {
            EditLocation::Cursor(cursor) => self.select(
                cursor.position.clone(),
                self.end_of_line_position(cursor.position),
                context,
            ),
            EditLocation::Selection(selection) => match selection.direction() {
                SelectionDirection::Backwards => {
                    self.select_to(self.end_of_line_position(selection.end), context)
                }
                SelectionDirection::Forwards => self.select(
                    selection.start,
                    self.end_of_line_position(selection.end),
                    context,
                ),
            },
        };
    }

    fn select_beginning_of_word(
        &mut self,
        _: &SelectBeginningOfWord,
        context: &mut ViewContext<Self>,
    ) {
        match self.edit_location.clone() {
            EditLocation::Cursor(cursor) => {
                self.select_to(self.beginning_of_word_position(cursor.position), context);
            }
            EditLocation::Selection(selection) => match selection.direction() {
                SelectionDirection::Backwards => {
                    self.select_to(self.beginning_of_word_position(selection.end), context)
                }
                SelectionDirection::Forwards => self.select(
                    selection.start,
                    self.beginning_of_word_position(selection.end),
                    context,
                ),
            },
        };
    }

    fn select_end_of_word(&mut self, _: &SelectEndOfWord, context: &mut ViewContext<Self>) {
        match self.edit_location.clone() {
            EditLocation::Cursor(cursor) => {
                self.select_to(self.end_of_word_position(cursor.position), context);
            }
            EditLocation::Selection(selection) => match selection.direction() {
                SelectionDirection::Backwards => {
                    self.select_to(self.end_of_word_position(selection.end), context)
                }
                SelectionDirection::Forwards => self.select(
                    selection.start,
                    self.end_of_word_position(selection.end),
                    context,
                ),
            },
        };
    }

    fn select_all(&mut self, _: &SelectAll, context: &mut ViewContext<Self>) {
        let start = self.beginning_of_file_position();
        let end = self.end_of_file_position();

        self.select(start, end, context);
    }

    fn remove_selection(&mut self, _: &RemoveSelection, context: &mut ViewContext<Self>) {
        if let EditLocation::Selection(selection) = self.edit_location.clone() {
            self.move_to(selection.start.clone(), selection.start.x, context);
        }
    }

    fn backspace(&mut self, _: &Backspace, context: &mut ViewContext<Self>) {
        match self.edit_location.clone() {
            EditLocation::Cursor(cursor) => {
                if cursor.position == self.beginning_of_file_position() {
                    return;
                }

                let line = self.buffer.line(cursor.position.y);

                match (line.clone().kind, cursor.position.x) {
                    (LineType::HeadlineStart(_level), 0) => {
                        let position = EditorPosition::new(cursor.position.y, line.beginning());
                        let range = position.clone()..cursor.position;

                        self.replace_range(range, "".into(), context);
                    }
                    (LineType::Normal, 0) => {
                        let soft_wrap_position = self.left_position(cursor.position.clone());
                        let position = self.left_position(soft_wrap_position.clone());
                        let range = position.clone()..cursor.position;

                        self.replace_range(range, "".into(), context);

                        let cursor_offset = self.buffer.position_to_offset(soft_wrap_position);
                        let new_cursor_position = self.buffer.offset_to_position(cursor_offset);

                        self.move_to(new_cursor_position, position.x, context);
                    }
                    _ => {
                        let position = self.left_position(cursor.position.clone());
                        let range = position.clone()..cursor.position;

                        self.replace_range(range, "".into(), context);

                        if position.x >= 0 {
                            self.move_to(position.clone(), position.x, context);
                        }
                    }
                };
            }
            EditLocation::Selection(selection) => {
                let smallest = selection.smallest();
                let range = smallest.clone()..selection.largest();

                self.replace_range(range, "".into(), context);
                let x = std::cmp::max(0, smallest.x);

                self.move_to(EditorPosition::new(smallest.y, x), x, context);
            }
        }
    }

    fn enter(&mut self, _: &Enter, context: &mut ViewContext<Self>) {
        let range = match self.edit_location.clone() {
            EditLocation::Cursor(cursor) => cursor.position.clone()..cursor.position,
            EditLocation::Selection(selection) => selection.smallest()..selection.largest(),
        };

        self.replace_range(range.clone(), "\n".into(), context);

        let y = range.end.y + 1;
        let line = self.buffer.line(y);
        let position = EditorPosition::new(y, line.beginning());

        self.move_to(position.clone(), position.x, context);
    }

    fn save(&mut self, _: &Save, context: &mut ViewContext<Self>) {
        // Check if the buffer has an associated file
        if !self.buffer.has_file() {
            // If there's no file, prompt for a save location
            self.prompt_to_save_file(context);
        } else {
            // If there is a file, save directly
            match self.buffer.save() {
                Ok(_) => {
                    context.notify();
                }
                Err(err) => {
                    // This shouldn't happen since we checked has_file(), but handle it anyway
                    let error_message = format!("Failed to save file: {:?}", err);
                    let error_prompt =
                        context.prompt(PromptLevel::Critical, &error_message, None, &["OK"]);

                    context
                        .foreground_executor()
                        .spawn(async move {
                            error_prompt.await.ok();
                        })
                        .detach();
                }
            }
        }
    }

    fn save_as(&mut self, _: &SaveAs, context: &mut ViewContext<Self>) {
        self.prompt_to_save_file(context);
    }

    fn copy(&mut self, _: &Copy, context: &mut ViewContext<Self>) {
        if let EditLocation::Selection(selection) = self.edit_location.clone() {
            let range = selection.smallest()..selection.largest();

            let text = self.read_range(range.clone());

            context.write_to_clipboard(ClipboardItem::new_string(text));
        }
    }

    fn cut(&mut self, _: &Cut, context: &mut ViewContext<Self>) {
        if let EditLocation::Selection(selection) = self.edit_location.clone() {
            let range = selection.smallest()..selection.largest();

            let text = self.read_range(range.clone());

            context.write_to_clipboard(ClipboardItem::new_string(text));
            self.replace_range(range, "".into(), context);
            self.move_to(selection.smallest(), selection.smallest().x, context);
        }
    }

    fn paste(&mut self, _: &Paste, context: &mut ViewContext<Self>) {
        let clipboard_item = context
            .read_from_clipboard()
            .unwrap_or(ClipboardItem::new_string("".into()));
        let range = match self.edit_location.clone() {
            EditLocation::Cursor(cursor) => cursor.position.clone()..cursor.position,
            EditLocation::Selection(selection) => selection.smallest()..selection.largest(),
        };
        let mut content = String::new();

        for entry in clipboard_item.entries() {
            if let gpui::ClipboardEntry::String(clipboard_string) = entry {
                content.push_str(clipboard_string.text());
            }
        }

        self.replace_range(range.clone(), content.clone(), context);

        let mut offset = self.buffer.position_to_offset(range.start);
        offset += content.len();
        let position = self.buffer.offset_to_position(offset);

        self.move_to(position.clone(), position.x, context);
    }

    fn move_to(
        &mut self,
        position: EditorPosition,
        preferred_x: isize,
        context: &mut ViewContext<Self>,
    ) {
        self.edit_location = EditLocation::Cursor(Cursor {
            position: position.clone(),
            preferred_x,
        });

        self.ensure_in_viewport(position);

        context.notify();
    }

    fn ensure_in_viewport(&mut self, position: EditorPosition) {
        let height = self.scroll_handle.bounds().size.height;
        let offset = self.scroll_handle.offset().y.abs();
        let current_line_offset = px(position.y as f32) * LINE_HEIGHT;
        let viewport = offset..height + offset;

        // Instead of scrolling right when we reach the edge
        // We'll do it when within the following margin
        let line_margin = LINE_HEIGHT * 3;

        if current_line_offset - line_margin < viewport.start - EDITOR_VERTICAL_MARGIN {
            self.scroll_to(-(current_line_offset + EDITOR_VERTICAL_MARGIN - line_margin));
        }

        if current_line_offset + LINE_HEIGHT + line_margin > viewport.end - EDITOR_VERTICAL_MARGIN {
            self.scroll_to(
                -(current_line_offset - height
                    + LINE_HEIGHT
                    + EDITOR_VERTICAL_MARGIN
                    + line_margin),
            );
        }
    }

    fn scroll_to(&mut self, y: Pixels) {
        self.scroll_handle.set_offset(Point::new(Pixels::ZERO, y));
    }

    fn select(
        &mut self,
        start: EditorPosition,
        end: EditorPosition,
        context: &mut ViewContext<Self>,
    ) {
        if start == end {
            self.move_to(start.clone(), start.x, context);
        } else {
            self.edit_location = EditLocation::Selection(Selection::new(start, end));
        }

        context.notify();
    }

    fn select_to(&mut self, end: EditorPosition, context: &mut ViewContext<Self>) {
        let start = match self.edit_location.clone() {
            EditLocation::Cursor(cursor) => cursor.position,
            EditLocation::Selection(selection) => selection.start,
        };

        self.ensure_in_viewport(end.clone());

        self.select(start, end, context);
    }

    fn read_range(&self, range: Range<EditorPosition>) -> String {
        let start_offset = self.buffer.position_to_offset(range.start);
        let end_offset = self.buffer.position_to_offset(range.end);
        let range = start_offset..end_offset;

        return self.buffer.read_range(range);
    }

    fn replace_range(
        &mut self,
        range: Range<EditorPosition>,
        replacement: String,
        context: &mut ViewContext<Self>,
    ) {
        let start_offset = self.buffer.position_to_offset(range.start);
        let end_offset = self.buffer.position_to_offset(range.end);

        self.buffer.replace(start_offset..end_offset, replacement);

        context.notify();
    }

    fn left_position(&self, point: EditorPosition) -> EditorPosition {
        let line = self.buffer.line(point.y);

        if point.y == 0 && point.x == line.beginning() {
            return point;
        }

        if point.x == line.beginning() {
            let new_line_index = point.y - 1;
            let line = self.buffer.line(new_line_index);

            return EditorPosition::new(new_line_index, line.end());
        }

        return EditorPosition::new(point.y, point.x - 1);
    }

    fn right_position(&self, point: EditorPosition) -> EditorPosition {
        let line_length = self.buffer.lines().len();
        let line = self.buffer.line(point.y);

        if point.y == line_length - 1 && point.x == line.end() {
            return point;
        }

        if point.x == line.end() {
            return EditorPosition::new(point.y + 1, 0);
        }

        return EditorPosition::new(point.y, point.x + 1);
    }

    fn up_position(&self, point: EditorPosition, preferred_x: isize) -> EditorPosition {
        if point.y == 0 {
            let line = self.buffer.line(0);
            return EditorPosition::new(0, line.beginning());
        }

        let previous_line = self.buffer.line(point.y - 1);

        let x = previous_line.clamp_x(preferred_x);

        return EditorPosition::new(point.y - 1, x);
    }

    fn down_position(&self, point: EditorPosition, preferred_x: isize) -> EditorPosition {
        let line = self.buffer.line(point.y);

        if point.y == self.buffer.lines().len() - 1 {
            return EditorPosition::new(point.y, line.end());
        }

        let next_line = self.buffer.line(point.y + 1);

        let x = next_line.clamp_x(preferred_x);

        return EditorPosition::new(point.y + 1, x);
    }

    fn beginning_of_file_position(&self) -> EditorPosition {
        let line = self.buffer.line(0);

        return EditorPosition::new(0, line.beginning());
    }

    fn end_of_file_position(&self) -> EditorPosition {
        let y = self.buffer.lines().len() - 1;
        let line = self.buffer.line(y);

        return EditorPosition::new(y, line.end());
    }

    fn beginning_of_line_position(&self, point: EditorPosition) -> EditorPosition {
        let line = self.buffer.line(point.y);

        return EditorPosition::new(point.y, line.beginning());
    }

    fn end_of_line_position(&self, point: EditorPosition) -> EditorPosition {
        let line = self.buffer.line(point.y);

        return EditorPosition::new(point.y, line.end());
    }

    fn beginning_of_word_position(&self, point: EditorPosition) -> EditorPosition {
        let line = self.buffer.line(point.y);
        let line_offset = (point.x - line.beginning()) as usize;

        // Handle edge case: at beginning of file
        if point.y == 0 && point.x <= line.beginning() {
            return self.beginning_of_file_position();
        }

        // First attempt: find previous word boundary in current line
        if point.x > line.beginning() && line_offset <= line.text.len() {
            let wrapped_text = WrappedText::new(line.text.clone());
            let word_boundary = wrapped_text.previous_word_boundary(line_offset);
            let new_x = line.beginning() + (word_boundary as isize);

            // Use this position if it's actually before the current position
            if new_x < point.x {
                return EditorPosition::new(point.y, new_x);
            }
        }

        // If we're here, we need to look at previous line

        // Handle edge case: already at first line
        if point.y == 0 {
            return self.beginning_of_file_position();
        }

        // Get previous line
        let previous_line = self.buffer.line(point.y - 1);

        // Handle edge case: previous line is empty
        if previous_line.text.trim().is_empty() {
            return EditorPosition::new(point.y - 1, previous_line.beginning());
        }

        // Find last word in previous line
        let wrapped_text = WrappedText::new(previous_line.text.clone());
        let last_valid_offset = previous_line.text.len();
        let word_boundary = wrapped_text.previous_word_boundary(last_valid_offset);
        let new_x = previous_line.beginning() + (word_boundary as isize);

        return EditorPosition::new(point.y - 1, new_x);
    }

    fn end_of_word_position(&self, point: EditorPosition) -> EditorPosition {
        let line = self.buffer.line(point.y);
        let line_offset = (point.x - line.beginning()) as usize;

        // First attempt: find next word boundary in current line
        if line_offset < line.text.len() {
            let wrapped_text = WrappedText::new(line.text.clone());

            if let Some(word_boundary) = wrapped_text.next_word_boundary(line_offset) {
                let new_x = line.beginning() + (word_boundary as isize);

                // Use this position if it doesn't exceed the end of the line
                if new_x <= line.end() {
                    return EditorPosition::new(point.y, new_x);
                }
            }
        }

        // If we're here, we need to look at the next line

        // Handle edge case: already at last line
        if point.y >= self.buffer.lines().len() - 1 {
            return EditorPosition::new(point.y, line.end());
        }

        // Get next line
        let next_line = self.buffer.line(point.y + 1);

        // Handle edge case: next line is empty
        if next_line.text.trim().is_empty() {
            return EditorPosition::new(point.y + 1, next_line.beginning());
        }

        // Find first non-whitespace character in next line
        let mut start_offset = 0;
        while start_offset < next_line.text.len()
            && next_line
                .text
                .chars()
                .nth(start_offset)
                .unwrap_or(' ')
                .is_whitespace()
        {
            start_offset += 1;
        }

        // If we reached the end of the next line
        if start_offset >= next_line.text.len() {
            return EditorPosition::new(point.y + 1, next_line.beginning());
        }

        // Go to the first word in the next line
        return EditorPosition::new(point.y + 1, next_line.beginning() + start_offset as isize);
    }
}

impl FocusableView for Editor {
    fn focus_handle(&self, _context: &AppContext) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl gpui::Render for Editor {
    fn render(&mut self, context: &mut gpui::ViewContext<Self>) -> impl gpui::IntoElement {
        div()
            .track_focus(&self.focus_handle(context))
            .key_context("editor")
            .on_action(context.listener(Self::new_file))
            .on_action(context.listener(Self::open_file))
            .on_action(context.listener(Self::save))
            .on_action(context.listener(Self::save_as))
            .on_action(context.listener(Self::set_buffer))
            .on_action(context.listener(Self::move_left))
            .on_action(context.listener(Self::move_right))
            .on_action(context.listener(Self::move_up))
            .on_action(context.listener(Self::move_down))
            .on_action(context.listener(Self::move_beginning_of_file))
            .on_action(context.listener(Self::move_end_of_file))
            .on_action(context.listener(Self::move_beginning_of_line))
            .on_action(context.listener(Self::move_end_of_line))
            .on_action(context.listener(Self::move_beginning_of_word))
            .on_action(context.listener(Self::move_end_of_word))
            .on_action(context.listener(Self::select_left))
            .on_action(context.listener(Self::select_right))
            .on_action(context.listener(Self::select_up))
            .on_action(context.listener(Self::select_down))
            .on_action(context.listener(Self::select_beginning_of_file))
            .on_action(context.listener(Self::select_end_of_file))
            .on_action(context.listener(Self::select_beginning_of_line))
            .on_action(context.listener(Self::select_end_of_line))
            .on_action(context.listener(Self::select_beginning_of_word))
            .on_action(context.listener(Self::select_end_of_word))
            .on_action(context.listener(Self::select_all))
            .on_action(context.listener(Self::remove_selection))
            .on_action(context.listener(Self::backspace))
            .on_action(context.listener(Self::enter))
            .on_action(context.listener(Self::copy))
            .on_action(context.listener(Self::cut))
            .on_action(context.listener(Self::paste))
            .group("editor-container")
            .w_full()
            .flex()
            .justify_center()
            .child(
                div()
                    .id("editor")
                    .w(CONTAINER_WIDTH)
                    .line_height(LINE_HEIGHT)
                    .py(EDITOR_VERTICAL_MARGIN)
                    .overflow_y_scroll()
                    .track_scroll(&self.scroll_handle)
                    .child(EditorElement {
                        input: context.view().clone(),
                    }),
            )
    }
}

struct EditorElement {
    input: View<Editor>,
}

impl ViewInputHandler for Editor {
    fn text_for_range(
        &mut self,
        range: Range<usize>,
        adjusted_range: &mut Option<Range<usize>>,
        cx: &mut ViewContext<Self>,
    ) -> Option<String> {
        println!("Text for range");
        todo!();
    }

    fn selected_text_range(
        &mut self,
        ignore_disabled_input: bool,
        cx: &mut ViewContext<Self>,
    ) -> Option<gpui::UTF16Selection> {
        println!("Selected text range");
        todo!();
    }

    fn marked_text_range(&self, cx: &mut ViewContext<Self>) -> Option<Range<usize>> {
        None
    }

    fn unmark_text(&mut self, cx: &mut ViewContext<Self>) {
        println!("Unmark text");
        todo!()
    }

    fn replace_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        context: &mut ViewContext<Self>,
    ) {
        // If no range is provided, use the current selection or cursor position
        let range = if let Some(range) = range {
            let start = self.buffer.offset_to_position(range.start);
            let end = self.buffer.offset_to_position(range.end);

            start..end
        } else {
            match &self.edit_location {
                EditLocation::Cursor(cursor) => cursor.position.clone()..cursor.position.clone(),
                EditLocation::Selection(selection) => selection.smallest()..selection.largest(),
            }
        };

        self.replace_range(range.clone(), text.to_string(), context);

        // Handle case where a new headline is being created with ' '
        if let EditLocation::Cursor(cursor) = self.edit_location.clone() {
            let line = self.buffer.line(cursor.position.y);

            if let LineType::HeadlineStart(level) = line.kind {
                // The reason it's not level + 1 is that:
                // Cursor position.x is 0-indexed
                // While level is not
                if cursor.position.x == (level as isize) && text == " " {
                    self.move_to(EditorPosition::new(cursor.position.y, 0), 0, context);
                    return;
                }
            }
        }

        let mut offset = self.buffer.position_to_offset(range.start.clone()) + text.len();
        let mut end_position = self.buffer.offset_to_position(offset);

        // If the inserts have caused a soft-wrap
        // then we need to adjust the offset to account for the extra whitespace characters
        if end_position.y > range.start.y {
            offset += end_position.y - range.start.y;
            end_position = self.buffer.offset_to_position(offset);
        }

        self.move_to(end_position.clone(), end_position.x, context);
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        new_text: &str,
        new_selected_range: Option<Range<usize>>,
        cx: &mut ViewContext<Self>,
    ) {
        println!("Replace and mark text in range");
        todo!()
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        element_bounds: Bounds<Pixels>,
        cx: &mut ViewContext<Self>,
    ) -> Option<Bounds<Pixels>> {
        println!("Bounds for range");
        todo!()
    }
}

impl IntoElement for EditorElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

#[derive(Debug, Clone)]
struct RenderedLine {
    shaped_line: ShapedLine,
    raw_line: Line,
}

impl RenderedLine {
    pub fn new(raw_line: Line, shaped_line: ShapedLine) -> RenderedLine {
        return RenderedLine {
            raw_line,
            shaped_line,
        };
    }
}

struct PrepaintState {
    lines: Vec<RenderedLine>,
    edit_location_rectangles: Vec<PaintQuad>,
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
        let input = self.input.read(context);
        let content = input.buffer.content();
        let lines = content.lines();

        let style = Style::default();
        let new_style = Style {
            size: gpui::Size {
                width: gpui::Length::Auto,
                height: gpui::Length::Definite(gpui::DefiniteLength::Absolute(
                    gpui::AbsoluteLength::Pixels(px(lines.len() as f32) * LINE_HEIGHT),
                )),
            },
            ..style
        };

        (context.request_layout(new_style, []), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&gpui::GlobalElementId>,
        bounds: gpui::Bounds<gpui::Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        context: &mut gpui::WindowContext,
    ) -> Self::PrepaintState {
        let input = self.input.read(context);
        let content = input.buffer.content();
        let style = context.text_style();
        let font_size = style.font_size.to_pixels(context.rem_size());
        let is_focused = input.focus_handle.is_focused(context);

        let mut lines: Vec<RenderedLine> = vec![];
        let raw_lines = content.lines();

        for line in &raw_lines {
            let run = match line.kind {
                LineType::HeadlineStart(_) => TextRun {
                    len: line.length(),
                    font: Font {
                        weight: FontWeight::EXTRA_BOLD,
                        ..style.font()
                    },
                    color: Hsla::from(rgb(COLOR_GRAY_800)),
                    background_color: None,
                    underline: None,
                    strikethrough: None,
                },
                LineType::HeadlineNotStart => TextRun {
                    len: line.length(),
                    font: Font {
                        weight: FontWeight::EXTRA_BOLD,
                        ..style.font()
                    },
                    color: Hsla::from(rgb(COLOR_GRAY_800)),
                    background_color: None,
                    underline: None,
                    strikethrough: None,
                },
                LineType::Normal => TextRun {
                    len: line.length(),
                    font: style.font(),
                    color: Hsla::from(rgb(COLOR_GRAY_700)),
                    background_color: None,
                    underline: None,
                    strikethrough: None,
                },
            };
            let runs = vec![run];

            let shaped_line = context
                .text_system()
                .shape_line(SharedString::from(line.text.clone()), font_size, &runs)
                .unwrap();

            lines.push(RenderedLine::new(line.clone(), shaped_line));
        }

        let edit_location_rectangles = match input.edit_location.clone() {
            EditLocation::Cursor(cursor) => {
                let left = bounds.left()
                    + EDITOR_HORIZONTAL_MARGIN
                    + px(cursor.position.x as f32) * CHARACTER_WIDTH
                    - px(1.);
                let top =
                    bounds.top() + context.line_height() * px(cursor.position.y as f32) + px(2.);

                let color = if is_focused {
                    rgb(COLOR_BLUE_DARK)
                } else {
                    rgb(COLOR_GRAY_400)
                };
                let rectangles = vec![fill(
                    Bounds::new(point(left, top), size(px(2.), px(20.))),
                    color,
                )];

                rectangles
            }
            EditLocation::Selection(selection) => {
                let mut rectangles = vec![];

                let smallest = selection.smallest();
                let largest = selection.largest();

                let line_range = smallest.y..largest.y + 1;

                for index in line_range.clone() {
                    let start = if index == line_range.start {
                        smallest.x
                    } else {
                        std::cmp::min(0, smallest.x)
                    };
                    let end = if index == line_range.end - 1 {
                        largest.x
                    } else {
                        CHARACTER_COUNT_PER_LINE as isize
                    };

                    let left = bounds.left()
                        + EDITOR_HORIZONTAL_MARGIN
                        + px(start as f32) * CHARACTER_WIDTH
                        - px(1.);
                    let top = bounds.top() + px(index as f32) * context.line_height();
                    let width = px((end - start) as f32) * CHARACTER_WIDTH + px(2.);

                    let color = if is_focused {
                        rgb(COLOR_BLUE_MEDIUM)
                    } else {
                        rgb(COLOR_GRAY_300)
                    };
                    let bounds = Bounds::new(point(left, top), size(width, context.line_height()));
                    let rectangle = fill(bounds, color);

                    rectangles.push(rectangle);
                }

                rectangles
            }
        };

        PrepaintState {
            lines,
            edit_location_rectangles,
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
        let edit_location_rectangles = prepaint.edit_location_rectangles.clone();
        let lines = prepaint.lines.clone();

        context.handle_input(
            &focus_handle,
            ElementInputHandler::new(bounds, self.input.clone()),
        );

        for rectangle in edit_location_rectangles {
            context.paint_quad(rectangle);
        }

        for (index, line) in lines.iter().enumerate() {
            let offset = match line.raw_line.kind {
                LineType::HeadlineStart(level) => {
                    EDITOR_HORIZONTAL_MARGIN - px(level as f32 + 1 as f32) * CHARACTER_WIDTH
                }
                LineType::HeadlineNotStart => EDITOR_HORIZONTAL_MARGIN,
                LineType::Normal => EDITOR_HORIZONTAL_MARGIN,
            };

            let point = Point::new(
                bounds.origin.x + offset,
                bounds.origin.y + (context.line_height() * index),
            );

            line.shaped_line
                .paint(point, context.line_height(), context)
                .unwrap();
        }
    }
}

fn get_documents_folder_path() -> PathBuf {
    let home = env::var("HOME").unwrap_or_else(|_| String::from("/Users/Shared"));

    return PathBuf::from(home).join("Documents");
}
