use image::GenericImageView;

// returns a random (x, y) pair within the given range
pub fn get_random_position(min_x: u32, max_x: u32, min_y: u32, max_y: u32) -> (u32, u32) {
    let x = rand::random_range(min_x..max_x);
    let y = rand::random_range(min_y..max_y);
    (x, y)
}

pub fn load_image_rgba8(path: &str) -> (Vec<u8>, u32, u32) {
    let img = image::open(path).expect("Failed to load image");
    let rgba = img.to_rgba8();
    let (width, height) = img.dimensions();
    (rgba.into_raw(), width, height)
}

pub fn pixel_to_clip_matrix(width: u32, height: u32) -> [[f32; 4]; 4] {
    let w = width as f32;
    let h = height as f32;

    [
        [ 2.0 / w, 0.0,      0.0, 0.0 ], // column 0
        [ 0.0,    -2.0 / h,  0.0, 0.0 ], // column 1
        [ 0.0,     0.0,      1.0, 0.0 ], // column 2
        [ -1.0,    1.0,      0.0, 1.0 ], // column 3
    ]
    
}