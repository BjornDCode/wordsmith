use std::cmp::Ordering;

#[derive(Debug, Clone)]
pub enum EditLocation {
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
pub struct Cursor {
    pub position: EditorPosition,
    pub preferred_x: isize,
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
pub struct Selection {
    pub start: EditorPosition,
    pub end: EditorPosition,
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
pub enum SelectionDirection {
    Backwards,
    Forwards,
}
