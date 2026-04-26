use image::{DynamicImage, GenericImage, GenericImageView, ImageReader};

use crate::HEADER_SIZE;

pub fn decode(img: &DynamicImage) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let width = img.width();

    let mut extracted_data_binary = String::new();
    let mut data_length_binary = String::new();

    while data_length_binary.len() < HEADER_SIZE {
        let pixel = img.get_pixel(
            (data_length_binary.len() / 3) as u32,
            (data_length_binary.len() / 3 / width as usize) as u32,
        );

        for j in 0..3 {
            if data_length_binary.len() >= HEADER_SIZE {
                break;
            }

            let bit = pixel[j] & 1;
            data_length_binary.push_str(&format!("{}", bit));
        }
    }

    let data_length = u32::from_str_radix(&data_length_binary, 2).unwrap();

    println!("Extracted data length binary: {}", data_length_binary);
    println!("Extracted data length: {}", data_length);

    let mut i = 0; // Start after the first 32 bits which represent the data length
    while i <= (data_length as usize * 8) + HEADER_SIZE {
        let x = (i / 3) as u32;
        let y = (i / 3) as u32 / width;

        if x <= 10 {
            i += 3; // Skip the first 11 pixels (33 bits) which contain the data length
            continue; // Skip the first 11 pixels which contain the data length
        }

        let pixel = img.get_pixel(x, y);

        for j in 0..3 {
            if i > (data_length as usize * 8) + HEADER_SIZE {
                break;
            }

            let bit = pixel[j] & 1;
            extracted_data_binary.push_str(&format!("{}", bit));

            i += 1;
        }
    }

    println!(
        "extracted_data_binary length: {}",
        extracted_data_binary.len()
    );
    println!("Extracted binary data: {}", extracted_data_binary);

    let extracted_data_bytes = extracted_data_binary
        .as_bytes()
        .chunks(8)
        .map(|chunk| {
            let byte_str = std::str::from_utf8(chunk).unwrap();
            u8::from_str_radix(byte_str, 2).unwrap()
        })
        .collect::<Vec<u8>>();

    Ok(extracted_data_bytes)
}

pub fn decode_string(img: &DynamicImage) -> Result<String, Box<dyn std::error::Error>> {
    let extracted_data_bytes = decode(img)?;
    let extracted_data = String::from_utf8(extracted_data_bytes)?;
    Ok(extracted_data)
}