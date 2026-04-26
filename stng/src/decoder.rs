use image::DynamicImage;
use postcard::from_bytes;

use crate::auth::{EncryptionSecret, EncryptionType, SecureContext};
use crate::header::Header;

pub struct Decoder;

impl Decoder {
    pub fn decode(img: &DynamicImage, secret: Option<&EncryptionSecret>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let channels = match img {
            DynamicImage::ImageRgb8(buf) => buf
                .pixels()
                .flat_map(|p| [p[0], p[1], p[2]])
                .collect::<Vec<_>>(),
            DynamicImage::ImageRgba8(buf) => buf
                .pixels()
                .flat_map(|p| [p[0], p[1], p[2]])
                .collect::<Vec<_>>(),
            _ => return Err("Unsupported format".into()),
        };

        let mut bit_iter = channels.iter().map(|b| b & 1);
        
        // Legge 8 bit consecutivi e li assembla in un byte
        macro_rules! read_byte {
            () => {{
                let mut byte = 0u8;
                for _ in 0..8 {
                    let bit = bit_iter.next().ok_or("Image too small")?;
                    byte = (byte << 1) | bit;
                }
                byte
            }};
        }

        // Leggi 1 byte = lunghezza dell'Auth serializzato
        let auth_len = read_byte!() as usize;

        // Leggi i byte dell'Auth
        let mut auth_bytes = Vec::with_capacity(auth_len);
        for _ in 0..auth_len {
            auth_bytes.push(read_byte!());
        }

        let auth: SecureContext = from_bytes(&auth_bytes)?;

        // Leggi 1 byte = lunghezza dell'header serializzato
        let header_len = read_byte!() as usize;


        // Leggi i byte dell'header
        let mut header_bytes = Vec::with_capacity(header_len);
        for _ in 0..header_len {
            header_bytes.push(read_byte!());
        }

        let header: Header = from_bytes(&header_bytes)?;
        assert!(header.magic == *crate::MAGIC, "Invalid magic number");

        let total_bits = header.length as usize * 8;
        let mut out = Vec::with_capacity(header.length as usize);

        let mut byte = 0u8;
        let mut count = 0;

        for _ in 0..total_bits {
            let bit = bit_iter.next().ok_or("Image ended early")?;
            byte = (byte << 1) | bit;
            count += 1;
            if count == 8 {
                out.push(byte);
                byte = 0;
                count = 0;
            }
        }

        // Applica decifratura se necessario
        let out = if !matches!(auth.encryption_type, EncryptionType::None) {
            let s = secret.ok_or("Secret required for decryption")?;
            auth.decrypt(&out, s)?
        } else {
            out
        };

        Ok(out)
    }

    pub fn decode_string(img: &DynamicImage, secret: Option<&EncryptionSecret>) -> Result<String, Box<dyn std::error::Error>> {
        let bytes = Decoder::decode(img, secret)?;
        Ok(String::from_utf8(bytes)?)
    }

    pub fn decode_file(
        img: &DynamicImage,
        output_path: &str,
        secret: Option<&EncryptionSecret>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let bytes = Decoder::decode(img, secret)?;
        std::fs::write(output_path, bytes)?;
        Ok(())
    }

    pub fn decode_bytes(img: &DynamicImage, secret: Option<&EncryptionSecret>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Decoder::decode(img, secret)
    }
}
