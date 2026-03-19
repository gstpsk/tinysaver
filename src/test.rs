// some unit test practice
// dont take these too seriously...
#[cfg(test)]
mod tests {
    use std::any::Any;

    use crate::{color::rainbow_rgba, draw::set_pixel_at};

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_rainbow_rgba() {
        let black: (u8,u8,u8,u8) = (0,0,0,0);
        assert!(rainbow_rgba(u32::max_value()).type_id() == black.type_id());
        assert!(rainbow_rgba(0).type_id() == black.type_id());
    }
    #[test]
    fn set_pixel_negative_coordinates() {
        let mut buf = vec![0; 4]; // create buffer of 1 blank pixel
        let before = buf.clone();

        set_pixel_at(&mut buf, -1, -1, 1, 1, (255, 255, 255, 255));

        assert_eq!(buf, before);
    }

    #[test]
    fn set_pixel_ignores_negative_dimensions() {
        let mut buf = Vec::new();
        buf.resize(4, 0);
        set_pixel_at(&mut buf, 1, 1, -1, 1, (255, 255, 255, 255));
        assert!(buf.is_empty());
    }

    #[test]
    fn set_pixel_ignores_negative_width_and_height() {
        let mut buf = Vec::new();
        buf.resize(4, 0);
        set_pixel_at(&mut buf, -1, -1, -1, -1, (255, 255, 255, 255));
        assert!(buf.is_empty());
    }

    #[test]
    fn set_pixel_writes_valid_pixel() {
        let mut buf = Vec::new();
        buf.resize(4, 0);
        set_pixel_at(&mut buf, 1, 1, 1, 1, (255, 255, 255, 255));
        assert!(!buf.is_empty());
    }

}