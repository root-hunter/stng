use image::{DynamicImage, GenericImage, GenericImageView, ImageReader};

use crate::HEADER_SIZE;

pub fn encode(img: &mut DynamicImage, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let width = img.width();
    let height = img.height();
    let pixels_count = width * height;

    println!("Image dimensions: {}x{}", width, height);
    println!("Total number of pixels: {}", pixels_count);
    println!("Bits needed for header encoding: {}", HEADER_SIZE);

    let data_length = data.len() as u32;
    let data_length_bytes = data_length.to_be_bytes();

    let mut data_binary = data_length_bytes
        .iter()
        .map(|byte| format!("{:08b}", byte))
        .collect::<Vec<String>>()
        .join("");

    println!("Data length in bytes: {}", data_length);
    println!("Data length in binary: {}", data_binary);

    data_binary.push_str(
        &data
            .iter()
            .map(|byte| format!("{:08b}", byte))
            .collect::<Vec<String>>()
            .join(""),
    );

    println!(
        "Bits needed for data encoding: {}",
        data_binary.len() - HEADER_SIZE
    );

    let mut x = 0;
    let mut y = 0;

    while x < width && y < height && !data_binary.is_empty() {
        let pixel = img.get_pixel(x, y);
        let mut r = pixel[0];
        let mut g = pixel[1];
        let mut b = pixel[2];
        let a = pixel[3];

        for j in 0..3 {
            if data_binary.is_empty() {
                break;
            }

            let bit = data_binary.chars().next().unwrap().to_digit(2).unwrap() as u8;

            if bit == 0 {
                match j {
                    0 => r = (r & 0xFE) | 0,
                    1 => g = (g & 0xFE) | 0,
                    2 => b = (b & 0xFE) | 0,
                    _ => unreachable!(),
                }
            } else {
                match j {
                    0 => r = (r & 0xFE) | 1,
                    1 => g = (g & 0xFE) | 1,
                    2 => b = (b & 0xFE) | 1,
                    _ => unreachable!(),
                }
            }
            data_binary.remove(0);
        }

        img.put_pixel(x, y, image::Rgba([r, g, b, a]));

        x += 1;
        if x >= width {
            x = 0;
            y += 1;
        }
    }

    img.save("images/encoded_image.png")?;

    Ok(())
}

pub fn encode_string(img: &mut DynamicImage, data: &str) -> Result<(), Box<dyn std::error::Error>> {
    encode(img, data.as_bytes())
}