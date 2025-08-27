use rayon::prelude::*;
use std::f32::consts::PI;

use crate::{color_space_conversions::hsv_to_rgb, io::Image};

pub fn l1_norm(mut img: Image) -> Image {
    let sum: f32 = img.array.par_iter().sum();
    img.array.par_iter_mut().for_each(|x| *x /= sum);
    img
}

pub fn feature_norm(mut img: Image) -> Image {
    // Find min and max values in the image
    let (min_val, max_val) = img
        .array
        .par_iter()
        .fold(
            || (f32::MAX, f32::MIN),
            |(min, max), &val| (min.min(val), max.max(val)),
        )
        .reduce(
            || (f32::MAX, f32::MIN),
            |(min1, max1), (min2, max2)| (min1.min(min2), max1.max(max2)),
        );

    // Calculate the range
    let range = max_val - min_val;

    // If range is zero (all values are the same), set everything to 0
    if range == 0.0 {
        img.array.par_iter_mut().for_each(|x| *x = 0.0);
    } else {
        // Scale values to [0, 1] range: (x - min) / range
        img.array
            .par_iter_mut()
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
    let mut data = Vec::with_capacity(w as usize); // 1D filter
    let center = (w as f32 - 1.0) / 2.0;
    let two_sigma_sq = 2.0 * sigma * sigma;

    // Calculate 1D Gaussian weights
    data.par_extend((0..w).into_par_iter().map(|x| {
        let dx = x as f32 - center;
        (-(dx * dx) / two_sigma_sq).exp()
    }));

    l1_norm(Image::new(w, 1, 1, data)) // Return a 1D horizontal filter
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

//preserve input output channels
pub fn convolve_image(img: Image, filter: Image, preserve: bool) -> Image {
    //maintains image size
    if !(filter.channels == 1 || filter.channels == img.channels) {
        panic!("Filter should have either 1 channel or same number of channels as image");
    }

    let output_channels = if preserve { img.channels } else { 1 };
    let mut new_array: Vec<f32> =
        Vec::with_capacity((img.width * img.height * output_channels) as usize);
    new_array.resize((img.width * img.height * output_channels) as usize, 0.0);

    // Calculate padding needed to preserve image dimensions
    // For even-sized kernels, we need asymmetric padding
    // Example: 4x4 kernel needs total padding of 3
    // We use pad_left=1, pad_right=2 (or pad_top=1, pad_bottom=2)
    let pad_left = (filter.width - 1) / 2;
    let pad_top = (filter.height - 1) / 2;

    let img_width = img.width;
    let img_height = img.height;
    let img_channels = img.channels;
    let filter_width = filter.width;
    let filter_height = filter.height;
    let filter_channels = filter.channels;

    // Process each row in parallel
    new_array
        .par_chunks_mut((img_width * output_channels) as usize)
        .enumerate()
        .for_each(|(y, row_chunk)| {
            for x in 0..img_width {
                let mut sum = vec![0.0; img_channels as usize];
                // Apply the filter
                for fy in 0..filter_height {
                    for fx in 0..filter_width {
                        // Calculate the position in the input image
                        let img_y = y as i32 + fy as i32 - pad_top as i32;
                        let img_x = x as i32 + fx as i32 - pad_left as i32;

                        // Safety: get_pixel handles out of bounds by returning zero_pixel
                        let pixel = img.get_pixel(img_x, img_y);

                        for chan in 0..(img_channels as usize) {
                            let filter_pix = if filter_channels == 1 {
                                // Safety: fx, fy are within filter bounds
                                unsafe { filter.get_pixel_unchecked(fx, fy)[0] }
                            } else {
                                // Safety: fx, fy are within filter bounds
                                unsafe { filter.get_pixel_unchecked(fx, fy)[chan] }
                            };
                            sum[chan] += filter_pix * pixel[chan];
                        }
                    }
                }

                // Write to the output
                let start_idx = (x * output_channels) as usize;
                let end_idx = start_idx + output_channels as usize;
                if preserve {
                    row_chunk[start_idx..end_idx].copy_from_slice(&sum);
                } else {
                    row_chunk[start_idx] = sum.iter().sum();
                }
            }
        });

    Image::new(img.width, img.height, output_channels, new_array)
}

fn convolve_1d(img: Image, filter: Image, axis: Axis, preserve: bool) -> Image {
    if filter.channels != 1 {
        panic!("1D filter must have 1 channel");
    }

    let (main_dim, cross_dim, filter_len) = match axis {
        Axis::Horizontal => (img.width, img.height, filter.width),
        Axis::Vertical => (img.height, img.width, filter.height),
    };

    let output_channels = if preserve { img.channels } else { 1 };
    let mut new_array = Vec::with_capacity((img.width * img.height * output_channels) as usize);
    new_array.resize((img.width * img.height * output_channels) as usize, 0.0);

    let pad = (filter_len - 1) / 2;

    match axis {
        Axis::Horizontal => {
            // Process each row in parallel
            new_array
                .par_chunks_mut((img.width * output_channels) as usize)
                .enumerate()
                .for_each(|(j, row_chunk)| {
                    for i in 0..main_dim {
                        let mut sum = vec![0.0; img.channels as usize];
                        for k in 0..filter_len {
                            let img_x = i as i32 + k as i32 - pad as i32;
                            let img_y = j as i32;

                            let pixel = img.get_pixel(img_x, img_y);
                            let filter_pix = unsafe { filter.get_pixel_unchecked(k, 0)[0] };

                            for chan in 0..(img.channels as usize) {
                                sum[chan] += filter_pix * pixel[chan];
                            }
                        }

                        let start_idx = (i * output_channels) as usize;
                        let end_idx = start_idx + output_channels as usize;
                        if preserve {
                            row_chunk[start_idx..end_idx].copy_from_slice(&sum);
                        } else {
                            row_chunk[start_idx] = sum.iter().sum();
                        }
                    }
                });
        }
        Axis::Vertical => {
            // For vertical convolution, we can't easily chunk the output array
            // So we'll use a different approach: calculate all results first, then write
            let pixel_results: Vec<(usize, Vec<f32>)> = (0..cross_dim * main_dim)
                .into_par_iter()
                .map(|idx| {
                    let j = idx / main_dim; // column
                    let i = idx % main_dim; // row within column

                    let mut sum = vec![0.0; img.channels as usize];
                    for k in 0..filter_len {
                        let img_x = j as i32;
                        let img_y = i as i32 + k as i32 - pad as i32;

                        let pixel = img.get_pixel(img_x, img_y);
                        let filter_pix = unsafe { filter.get_pixel_unchecked(0, k)[0] };

                        for chan in 0..(img.channels as usize) {
                            sum[chan] += filter_pix * pixel[chan];
                        }
                    }

                    let output_idx = (i * img.width + j) * output_channels;
                    (output_idx as usize, sum)
                })
                .collect();

            // Write results back to the array
            for (output_idx, sum) in pixel_results {
                if preserve {
                    for (chan, &value) in sum.iter().enumerate() {
                        new_array[output_idx + chan] = value;
                    }
                } else {
                    new_array[output_idx] = sum.iter().sum();
                }
            }
        }
    }

    Image::new(img.width, img.height, output_channels, new_array)
}
/// Applies a separable convolution filter (e.g., Gaussian) by performing two 1D convolutions.
pub fn convolve_image_separable(img: Image, filter_1d: Image, preserve: bool) -> Image {
    if filter_1d.width == 1 && filter_1d.height > 1 {
        // Vertical filter
        let temp_img = convolve_1d(img, filter_1d, Axis::Vertical, preserve);
        temp_img
    } else if filter_1d.height == 1 && filter_1d.width > 1 {
        // Horizontal filter
        let temp_img = convolve_1d(img, filter_1d, Axis::Horizontal, preserve);
        temp_img
    } else if filter_1d.width > 1 && filter_1d.height > 1 {
        // Assume square 2D filter, decompose into 1D horizontal and vertical
        // This requires the 2D filter to be separable, which is true for Gaussian
        let w = filter_1d.width;
        let mut horizontal_filter_data = Vec::with_capacity(w as usize);
        let mut vertical_filter_data: Vec<f32> = Vec::with_capacity(w as usize);

        // For a separable 2D Gaussian, the 1D filters are the square root of the 2D filter values
        // This is a simplification, a proper separable filter decomposition is more complex
        // For now, let's assume make_gaussian_filter returns a 1D filter if w=1 or h=1
        // Or, we create a 1D Gaussian filter directly.
        // Let's create a 1D Gaussian filter here.
        let sigma = (w as f32 - 1.0) / 6.0; // Reverse engineer sigma from w
        let center = (w as f32 - 1.0) / 2.0;
        let two_sigma_sq = 2.0 * sigma * sigma;

        for x in 0..w {
            let dx = x as f32 - center;
            let value = (-(dx * dx) / two_sigma_sq).exp();
            horizontal_filter_data.push(value);
        }
        let horizontal_filter = l1_norm(Image::new(w, 1, 1, horizontal_filter_data));
        let vertical_filter = l1_norm(Image::new(1, w, 1, horizontal_filter.array.clone())); // Vertical filter is just transposed horizontal

        let temp_img = convolve_1d(img, horizontal_filter, Axis::Horizontal, preserve);
        convolve_1d(temp_img, vertical_filter, Axis::Vertical, preserve)
    } else {
        panic!("Filter dimensions not suitable for separable convolution");
    }
}

#[derive(Clone, Copy)]
enum Axis {
    Horizontal,
    Vertical,
}

pub fn sobel_image(img: &Image) -> (Image, Image) {
    let imgx = convolve_image(img.clone(), make_sobel_x_filter(), false);
    let imgy = convolve_image(img.clone(), make_sobel_y_filter(), false);

    let mut magnitude_array =
        Vec::with_capacity((imgx.width * imgx.height * imgx.channels) as usize);
    let mut direction_array =
        Vec::with_capacity((imgy.width * imgy.height * imgy.channels) as usize);

    magnitude_array.resize((imgx.width * imgx.height * imgx.channels) as usize, 0.0);
    direction_array.resize((imgy.width * imgy.height * imgy.channels) as usize, 0.0);

    let imgx_width = imgx.width;
    let imgx_height = imgx.height;
    let imgx_channels = imgx.channels;

    magnitude_array
        .par_chunks_mut(imgx_channels as usize)
        .zip(direction_array.par_chunks_mut(imgx_channels as usize))
        .enumerate()
        .for_each(|(i, (mag_pixel_ref, dir_pixel_ref))| {
            let x_i = (i as u32) % imgx_width;
            let y_i = (i as u32) / imgx_width;

            // Safety: x_i, y_i are within imgx and imgy bounds
            unsafe {
                let pixel1 = imgx.get_pixel_unchecked(x_i, y_i);
                let pixel2 = imgy.get_pixel_unchecked(x_i, y_i);

                for c in 0..imgx_channels as usize {
                    mag_pixel_ref[c] = (pixel1[c].powf(2.0) + pixel2[c].powf(2.0)).sqrt();
                    dir_pixel_ref[c] = pixel2[c].atan2(pixel1[c]);
                }
            }
        });

    (
        Image::new(imgx.width, imgx.height, imgx.channels, magnitude_array),
        Image::new(imgy.width, imgy.height, imgy.channels, direction_array),
    )
}

pub fn colorize_sobel(img: Image) -> Image {
    let mut new_img_array = Vec::with_capacity((img.width * img.height * 3) as usize); // 3 channels for HSV
    new_img_array.resize((img.width * img.height * 3) as usize, 0.0);

    // Use separable convolution for Gaussian blur
    let gaussian_filter_1d = make_gaussian_filter(1.0);
    let convolved_img = convolve_image_separable(img, gaussian_filter_1d, true);

    let (magnitude, direction) = sobel_image(&convolved_img); // Pass reference
    let magnitude = feature_norm(magnitude);

    let magnitude_width = magnitude.width;
    let magnitude_height = magnitude.height;
    let magnitude_channels = magnitude.channels;

    new_img_array
        .par_chunks_mut(3) // Output is always 3 channels for HSV
        .enumerate()
        .for_each(|(i, pixel_ref)| {
            let x_i = (i as u32) % magnitude_width;
            let y_i = (i as u32) / magnitude_width;

            // Safety: x_i, y_i are within magnitude and direction bounds
            unsafe {
                let direction_pix = direction.get_pixel_unchecked(x_i, y_i);
                let magnitude_pix = magnitude.get_pixel_unchecked(x_i, y_i);

                for c in 0..magnitude_channels as usize {
                    pixel_ref[0] = (direction_pix[c] / (2.0 * PI)) + 0.7; // Direct conversion from radians to 0-1;
                    pixel_ref[1] = magnitude_pix[c];
                    pixel_ref[2] = magnitude_pix[c];
                }
            }
        });

    hsv_to_rgb(Image::new(
        magnitude.width,
        magnitude.height,
        3,
        new_img_array,
    ))
}
