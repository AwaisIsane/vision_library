use crate::io::Image;

pub fn resize_image_nearest_neighbour(img: Image, to_width: u32, to_height: u32) -> Image {
    let mut new_img = vec![];

    let width_ratio = img.width as f32 / to_width as f32;
    let height_ratio = img.height as f32 / to_height as f32;

    for y in 0..to_height {
        for x in 0..to_width {
            // let src_x = ((x as f32 * width_ratio).floor() as u32).min(width - 1);
            // let src_y = ((y as f32 * height_ratio).floor() as u32).min(height - 1);
            let src_x = (x as f32 * width_ratio).floor() as i32;
            let src_y = (y as f32 * height_ratio).floor() as i32;

            // Get the pixel from the source image
            let pixel = img.get_pixel(src_x, src_y);

            // Set the pixel in the target image
            new_img.extend(pixel);
        }
    }
    Image::new(to_width, to_height, img.channels, new_img)
}

pub fn bilinear_resize(img: Image, to_width: u32, to_height: u32) -> Image {
    let mut new_img = vec![];

    let width_ratio = img.width as f32 / to_width as f32;
    let height_ratio = img.height as f32 / to_height as f32;

    for y in 0..to_height {
        for x in 0..to_width {
            // let src_x = ((x as f32 * width_ratio).floor() as u32).min(width - 1);
            // let src_y = ((y as f32 * height_ratio).floor() as u32).min(height - 1);
            let src_x = x as f32 * width_ratio;
            let src_y = y as f32 * height_ratio;

            let x_floor = src_x.floor() as i32;
            let y_floor = src_y.floor() as i32;

            let x_ceil = (src_x.ceil() as i32).min(img.width as i32 - 1);
            let y_ceil = (src_y.ceil() as i32).min(img.height as i32 - 1);

            // Get the pixel from the source image
            let v1 = img.get_pixel(x_floor, y_floor);
            let v2 = img.get_pixel(x_ceil, y_floor);
            let v3 = img.get_pixel(x_floor, y_ceil);
            let v4 = img.get_pixel(x_ceil, y_ceil);

            let dx = src_x - x_floor as f32;
            let dy = src_y - y_floor as f32;

            let mut pixel = [0.0f32; 3];

            for i in 0..pixel.len() {
                let top = v1[i] * (1.0 - dy) + v3[i] * dy;
                let bottom = v2[i] * (1.0 - dy) + v4[i] * dy;
                pixel[i] = top * (1.0 - dx) + bottom * dx;
            }

            // Set the pixel in the target image
            new_img.extend(pixel);
        }
    }
    Image::new(to_width, to_height, img.channels, new_img)
}

pub fn shift_image(mut img: Image, channel: u32, shift_by: f32) -> Image {
    if channel >= img.channels {
        return img;
    }
    for y_i in 0..img.height {
        for x_i in 0..img.width {
            let mut pixel = img.get_pixel(x_i as i32, y_i as i32).to_vec();
            pixel[channel as usize] += shift_by;
            img.put_pixel(x_i, y_i, pixel);
        }
    }
    img
}

pub fn scale_image(mut img: Image, channel: u32, scale_by: f32) -> Image {
    if channel >= img.channels {
        return img;
    }
    for y_i in 0..img.height {
        for x_i in 0..img.width {
            let mut pixel = img.get_pixel(x_i as i32, y_i as i32).to_vec();
            pixel[channel as usize] *= scale_by;
            img.put_pixel(x_i, y_i, pixel);
        }
    }
    img
}

pub fn add_image(img1: Image, img2: Image) -> Image {
    if img1.channels != img2.channels || img1.height != img2.height || img1.width != img2.width {
        panic!("dimesnsions not same")
    }
    let mut new_image = vec![];
    for y_i in 0..img1.height {
        for x_i in 0..img1.width {
            let pixel1 = img1.get_pixel(x_i as i32, y_i as i32);
            let pixel2 = img2.get_pixel(x_i as i32, y_i as i32);

            for c in 0..img1.channels as usize {
                new_image.push(pixel1[c] + pixel2[c]);
            }
        }
    }
    Image::new(img1.width, img1.height, img1.channels, new_image)
}

pub fn sub_image(img1: Image, img2: Image) -> Image {
    if img1.channels != img2.channels || img1.height != img2.height || img1.width != img2.width {
        panic!("dimesnsions not same")
    }
    let mut new_image = vec![];
    for y_i in 0..img1.height {
        for x_i in 0..img1.width {
            let pixel1 = img1.get_pixel(x_i as i32, y_i as i32);
            let pixel2 = img2.get_pixel(x_i as i32, y_i as i32);

            for c in 0..img1.channels as usize {
                new_image.push(pixel1[c] - pixel2[c]);
            }
        }
    }
    Image::new(img1.width, img1.height, img1.channels, new_image)
}
