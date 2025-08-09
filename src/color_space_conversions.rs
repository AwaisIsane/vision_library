use crate::io::Image;

pub fn rgb_to_hsv(mut img: Image) -> Image {
    for y_i in 0..img.height {
        for x_i in 0..img.width {
            let pixel = img.get_pixel(x_i as i32, y_i as i32);
            let v = pixel[0].max(pixel[1]).max(pixel[2]);
            let m = pixel[0].min(pixel[1]).min(pixel[2]);
            let c = v - m;
            let s = if v != 0.0 { c / v } else { 0.0 };
            let h_c = if c == 0.0 {
                0.0
            } else if v == pixel[0] {
                (pixel[1] - pixel[2]) / c
            } else if v == pixel[1] {
                (pixel[2] - pixel[0]) / c + 2.0
            } else {
                (pixel[0] - pixel[1]) / c + 4.0
            };
            let h = if h_c < 0.0 {
                h_c / 6.0 + 1.0
            } else {
                h_c / 6.0
            };
            img.put_pixel(x_i, y_i, vec![h, s, v]);
        }
    }

    img
}

pub fn hsv_to_rgb(mut img: Image) -> Image {
    for y_i in 0..img.height {
        for x_i in 0..img.width {
            let pixel = img.get_pixel(x_i as i32, y_i as i32);
            let [h, s, v] = [pixel[0], pixel[1], pixel[2]];
            let c = v * s;
            let x = c * (1.0 - ((h * 6.0) % 2.0 - 1.0).abs());
            let m = v - c;

            let (r, g, b) = match (h * 6.0).floor() as i32 {
                0 => (c, x, 0.0),
                1 => (x, c, 0.0),
                2 => (0.0, c, x),
                3 => (0.0, x, c),
                4 => (x, 0.0, c),
                _ => (c, 0.0, x),
            };
            img.put_pixel(x_i, y_i, vec![r + m, g + m, b + m]);
        }
    }
    img
}

pub fn rgb_to_hcl(mut img: Image) -> Image {
    //https://www.easyrgb.com/en/math.php

    for y_i in 0..img.height {
        for x_i in 0..img.width {
            let pixel = img.get_pixel(x_i as i32, y_i as i32);
            let [r, g, b] = [pixel[0], pixel[1], pixel[2]];

            //sRGB => CIEXYZ
            let r = if r > 0.04045 {
                ((r + 0.055) / 1.055).powf(2.4)
            } else {
                r / 12.92
            };
            let g = if g > 0.04045 {
                ((g + 0.055) / 1.055).powf(2.4)
            } else {
                g / 12.92
            };
            let b = if b > 0.04045 {
                ((b + 0.055) / 1.055).powf(2.4)
            } else {
                b / 12.92
            };
            let r = r * 100.0;
            let g = g * 100.0;
            let b = b * 100.0;
            let x = 0.4124 * r + 0.3576 * g + 0.1805 * b;
            let y = 0.2126 * r + 0.7152 * g + 0.0722 * b;
            let z = 0.0193 * r + 0.1192 * g + 0.9505 * b;

            //CIEXYZ => CIEL*UV
            let ref_x = 95.047;
            let ref_y = 100.0;
            let ref_z = 108.883;

            let u_prime = (4.0 * x) / (x + (15.0 * y) + (3.0 * z));
            let v_prime = (9.0 * y) / (x + (15.0 * y) + (3.0 * z));
            let y_prime = y / 100.0;

            let y_prime = if y_prime > 0.008856 {
                y_prime.powf(1.0 / 3.0)
            } else {
                (7.787 * y_prime) + (16.0 / 116.0)
            };

            let ref_u_prime = (4.0 * ref_x) / (ref_x + (15.0 * ref_y) + (3.0 * ref_z));
            let ref_v_prime = (9.0 * ref_y) / (ref_x + (15.0 * ref_y) + (3.0 * ref_z));

            let l = 116.0 * y_prime - 16.0;

            let u = 13.0 * l * (u_prime - ref_u_prime);
            let v = 13.0 * l * (v_prime - ref_v_prime);

            let c = u.hypot(v);
            let h = v.atan2(u);

            img.put_pixel(x_i, y_i, vec![h, c, l]);
        }
    }
    img
}

pub fn hcl_to_rgb(mut img: Image) -> Image {
    for y_i in 0..img.height {
        for x_i in 0..img.width {
            let pixel = img.get_pixel(x_i as i32, y_i as i32);
            let h = pixel[0];
            let c = pixel[1];
            let l = pixel[2];

            // HCL to LUV
            let u = c * h.cos();
            let v = c * h.sin();

            // LUV to XYZ
            let y = (l + 16.0) / 116.0;
            let y = if y.powi(3) > 0.008856 {
                y.powi(3)
            } else {
                ((y - 16.0) / 116.0) / 7.787
            };

            let ref_u = (4.0 * 95.047) / (95.047 + (15.0 * 100.0) + (3.0 * 108.883));
            let ref_v = (9.0 * 100.0) / (95.047 + (15.0 * 100.0) + (3.0 * 108.883));

            let u_prime = u / (13.0 * l) + ref_u;
            let v_prime = v / (13.0 * l) + ref_v;

            let y = y * 100.0;
            let x = -(y * 9.0 * u_prime) / ((u_prime - 4.0) * v_prime - u_prime * v_prime);
            let z = (9.0 * y - (15.0 * v_prime * y) - (v_prime * x)) / (3.0 * v_prime);

            // XYZ to sRGB

            let x = x / 100.0;
            let y = y / 100.0;
            let z = z / 100.0;

            let r = 3.2406 * x - 1.5372 * y - 0.4986 * z;
            let g = -0.9689 * x + 1.8758 * y + 0.0415 * z;
            let b = 0.0557 * x - 0.2040 * y + 1.0570 * z;

            // Apply gamma correction
            let r = if r > 0.0031308 {
                1.055 * r.powf(1.0 / 2.4) - 0.055
            } else {
                12.92 * r
            };
            let g = if g > 0.0031308 {
                1.055 * g.powf(1.0 / 2.4) - 0.055
            } else {
                12.92 * g
            };
            let b = if b > 0.0031308 {
                1.055 * b.powf(1.0 / 2.4) - 0.055
            } else {
                12.92 * b
            };

            // Clamp and convert to f32
            img.put_pixel(x_i, y_i, vec![r, g, b]);
        }
    }

    img
}

pub fn rgb_to_grayscale(img: Image) -> Image {
    let mut grayscale = vec![];

    for y in 0..img.height {
        for x in 0..img.width {
            let pixel = img.get_pixel(x as i32, y as i32);
            let luma = 0.299 * pixel[0] + 0.587 * pixel[1] + 0.114 * pixel[2];
            grayscale.push(luma);
        }
    }
    Image {
        array: grayscale,
        height: img.height,
        width: img.width,
        channels: 1,
    }
}
