// returns a color based on the step value given
pub fn rainbow_rgba(step: u32) -> (u8,u8,u8,u8) {
    let steps = 255 * 6;
    let speed = 1;
    
    let step = step * speed;

    if (step % steps) < 255 {
        (255, (step % 255) as u8, 0, 255)
    } else if (step % steps) >= 255 && (step % steps) < 255 * 2 {
        (255 - (step % 255) as u8, 255, 0, 255)
    } else if (step % steps) >= 255 * 2 && (step % steps) < 255 * 3 {
        (0, 255, (step % 255) as u8, 255)
    } else if (step % steps) >= 255 * 3 && (step % steps) < 255 * 4 {
        (0, 255 - (step % 255) as u8, 255, 255)
    } else if (step % steps) >= 255 * 4 && (step % steps) < 255 * 5 {
        ((step % 255) as u8, 0, 255, 255)
    } else if (step % steps) >= 255 * 5 {
        (255, 0, 255 - (step % 255) as u8, 255)
    } else {
        panic!("None match? Step {step}/{steps}");
    }
}

pub fn hsv_to_rgb() {
    // to be implemented
    // allows us to map a range to the rgb values
    // so we could have 12 total steps, which would generate a very coarse rainbow
    // or we could have 10000 steps, which would generate a very smooth rainbow
    // the amount of unique colors in this step process would be 255*6=1530
}

pub fn color_image(img_data: &mut [u8], color: (u8, u8, u8)) {
    for px in img_data.chunks_mut(4) {
        let alpha = px[3];

        // skip transparent pixels
        if alpha == 0 {
            continue;
        }

        // change pixel color
        px[0] = color.0;
        px[1] = color.1;
        px[2] = color.2;
    }
}