use crate::io::Image;
use rayon::prelude::*;
// pub fn resize_image_nearest_neighbour(img: Image, to_width: u32, to_height: u32) -> Image {
//     let mut new_img_array = Vec::with_capacity((to_width * to_height * img.channels) as usize);
//     new_img_array.resize((to_width * to_height * img.channels) as usize, 0.0);

//     let width_ratio = img.width as f32 / to_width as f32;
//     let height_ratio = img.height as f32 / to_height as f32;
//     let channels = img.channels as usize;

//     // Collect all pixel data first, then write to output
//     let pixels: Vec<_> = (0..to_height)
//         .into_par_iter()
//         .flat_map(|y| {
//             (0..to_width).into_par_iter().map(move |x| {
//                 let src_x = (x as f32 * width_ratio).floor() as i32;
//                 let src_y = (y as f32 * height_ratio).floor() as i32;
//                 let pixel = img.get_pixel(src_x, src_y);
//                 ((y, x), pixel.to_vec())
//             })
//         })
//         .collect();

//     let mut output_img = Image::new(to_width, to_height, img.channels, new_img_array);

//     // Write collected pixels to output
//     for ((y, x), pixel_data) in pixels {
//         if let Some(output_pixel) = output_img.get_pixel_mut(x, y) {
//             output_pixel.copy_from_slice(&pixel_data);
//         }
//     }

//     output_img
// }

// pub fn bilinear_resize(img: Image, to_width: u32, to_height: u32) -> Image {
//     let mut new_img_array = Vec::with_capacity((to_width * to_height * img.channels) as usize);
//     new_img_array.resize((to_width * to_height * img.channels) as usize, 0.0);

//     let width_ratio = img.width as f32 / to_width as f32;
//     let height_ratio = img.height as f32 / to_height as f32;
//     let channels = img.channels as usize;

//     // Collect all interpolated pixel data first, then write to output
//     let pixels: Vec<_> = (0..to_height)
//         .into_par_iter()
//         .flat_map(|y| {
//             (0..to_width).into_par_iter().map(move |x| {
//                 let src_x = x as f32 * width_ratio;
//                 let src_y = y as f32 * height_ratio;

//                 let x_floor = src_x.floor() as i32;
//                 let y_floor = src_y.floor() as i32;

//                 let x_ceil = (src_x.ceil() as i32).min(img.width as i32 - 1);
//                 let y_ceil = (src_y.ceil() as i32).min(img.height as i32 - 1);

//                 let v1 = img.get_pixel(x_floor, y_floor);
//                 let v2 = img.get_pixel(x_ceil, y_floor);
//                 let v3 = img.get_pixel(x_floor, y_ceil);
//                 let v4 = img.get_pixel(x_ceil, y_ceil);

//                 let dx = src_x - x_floor as f32;
//                 let dy = src_y - y_floor as f32;

//                 let mut pixel = vec![0.0f32; channels];

//                 for i in 0..channels {
//                     let top = v1[i] * (1.0 - dy) + v3[i] * dy;
//                     let bottom = v2[i] * (1.0 - dy) + v4[i] * dy;
//                     pixel[i] = top * (1.0 - dx) + bottom * dx;
//                 }

//                 ((y, x), pixel)
//             })
//         })
//         .collect();

//     let mut output_img = Image::new(to_width, to_height, img.channels, new_img_array);

//     // Write collected pixels to output
//     for ((y, x), pixel_data) in pixels {
//         if let Some(output_pixel) = output_img.get_pixel_mut(x, y) {
//             output_pixel.copy_from_slice(&pixel_data);
//         }
//     }

//     output_img
// }

// // Alternative approach: Process chunks of rows in parallel
// pub fn resize_image_nearest_neighbour_chunked(img: Image, to_width: u32, to_height: u32) -> Image {
//     let mut new_img_array = Vec::with_capacity((to_width * to_height * img.channels) as usize);
//     new_img_array.resize((to_width * to_height * img.channels) as usize, 0.0);

//     let width_ratio = img.width as f32 / to_width as f32;
//     let height_ratio = img.height as f32 / to_height as f32;

//     let mut output_img = Image::new(to_width, to_height, img.channels, new_img_array);

//     // Process rows in parallel chunks
//     let chunk_size = (to_height / rayon::current_num_threads() as u32).max(1);
//     let rows: Vec<u32> = (0..to_height).collect();

//     rows.par_chunks(chunk_size as usize).for_each(|row_chunk| {
//         for &y in row_chunk {
//             for x in 0..to_width {
//                 let src_x = (x as f32 * width_ratio).floor() as i32;
//                 let src_y = (y as f32 * height_ratio).floor() as i32;

//                 let pixel = img.get_pixel(src_x, src_y);

//                 // This is safe because each chunk processes different rows
//                 unsafe {
//                     let output_pixel = output_img.get_pixel_unchecked_mut(x, y);
//                     output_pixel.copy_from_slice(pixel);
//                 }
//             }
//         }
//     });

//     output_img
// }

pub fn shift_image(mut img: Image, channel: u32, shift_by: f32) -> Image {
    if channel >= img.channels {
        return img;
    }
    let channels = img.channels;
    img.array
        .par_chunks_mut(channels as usize)
        .for_each(|pixel_ref| {
            pixel_ref[channel as usize] += shift_by;
        });
    img
}

pub fn scale_image(mut img: Image, channel: u32, scale_by: f32) -> Image {
    if channel >= img.channels {
        return img;
    }
    let channels = img.channels;
    img.array
        .par_chunks_mut(channels as usize)
        .for_each(|pixel_ref| {
            pixel_ref[channel as usize] *= scale_by;
        });
    img
}

pub fn add_image(img1: Image, img2: Image) -> Image {
    if img1.channels != img2.channels || img1.height != img2.height || img1.width != img2.width {
        panic!("dimesnsions not same")
    }
    let mut new_image_array =
        Vec::with_capacity((img1.width * img1.height * img1.channels) as usize);
    new_image_array.resize((img1.width * img1.height * img1.channels) as usize, 0.0);

    let channels = img1.channels;
    let mut output_img = Image::new(img1.width, img1.height, img1.channels, new_image_array);

    output_img
        .array
        .par_chunks_mut(channels as usize)
        .zip(img1.array.par_chunks(channels as usize))
        .zip(img2.array.par_chunks(channels as usize))
        .for_each(|((output_pixel, pixel1), pixel2)| {
            for c in 0..channels as usize {
                output_pixel[c] = pixel1[c] + pixel2[c];
            }
        });
    output_img
}

pub fn sub_image(img1: Image, img2: Image) -> Image {
    if img1.channels != img2.channels || img1.height != img2.height || img1.width != img2.width {
        panic!("dimesnsions not same")
    }
    let mut new_image_array =
        Vec::with_capacity((img1.width * img1.height * img1.channels) as usize);
    new_image_array.resize((img1.width * img1.height * img1.channels) as usize, 0.0);

    let channels = img1.channels;
    let mut output_img = Image::new(img1.width, img1.height, img1.channels, new_image_array);

    output_img
        .array
        .par_chunks_mut(channels as usize)
        .zip(img1.array.par_chunks(channels as usize))
        .zip(img2.array.par_chunks(channels as usize))
        .for_each(|((output_pixel, pixel1), pixel2)| {
            for c in 0..channels as usize {
                output_pixel[c] = pixel1[c] - pixel2[c];
            }
        });
    output_img
}
