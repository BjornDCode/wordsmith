use gpui::SharedString;

#[derive(Clone)]
pub struct Text {
    content: SharedString,
}

impl Text {
    pub fn new(content: impl Into<SharedString>) -> Text {
        Text {
            content: content.into(),
        }
    }

    pub fn to_string(&self) -> String {
        self.content.to_string()
    }
}

impl Text {
    pub fn lines(&self) -> std::str::Lines<'_> {
        self.content.lines()
    }
}
