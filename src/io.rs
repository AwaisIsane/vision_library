use image::{DynamicImage, GenericImageView, ImageBuffer, Luma, Rgb};
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub channels: u32,
    pub array: Vec<f32>,
    zero_pixel: Vec<f32>,
}

impl Image {
    pub fn new(width: u32, height: u32, channels: u32, array: Vec<f32>) -> Self {
        Self {
            width,
            height,
            channels,
            array,
            zero_pixel: vec![0.0; channels as usize],
        }
    }
    pub fn get_pixel(&self, x: i32, y: i32) -> &[f32] {
        if x >= self.width as i32 || y >= self.height as i32 || x < 0 || y < 0 {
            // Return zeroed pixel if out of bounds
            return &self.zero_pixel;
        }

        let index = ((y * self.width as i32 + x) * self.channels as i32) as usize;

        &self.array[index..index + self.channels as usize]
    }
    pub fn put_pixel(&mut self, x: u32, y: u32, pixel: Vec<f32>) -> bool {
        if x >= self.width || y >= self.height || pixel.len() != self.channels as usize {
            return false;
        }

        let index_r = ((y * self.width + x) * self.channels) as usize;

        for c in 0..self.channels {
            self.array[index_r + c as usize] = pixel[c as usize];
        }

        true
    }

    // Write RGB Array3<f32> to JPEG file
    pub fn write_image(self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let Image {
            height,
            width,
            channels,
            array,
            ..
        } = self;
        if channels != 3 && channels != 1 {
            return Err("Array must have 3 channels or 1 channel  for RGB or grayscale".into());
        }

        let u8_data: Vec<u8> = array
            .iter()
            .map(|&x| (x.clamp(0.0, 1.0) * 255.0) as u8)
            .collect();

        let dynamic_img = if channels == 3 {
            let img_buffer =
                ImageBuffer::<Rgb<u8>, Vec<u8>>::from_raw(width as u32, height as u32, u8_data)
                    .ok_or("Failed to create RGB image buffer")?;
            DynamicImage::ImageRgb8(img_buffer)
        } else {
            let img_buffer =
                ImageBuffer::<Luma<u8>, Vec<u8>>::from_raw(width as u32, height as u32, u8_data)
                    .ok_or("Failed to create grayscale image buffer")?;
            DynamicImage::ImageLuma8(img_buffer)
        };

        dynamic_img.save(filename)?;
        Ok(())
    }
}

pub fn read_image(path: &str) -> Result<Image, Box<dyn std::error::Error>> {
    let img = image::open(path)?;
    let (width, height) = img.dimensions();

    let (converted, channels) = match img.color() {
        image::ColorType::L8 | image::ColorType::L16 => (img.to_luma32f().into_raw(), 1),
        image::ColorType::Rgb8 | image::ColorType::Rgb16 | image::ColorType::Rgb32F => {
            (img.to_rgb32f().into_raw(), 3)
        }
        image::ColorType::Rgba8 | image::ColorType::Rgba16 | image::ColorType::Rgba32F => {
            (img.to_rgba32f().into_raw(), 4)
        }
        other => {
            return Err(format!("Unsupported color type: {:?}", other).into());
        }
    };

    Ok(Image::new(width, height, channels, converted))
}
