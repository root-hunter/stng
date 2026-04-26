use image::{ImageReader};
use stng::{decoder::decode_string, encoder::encode_string};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut img = ImageReader::open("images/stego.jpg")?.decode()?;
    let data = "Ciao a tutti mi chiamo Antonio!!!!";

    encode_string(&mut img, data)?;

    let extracted_data = decode_string(&img)?;
    println!("Extracted data: {}", extracted_data);

    Ok(())
}

mod tests {
    use super::*;

    #[test]
    fn test_steganography() {
        let data = "Hello my name is tony!";
        let data_bytes = data.as_bytes();
        let data_length = data_bytes.len() as u32;

        let data_length_bytes = data_length.to_be_bytes();

        let mut data_binary = data_length_bytes
            .iter()
            .map(|byte| format!("{:08b}", byte))
            .collect::<Vec<String>>()
            .join("");

        println!("Data length in bytes: {}", data_length);
        println!("Data length in binary: {}", data_binary);

        data_binary.push_str(
            &data_bytes
                .iter()
                .map(|byte| format!("{:08b}", byte))
                .collect::<Vec<String>>()
                .join(""),
        );

        println!("Final binary string: {}", data_binary);
    }
}
