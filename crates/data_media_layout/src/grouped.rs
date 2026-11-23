use data_media_types::Size;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self { x, y, width, height }
    }
}

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct RectSides: u8 {
        const LEFT   = 0b0001;
        const TOP    = 0b0010;
        const RIGHT  = 0b0100;
        const BOTTOM = 0b1000;
        const ALL    = 0b1111;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupMediaLayout {
    pub geometry: Rect,
    pub sides: RectSides,
}

pub fn layout_playlist(sizes: &[Size]) -> Vec<GroupMediaLayout> {
    if sizes.is_empty() {
        return Vec::new();
    }

    let max_width = sizes.iter().map(|s| s.width).max().unwrap_or(0);
    let mut result = Vec::with_capacity(sizes.len());
    let mut top = 0;

    for (i, size) in sizes.iter().enumerate() {
        let mut sides = RectSides::LEFT | RectSides::RIGHT;
        if i == 0 {
            sides |= RectSides::TOP;
        }
        if i == sizes.len() - 1 {
            sides |= RectSides::BOTTOM;
        }

        result.push(GroupMediaLayout {
            geometry: Rect::new(0, top, max_width, size.height),
            sides,
        });

        top += size.height;
    }

    result
}

pub fn layout_grid_2x2(sizes: &[Size], max_width: i32) -> Vec<GroupMediaLayout> {
    let half_w = max_width / 2;
    let mut result = Vec::new();

    for (i, _) in sizes.iter().enumerate().take(4) {
        let col = i % 2;
        let row = i / 2;
        let x = col as i32 * half_w;
        let y = row as i32 * half_w; // square cells in basic 2x2 grid

        let mut sides = RectSides::empty();
        if col == 0 { sides |= RectSides::LEFT; }
        if col == 1 { sides |= RectSides::RIGHT; }
        if row == 0 { sides |= RectSides::TOP; }
        if row == 1 { sides |= RectSides::BOTTOM; }

        result.push(GroupMediaLayout {
            geometry: Rect::new(x, y, half_w, half_w),
            sides,
        });
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::downscale::*;

    #[test]
    fn test_downscale() {
        let orig = Size::new(1920, 1080);
        let max_box = Size::new(430, 430);
        let scaled = downscaled_size(orig, max_box);

        assert!(scaled.width <= 430);
        assert!(scaled.height <= 430);
        assert_eq!(scaled.width, 430);
        assert_eq!(scaled.height, 242);
    }

    #[test]
    fn test_layout_playlist() {
        let items = vec![Size::new(300, 50), Size::new(320, 60), Size::new(280, 55)];
        let layout = layout_playlist(&items);

        assert_eq!(layout.len(), 3);
        assert_eq!(layout[0].geometry, Rect::new(0, 0, 320, 50));
        assert!(layout[0].sides.contains(RectSides::TOP));
        assert_eq!(layout[1].geometry, Rect::new(0, 50, 320, 60));
        assert_eq!(layout[2].geometry, Rect::new(0, 110, 320, 55));
        assert!(layout[2].sides.contains(RectSides::BOTTOM));
    }
}
