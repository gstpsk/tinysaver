use font_kit::{canvas::{Canvas, Format, RasterizationOptions}, font::Font, hinting::HintingOptions};
use pathfinder_geometry::{transform2d::Transform2F, vector::{Vector2F, Vector2I}};

use crate::color::color_image;

pub fn set_pixel_at(
    frame: &mut [u8],
    x: i32,
    y: i32,
    surface_width: i32,
    surface_height: i32,
    rgba: (u8, u8, u8, u8),
) {
    // reject negative coordinates as they are off screen
    if x < 0 || y < 0 {
        return;
    }

    // reject coordinates bigger than the surface
    if x >= surface_width || y >= surface_height {
        return;
    }

    // safely cast to usize because y and x are positive
    let index = ((y * surface_width + x) * 4) as usize;
    
    frame[index]     = rgba.0;
    frame[index + 1] = rgba.1;
    frame[index + 2] = rgba.2;
    frame[index + 3] = rgba.3;
}

pub fn draw_rect(
    frame: &mut [u8],
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    border_width: i32,
    border_color: Option<(u8,u8,u8,u8)>,
    fill: bool,
    fill_color: (u8,u8,u8,u8),
    surface_width: i32,
    surface_height: i32
) {
    let left_bound = x - border_width;
    let right_bound = x + width + border_width;
    let top_bound = y - border_width;
    let bottom_bound = y + height + border_width;

    let draw_x0 = left_bound.max(0);
    let draw_y0 = top_bound.max(0);
    let draw_x_end = right_bound.min(surface_width);
    let draw_y_end = bottom_bound.min(surface_height);


    for px in draw_x0..draw_x_end {
        for py in draw_y0..draw_y_end {           
            // if the px is being drawn in the range of the border, draw the border
            let in_left_border   = px >= x - border_width && px < x;
            let in_right_border  = px >= x + width && px < x + width + border_width;

            let in_top_border    = py >= y - border_width && py < y;
            let in_bottom_border = py >= y + height && py < y + height + border_width;

            let in_border = in_left_border || in_right_border || in_top_border || in_bottom_border;

            if in_border && border_width > 0 {
                set_pixel_at(frame, px, py, surface_width, surface_height, border_color.expect("Expected valid border color but got None!"));
            }

            if fill && !in_border {
                set_pixel_at(frame, px, py, surface_width, surface_height, fill_color);
            }
        }
    }

}

fn try_compute_frame_index_from_position(frame: &mut [u8], x: i32, y: i32, surface_width: i32, surface_height: i32) -> Option<usize> {
    if x < 0 || y < 0 || x >= surface_width || y >= surface_height {
        eprintln!("This is off screen mate, got position ({x}, {y}) with screen size {surface_width}x{surface_height}");
        return None;
    }

    let pixel_index = y * surface_width + x;
    let byte_index = pixel_index * 4;

    if byte_index as usize >= frame.len() {
        let frame_size = frame.len();
        eprintln!("Got byte_index {byte_index} bigger than the frame size {frame_size}...");
        return None;
    }
    
    Some(byte_index as usize)
}

pub fn draw_image(frame: &mut [u8], img_data: &[u8], x: i32, y: i32, img_width: i32, img_height: i32, surface_width: i32, surface_height: i32) {
    for row in 0..img_height {
        let image_row_start = (row * img_width * 4) as usize;
        let image_row_end = (( (row * img_width) + img_width ) * 4) as usize;

        //println!("attempting to draw image at {x}, {y}");

        let frame_start_index = try_compute_frame_index_from_position(frame, x, y+row, surface_width, surface_height).expect("Expected valid conversion of position to frame index but got None!");
        let frame_end_index = try_compute_frame_index_from_position(frame, x+img_width, y+row, surface_width, surface_height).expect("Expected valid conversion of position to frame index but got None!");


        //println!("gonna copy from image data {image_row_start} until {image_row_end} to frame {frame_start_index} until {frame_end_index}");

        frame[frame_start_index..frame_end_index].copy_from_slice(&img_data[image_row_start..image_row_end]);
    }
}

pub fn clear_image(frame: &mut [u8], img_data: &mut [u8], x: i32, y: i32, img_width: i32, img_height: i32, surface_width: i32, surface_height: i32) {
    color_image(img_data, (0, 0, 0));
    draw_image(frame, img_data, x, y, img_width, img_height, surface_width, surface_height);
}

pub fn draw_glyph(
    frame: &mut [u8],
    glyph_data: &[u8],
    color: (u8, u8, u8),
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    surface_width: i32,
    surface_height: i32,
) {
    for draw_y in 0..height {
        for draw_x in 0..width {
            // compute destination coords.
            let dst_x = x + draw_x;
            let dst_y = y + draw_y;

            // skip pixels outside framebuffer
            if dst_x < 0 || dst_y < 0 || dst_x >= surface_width || dst_y >= surface_height {
                continue;
            }

            
            // get alpha from glyph pixel
            let glyph_index = (draw_y * width + draw_x) as usize;
            let alpha = glyph_data[glyph_index];
            
            if alpha > 0 {
                let rgba = (color.0, color.1, color.2, alpha);
                set_pixel_at(frame, dst_x, dst_y, surface_width, surface_height, rgba);
            }

        }
    }
}


pub fn draw_string(frame: &mut [u8], string: &str, font: &Font, color: (u8, u8, u8), x: i32, y: i32, surface_width: i32, surface_height: i32) {
    for (i, char) in string.chars().enumerate() {
            
            let glyph_id = font.glyph_for_char(char).expect("Expected valid glyph_id for character but got None!");
            
            let mut canvas = Canvas::new(Vector2I::new(64, 64), Format::A8);
            
            font.rasterize_glyph(
                &mut canvas,
                glyph_id,
                32.0,
                Transform2F::from_translation(Vector2F::new(0.0, 32.0)),
                HintingOptions::None,
                RasterizationOptions::GrayscaleAa,
            ).unwrap();

            // calculate x based on character index
            let px = x + (i as i32 * 64);

            draw_glyph(frame, &canvas.pixels, color, px, y, canvas.size.x(), canvas.size.y(), surface_width, surface_height);
    }
    
}