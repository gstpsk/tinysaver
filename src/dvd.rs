use crate::utils;
use crate::draw;
use crate::color;

// invert speed if the image exceeds surface width after computation
fn invert_xy_speed(dvd_state: &mut DvdState) {
    // top corners
    if (dvd_state.x as i32 + dvd_state.img_width as i32 + dvd_state.speed_x as i32) >= dvd_state.surface_width as i32 || (dvd_state.x as i32 + dvd_state.speed_x as i32) <= 0 {
        dvd_state.speed_x = -dvd_state.speed_x;
    }

    // bottom corners
    if (dvd_state.y as i32 + dvd_state.img_height as i32 + dvd_state.speed_y as i32) >= dvd_state.surface_height as i32 || (dvd_state.y as i32 + dvd_state.speed_y as i32) <= 0 {
        dvd_state.speed_y = -dvd_state.speed_y;
    }

}

pub struct DvdState {
    pub x: i32,
    pub y: i32,
    pub img_data: Vec<u8>,
    pub img_width: i32,
    pub img_height: i32,
    pub speed_x: i8,
    pub speed_y: i8,
    pub surface_width: i32,
    pub surface_height: i32,
}

impl DvdState {
    pub fn with_random_position(initial_speed: i8, surface_width: i32, surface_height: i32) -> Self {
        let (x, y) = utils::get_random_position(surface_width, surface_height);
        let (img_data, img_width, img_height) = utils::load_image_rgba("arch25percent.png");
        
        println!("initial dvd location x: {x}, y: {y}");

        Self {
            x: x,
            y: y,
            img_data: img_data,
            img_width: img_width as i32,
            img_height: img_height as i32,
            speed_x: initial_speed,
            speed_y: initial_speed,
            surface_width: surface_width,
            surface_height         
        }
    }

    fn update_position(&mut self) {
        invert_xy_speed(self);
        self.x = self.x + self.speed_x as i32;
        self.y = self.y + self.speed_y as i32;
    }

    pub fn increase_speed_by(&mut self, amount: i8) {
            if self.speed_x >= 0 { self.speed_x += amount; } else { self.speed_x -= amount; }
            if self.speed_y >= 0 { self.speed_y += amount; } else { self.speed_y -= amount; }
    }

    pub fn decrease_speed_by(&mut self, amount: i8) {
            if self.speed_x >= 0 { self.speed_x -= amount; } else { self.speed_x += amount; }
            if self.speed_y >= 0 { self.speed_y -= amount; } else { self.speed_y += amount; }
    }
}

pub fn dvd_style(frame: &mut [u8], frame_count: u32, dvd_state: &mut DvdState) {
    // store previous position
    let old_x = dvd_state.x;
    let old_y = dvd_state.y;

    dvd_state.update_position();

    let black: (u8,u8,u8,u8) = (0,0,0,0);
    let color = color::rainbow_rgba(frame_count);
    let rgb_color = (color.0, color.1, color.2);
    
    // erase old
    //draw::draw_rect(frame, old_x, old_y, dvd_state.img_width, dvd_state.img_height, 0, None, true, color, dvd_state.surface_width, dvd_state.surface_height);
    draw::clear_image(frame, &mut dvd_state.img_data, old_x, old_y, dvd_state.img_width, dvd_state.img_height, dvd_state.surface_width, dvd_state.surface_height);
    
    color::color_image(&mut dvd_state.img_data, rgb_color);
    draw::draw_image(frame, &dvd_state.img_data, dvd_state.x, dvd_state.y, dvd_state.img_width, dvd_state.img_height, dvd_state.surface_width, dvd_state.surface_height);
    //draw_rect(frame, *dvd_x, *dvd_y, img_width, img_height, 0, None, true, color, surface_width, surface_height);

}