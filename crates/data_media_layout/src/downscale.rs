use data_media_types::Size;

pub const DEFAULT_MAX_MEDIA_WIDTH: i32 = 430;
pub const DEFAULT_MAX_MEDIA_HEIGHT: i32 = 430;
pub const DEFAULT_MIN_MEDIA_SIZE: i32 = 100;

#[must_use]
pub fn non_empty_size(size: Size) -> Size {
    Size::new(size.width.max(1), size.height.max(1))
}

#[must_use]
pub fn downscaled_size(size: Size, box_size: Size) -> Size {
    let size = non_empty_size(size);
    if size.width <= box_size.width && size.height <= box_size.height {
        return size;
    }

    let w_ratio = box_size.width as f64 / size.width as f64;
    let h_ratio = box_size.height as f64 / size.height as f64;
    let ratio = w_ratio.min(h_ratio);

    let new_w = (size.width as f64 * ratio).round() as i32;
    let new_h = (size.height as f64 * ratio).round() as i32;

    non_empty_size(Size::new(new_w, new_h))
}

#[must_use]
pub fn clamp_media_size(size: Size, max_box: Size, min_side: i32) -> Size {
    let downscaled = downscaled_size(size, max_box);
    Size::new(
        downscaled.width.max(min_side),
        downscaled.height.max(min_side),
    )
}
