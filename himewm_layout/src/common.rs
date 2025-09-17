use serde::{Deserialize, Serialize};
use windows::Win32::Foundation::*;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Position(i32, i32, i32, i32);

impl From<RECT> for Position {
    fn from(value: RECT) -> Self {
        Self(
            value.left,
            value.top,
            value.right - value.left,
            value.bottom - value.top,
        )
    }
}

impl From<(i32, i32, i32, i32)> for Position {
    fn from(value: (i32, i32, i32, i32)) -> Self {
        Self(value.0, value.1, value.2, value.3)
    }
}

impl Position {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self(x, y, w, h)
    }

    pub fn x(&self) -> i32 {
        self.0
    }

    pub fn set_x(&mut self, val: i32) {
        self.0 = val;
    }

    pub fn y(&self) -> i32 {
        self.1
    }

    pub fn set_y(&mut self, val: i32) {
        self.1 = val;
    }

    pub fn w(&self) -> i32 {
        self.2
    }

    pub fn set_w(&mut self, val: i32) {
        self.2 = val;
    }

    pub fn h(&self) -> i32 {
        self.3
    }

    pub fn set_h(&mut self, val: i32) {
        self.3 = val;
    }
}
