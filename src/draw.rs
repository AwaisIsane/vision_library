use crate::{
    harris_corner::{harris_corner_detector, Descriptor},
    io::Image,
    panorma::{match_descriptors, Match},
};

// Helper: Stitches two images side-by-side
pub fn both_images(a: &Image, b: &Image) -> Image {
    let width = a.width + b.width;
    let height = a.height.max(b.height); // Height is the max of the two
    let channels = a.channels; // Assuming both have same channels (usually 3)

    // 1. Create a blank canvas (black)
    // We use map to create a vector of the correct size filled with 0.0
    let data = vec![0.0; (width * height * channels) as usize];
    let mut both = Image::new(width, height, channels, data);

    // 2. Copy Image A (Left side)
    for y in 0..a.height {
        for x in 0..a.width {
            let val = a.get_pixel(x as i32, y as i32);
            both.put_pixel(x, y, val.to_vec());
        }
    }

    // 3. Copy Image B (Right side)
    // We shift the x-coordinate by a.width
    for y in 0..b.height {
        for x in 0..b.width {
            let val = b.get_pixel(x as i32, y as i32);
            both.put_pixel(x + a.width, y, val.to_vec());
        }
    }

    both
}

// Helper: Bresenham's Line Algorithm
// The C code used a simple math formula (y = mx + b), but that leaves gaps
// if the line is steep. This algorithm draws solid lines in any direction.
pub fn draw_line(img: &mut Image, x0: i32, y0: i32, x1: i32, y1: i32, color: &[f32]) {
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    let mut x = x0;
    let mut y = y0;

    loop {
        // Draw the pixel if it is inside bounds
        if x >= 0 && x < img.width as i32 && y >= 0 && y < img.height as i32 {
            img.put_pixel(x as u32, y as u32, color.to_vec());
        }

        if x == x1 && y == y1 {
            break;
        }

        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}
// Assuming you have the struct Match from the previous step
// and the descriptors list to look up coordinates.
pub fn draw_matches(
    a: &Image,
    b: &Image,
    matches: &[Match],
    a_descriptors: &[Descriptor], // Needed to find Point A coordinates
    b_descriptors: &[Descriptor], // Needed to find Point B coordinates
) -> Image {
    // 1. Create a canvas with both images stitched together
    let mut both = both_images(a, b);

    // 2. Loop through every match
    for m in matches {
        // USE THE INDICES TO LOOK UP THE COORDINATES
        // "m.a_index" is just a number. We use it to get the actual Descriptor.
        let p_a = &a_descriptors[m.a_index];
        let p_b = &b_descriptors[m.b_index];

        let x1 = p_a.x as i32;
        let y1 = p_a.y as i32;

        // Shift x2 by the width of image A so it lands on the right side
        let x2 = (p_b.x + a.width) as i32;
        let y2 = p_b.y as i32;

        // Draw a green line for every match
        let green = vec![0.0, 255.0, 0.0];
        draw_line(&mut both, x1, y1, x2, y2, &green);
    }

    both
}
/// Draws a + sign at each point in `points` on the image.
/// If `value` is None, it defaults to red for RGB/RGBA and black for grayscale/luma.
pub fn draw_plus_on_image(
    img: &mut Image,
    points: &[(u32, u32)],
    size: u32,
    value: Option<&[f32]>,
) {
    let width = img.width as i32;
    let height = img.height as i32;
    let channels = img.channels as usize;

    // Determine the color to use
    let default_value: Vec<f32> = match value {
        Some(v) => {
            assert_eq!(
                v.len(),
                channels,
                "Value array length must match image channels"
            );
            v.to_vec()
        }
        None => {
            match channels {
                1 => vec![0.0],                    // black for grayscale
                3 => vec![255.0, 0.0, 0.0],        // red for RGB
                4 => vec![255.0, 0.0, 0.0, 255.0], // red + full alpha for RGBA
                _ => vec![255.0; channels],        // fallback: white for other channels
            }
        }
    };

    for &(x, y) in points {
        let x = x as i32;
        let y = y as i32;

        // Horizontal line
        for dx in -(size as i32)..=(size as i32) {
            let nx = x + dx;
            if nx >= 0 && nx < width && y >= 0 && y < height {
                img.put_pixel(nx as u32, y as u32, default_value.clone());
            }
        }

        // Vertical line
        for dy in -(size as i32)..=(size as i32) {
            let ny = y + dy;
            if ny >= 0 && ny < height && x >= 0 && x < width {
                img.put_pixel(x as u32, ny as u32, default_value.clone());
            }
        }
    }
}
pub fn find_and_draw_matches(
    mut a: Image,
    mut b: Image,
    sigma: f32,
    thresh: f32,
    nms_window: i32,
) -> Image {
    let ad = harris_corner_detector(a.clone(), sigma, thresh, nms_window);
    let bd = harris_corner_detector(b.clone(), sigma, thresh, nms_window);
    let m = match_descriptors(&ad, &bd);
    let corners: Vec<(u32, u32)> = ad.iter().map(|d| (d.x, d.y)).collect();
    draw_plus_on_image(&mut a, &corners, 10, None);

    let corners: Vec<(u32, u32)> = bd.iter().map(|d| (d.x, d.y)).collect();
    draw_plus_on_image(&mut b, &corners, 10, None);

    draw_matches(&a, &b, &m, &ad, &bd)
}
