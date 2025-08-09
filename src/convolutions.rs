use crate::io::Image;

pub fn l1_norm(mut img: Image) -> Image {
    let sum: f32 = img.array.iter().sum();
    img.array.iter_mut().for_each(|x| *x /= sum);
    img
}

pub fn feature_norm(mut img: Image) -> Image {
    // Find min and max values in the image
    //TODO simplify this
    let min_val = *img
        .array
        .iter()
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    let max_val = *img
        .array
        .iter()
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();

    // Calculate the range
    let range = max_val - min_val;

    // If range is zero (all values are the same), set everything to 0
    if range == 0.0 {
        img.array.iter_mut().for_each(|x| *x = 0.0);
    } else {
        // Scale values to [0, 1] range: (x - min) / range
        img.array
            .iter_mut()
            .for_each(|x| *x = (*x - min_val) / range);
    }

    img
}
pub fn make_box_filter(w: u32) -> Image {
    let px_value: f32 = 1.0 / ((w * w) as f32);
    Image::new(w, w, 1, vec![px_value; (w * w) as usize])
}

pub fn make_gaussian_filter(sigma: f32) -> Image {
    let w: u32 = (sigma * 6.0 + if sigma % 2.0 == 0.0 { 1.0 } else { 0.0 }) as u32;
    let mut data = Vec::with_capacity((w * w) as usize);
    let center = (w as f32 - 1.0) / 2.0;
    let two_sigma_sq = 2.0 * sigma * sigma;

    // Calculate Gaussian weights
    for y in 0..w {
        for x in 0..w {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            let value = (-(dx * dx + dy * dy) / two_sigma_sq).exp();
            data.push(value);
        }
    }

    l1_norm(Image::new(w, w, 1, data))
}

pub fn make_high_pass_filter() -> Image {
    let high_pass_filter: Vec<f32> = vec![0.0, -1.0, 0.0, -1.0, 4.0, -1.0, 0.0, -1.0, 0.0];
    Image::new(3, 3, 1, high_pass_filter)
}
pub fn make_sharpen_filter() -> Image {
    let sharpen_filter: Vec<f32> = vec![0.0, -1.0, 0.0, -1.0, 5.0, -1.0, 0.0, -1.0, 0.0];
    Image::new(3, 3, 1, sharpen_filter)
}
pub fn make_emboss_filter() -> Image {
    let embossfilter: Vec<f32> = vec![-2.0, -1.0, 0.0, -1.0, 1.0, 1.0, 0.0, 1.0, 2.0];
    Image::new(3, 3, 1, embossfilter)
}
pub fn make_sobel_x_filter() -> Image {
    let sobelfilter_x: Vec<f32> = vec![-1.0, 0.0, 1.0, -2.0, 0.0, 2.0, -1.0, 0.0, 1.0];
    Image::new(3, 3, 1, sobelfilter_x)
}
pub fn make_sobel_y_filter() -> Image {
    let sobelfilter_x: Vec<f32> = vec![-1.0, -2.0, -1.0, 0.0, 0.0, 0.0, 1.0, 2.0, 1.0];
    Image::new(3, 3, 1, sobelfilter_x)
}
//preserve to preserve input output channels
pub fn convolve_image(img: Image, filter: Image, preserve: bool) -> Image {
    //maintains image size
    if !(filter.channels == 1 || filter.channels == img.channels) {
        panic!("Filter should have either 1 channel or same number of channels as image");
    }

    let mut new_array: Vec<f32> = vec![];

    // Calculate padding needed to preserve image dimensions
    // For even-sized kernels, we need asymmetric padding
    // Example: 4x4 kernel needs total padding of 3
    // We use pad_left=1, pad_right=2 (or pad_top=1, pad_bottom=2)
    let pad_left = (filter.width - 1) / 2;
    let pad_top = (filter.height - 1) / 2;

    for y in 0..img.height {
        for x in 0..img.width {
            let mut sum = vec![0.0; img.channels as usize];
            // Apply the filter
            for fy in 0..filter.height {
                for fx in 0..filter.width {
                    // Calculate the position in the input image
                    let img_y = y as i32 + fy as i32 - pad_top as i32;
                    let img_x = x as i32 + fx as i32 - pad_left as i32;
                    let pixel = img.get_pixel(img_x, img_y);

                    for chan in 0..(img.channels as usize) {
                        let filter_pix = if filter.channels == 1 {
                            filter.get_pixel(fx as i32, fy as i32)[0]
                        } else {
                            filter.get_pixel(fx as i32, fy as i32)[chan]
                        };
                        sum[chan] += filter_pix * pixel[chan];
                    }
                }
            }
            if preserve {
                new_array.extend(sum);
            } else {
                new_array.push(sum.iter().sum());
            }
        }
    }
    Image::new(
        img.width,
        img.height,
        if preserve { img.channels } else { 1 },
        new_array,
    )
}

pub fn sobel_image(img: Image) -> (Image, Image) {
    let mut magnitude = vec![];
    let mut direction = vec![];
    let imgx = convolve_image(img.clone(), make_sobel_x_filter(), false);
    let imgy = convolve_image(img, make_sobel_y_filter(), false);

    for y_i in 0..imgx.height {
        for x_i in 0..imgx.width {
            let pixel1 = imgx.get_pixel(x_i as i32, y_i as i32);
            let pixel2 = imgy.get_pixel(x_i as i32, y_i as i32);
            for c in 0..imgx.channels as usize {
                magnitude.push((pixel1[c].powf(2.0) + pixel2[c].powf(2.0)).sqrt());
                direction.push(pixel2[c].atan2(pixel1[c]));
            }
        }
    }
    (
        Image::new(imgx.width, imgx.height, imgx.channels, magnitude),
        Image::new(imgx.width, imgx.height, imgx.channels, direction),
    )
}
