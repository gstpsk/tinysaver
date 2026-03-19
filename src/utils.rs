use image::GenericImageView;

// returns a random valid (x, y) pair
pub fn get_random_position(surface_width: i32, surface_height: i32) -> (i32, i32) {
    let mut result: (i32, i32) = (0 ,0);
    
    result.0 = rand::random::<i32>() % surface_width;
    result.1 = rand::random::<i32>() % surface_height;

    result.0 = result.0.abs();
    result.1 = result.1.abs();

    result
}

pub fn load_image_rgba(path: &str) -> (Vec<u8>, u32, u32) {
    let img = image::open(path).expect("Failed to load image");
    let rgba = img.to_rgba8();
    let (width, height) = img.dimensions();
    (rgba.into_raw(), width, height)
}