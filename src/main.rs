use image::ImageReader;
use stng::{decoder::decode_file, encoder::encode_file};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut img = ImageReader::open("images/dyno.png")?.decode()?;

    let file_path = "texts/commedia.txt";

    let img = encode_file(&mut img, file_path)?;
    img.save("images/encoded_image.png")?;

    let output_path = "texts/output.txt";
    decode_file(&img, output_path)?;

    Ok(())
}