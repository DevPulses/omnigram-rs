/// Style data types mirroring tdesktop's structure_types.h and style_core_types.h

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TypeTag {
    Int,
    Bool,
    Double,
    Pixels,
    String,
    Color,
    Point,
    Size,
    Align,
    Margins,
    Font,
    Icon,
    Struct,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Size {
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Margins {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
    pub fallback: Option<String>,
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct FontFlags: u32 {
        const BOLD = 0x01;
        const ITALIC = 0x02;
        const UNDERLINE = 0x04;
        const SEMIBOLD = 0x08;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Font {
    pub family: String,
    pub size: i32,
    pub flags: FontFlags,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Align {
    #[default]
    TopLeft,
    Top,
    TopRight,
    Right,
    BottomRight,
    Bottom,
    BottomLeft,
    Left,
    Center,
    Justify,
}

impl Align {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "topleft" => Some(Align::TopLeft),
            "top" => Some(Align::Top),
            "topright" => Some(Align::TopRight),
            "right" => Some(Align::Right),
            "bottomright" => Some(Align::BottomRight),
            "bottom" => Some(Align::Bottom),
            "bottomleft" => Some(Align::BottomLeft),
            "left" => Some(Align::Left),
            "center" => Some(Align::Center),
            "justify" => Some(Align::Justify),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MonoIcon {
    pub filename: String,
    pub color: String, // Color variable name
    pub padding: Margins,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Icon {
    pub parts: Vec<MonoIcon>,
}
