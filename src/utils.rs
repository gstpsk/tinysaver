use image::GenericImageView;

// returns a random valid (x, y) pair
pub fn get_random_position(max_x: i32, max_y: i32) -> (i32, i32) {
    // use u32 to ensure non-negative numbers
    let x = rand::random::<u32>() % (max_x as u32);
    let y = rand::random::<u32>() % (max_y as u32);
    (x as i32, y as i32)
}

pub fn load_image_rgba8(path: &str) -> (Vec<u8>, u32, u32) {
    let img = image::open(path).expect("Failed to load image");
    let rgba = img.to_rgba8();
    let (width, height) = img.dimensions();
    (rgba.into_raw(), width, height)
}