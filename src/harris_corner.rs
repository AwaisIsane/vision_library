use crate::{
    convolutions::{convolve_image, l1_norm, make_sobel_x_filter, make_sobel_y_filter},
    io::Image,
};

pub fn harris_corner_detector(
    img: Image,
    sigma: f32,
    thresh: f32,
    nms_window: i32,
) -> Vec<Descriptor> {
    let s = structure_matrix(img.clone(), sigma);
    let k: f32 = 0.06; // Typical value between 0.04-0.06
    let corners = cornerness_response(s, k);
    let corners = nms_image(corners, nms_window);
    let mut corners_final = Vec::new();
    for y in 0..corners.height {
        for x in 0..corners.width {
            let val = corners.get_pixel(x as i32, y as i32)[0];
            if val > thresh {
                // corners_final.push((x, y));
                let descriptor = describe_index(&img, x, y);
                corners_final.push(descriptor);
            }
        }
    }
    corners_final
}

pub fn cornerness_response(s: Image, k: f32) -> Image {
    let size = (s.width * s.height) as usize;
    let mut response_data = Vec::with_capacity(size);

    for i in 0..size {
        let ix2 = s.array[i * 3]; // Ix² (channel 0)
        let iy2 = s.array[i * 3 + 1]; // Iy² (channel 1)
        let ixiy = s.array[i * 3 + 2];

        // Structure matrix M = [[Ix², Ix·Iy],
        //                       [Ix·Iy, Iy²]]

        // Determinant: det(M) = (Ix²)(Iy²) - (Ix·Iy)²
        let det = ix2 * iy2 - ixiy * ixiy;

        // Trace: trace(M) = Ix² + Iy²
        let trace = ix2 + iy2;
        // Harris corner response
        // R > 0: corner (both eigenvalues large)
        // R < 0: edge (one eigenvalue large)
        // R ≈ 0: flat region (both eigenvalues small)
        let r = det - k * trace * trace;

        response_data.push(r);
    }
    Image::new(s.width, s.height, 1, response_data)
}

pub fn nms_image(im: Image, w: i32) -> Image {
    let width = im.width as i32;
    let height = im.height as i32;
    let mut result = im.clone();

    for y in 0..height {
        for x in 0..width {
            let current_val = im.get_pixel(x, y)[0];

            let mut is_max = true;

            // Check all neighbors in the (2w+1)x(2w+1) window
            'outer: for dy in -w..=w {
                for dx in -w..=w {
                    let nx = x + dx;
                    let ny = y + dy;

                    // Skip out-of-bounds neighbors
                    if nx < 0 || nx >= width || ny < 0 || ny >= height {
                        continue;
                    }

                    // Skip the center pixel itself
                    if dx == 0 && dy == 0 {
                        continue;
                    }

                    let neighbor_val = im.get_pixel(nx, ny)[0];

                    // If any neighbor is stronger, mark current pixel for suppression
                    if neighbor_val >= current_val {
                        is_max = false;
                        break 'outer; // no need to check other neighbors
                    }
                }
            }
            if !is_max {
                result.put_pixel(x as u32, y as u32, [-1e9].to_vec()); // very negative number
            }
        }
    }
    return result;
}
pub fn structure_matrix(im: Image, sigma: f32) -> Image {
    let sobel_x = make_sobel_x_filter();
    let sobel_y = make_sobel_y_filter();
    let width = im.width;
    let height = im.height;
    let size = (width * height) as usize;

    let i_x = convolve_image(im.clone(), sobel_x, false);
    let i_y = convolve_image(im, sobel_y, false);

    let mut data = Vec::with_capacity(size * 3);
    for i in 0..size {
        let ix = i_x.array[i];
        let iy = i_y.array[i];
        data.push(ix * ix);
        data.push(iy * iy);
        data.push(ix * iy);
    }
    let s_matrix = Image::new(width, height, 3, data);
    smooth_image(&s_matrix, sigma)
}

// Helper function to create 1D Gaussian filter for separable convolution
pub fn make_1d_gaussian(sigma: f32) -> Image {
    // Calculate filter width: 6*sigma, ensure it's odd
    // Match the convention from make_gaussian_filter
    let mut w = (sigma * 6.0) as u32;
    if w % 2 == 0 {
        w += 1; // Ensure odd size for symmetric filter
    }

    let mut data = Vec::with_capacity(w as usize);
    let center = (w as f32 - 1.0) / 2.0;
    let two_sigma_sq = 2.0 * sigma * sigma;

    // Generate 1D Gaussian values
    for x in 0..w {
        let dx = x as f32 - center;
        let value = (-(dx * dx) / two_sigma_sq).exp();
        data.push(value);
    }

    // Normalize so sum equals 1
    l1_norm(Image::new(w, 1, 1, data))
}

// Smooth image using separable Gaussian filtering (optimized version)
pub fn smooth_image(img: &Image, sigma: f32) -> Image {
    if sigma <= 0.0 {
        // No smoothing needed, return a copy
        return img.clone();
    }

    // Create 1D horizontal Gaussian filter (1 x N)
    let g1d_horizontal = make_1d_gaussian(sigma);

    // First pass: convolve horizontally
    let horizontal_pass = convolve_image(img.clone(), g1d_horizontal.clone(), true);

    // Create vertical Gaussian filter (N x 1) by transposing
    let g1d_vertical = Image::new(1, g1d_horizontal.width, 1, g1d_horizontal.array.clone());

    // Second pass: convolve vertically
    convolve_image(horizontal_pass, g1d_vertical, true)
}

// Descriptor structure to hold corner information
#[derive(Clone, Debug)]
pub struct Descriptor {
    pub x: u32,
    pub y: u32,
    pub data: Vec<f32>,
}

// Create a feature descriptor for a pixel location
// This uses a 5x5 window around the corner point
pub fn describe_index(im: &Image, x: u32, y: u32) -> Descriptor {
    let w = 5; // Window size
    let mut data = Vec::with_capacity((w * w * im.channels) as usize);

    // For each channel
    for c in 0..im.channels {
        // Get the central pixel value
        let cval = im.get_pixel(x as i32, y as i32)[c as usize];

        // Extract features in a 5x5 window
        for dy in -(w as i32 / 2)..=(w as i32 / 2) {
            for dx in -(w as i32 / 2)..=(w as i32 / 2) {
                let val = im.get_pixel(x as i32 + dx, y as i32 + dy)[c as usize];
                // Subtract central value to compensate for exposure/lighting changes
                data.push(cval - val);
            }
        }
    }

    Descriptor { x, y, data }
}
