use crate::io::Image;

pub fn l1_norm(mut img: Image) -> Image {
    let sum: f32 = img.array.iter().sum();
    img.array.iter_mut().for_each(|x| *x /= sum);
    img
}
pub fn make_box_filter(w: u32) -> Image {
    let px_value: f32 = 1.0 / ((w * w) as f32);
    Image::new(w, w, 1, vec![px_value; (w * w) as usize])
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
