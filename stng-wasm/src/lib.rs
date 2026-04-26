use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use stng::auth::EncryptionSecret;
use stng::data::{Data, DataElement, DataType};
use wasm_bindgen::prelude::*;

fn parse_secret(encryption: &str, key: &[u8]) -> Option<EncryptionSecret> {
    match encryption {
        "xor" => Some(EncryptionSecret::Xor(key.to_vec())),
        "aes256" => {
            let mut k = [0u8; 32];
            let len = key.len().min(32);
            k[..len].copy_from_slice(&key[..len]);
            Some(EncryptionSecret::Aes256(k.to_vec()))
        }
        _ => None,
    }
}

fn img_to_png_bytes(img: image::DynamicImage) -> Result<Vec<u8>, JsValue> {
    let mut out: Vec<u8> = Vec::new();
    img.write_to(
        &mut std::io::Cursor::new(&mut out),
        image::ImageFormat::Png,
    )
    .map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(out)
}

// ── Legacy single-string API (kept for compatibility) ─────────────────────────

#[wasm_bindgen]
pub fn encode_string(image_bytes: &[u8], message: &str) -> Result<Vec<u8>, JsValue> {
    encode_string_secure(image_bytes, message, "none", &[])
}

#[wasm_bindgen]
pub fn encode_string_secure(
    image_bytes: &[u8],
    message: &str,
    encryption: &str,
    key: &[u8],
) -> Result<Vec<u8>, JsValue> {
    let mut img =
        image::load_from_memory(image_bytes).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let secret = parse_secret(encryption, key);
    stng::encoder::Encoder::encode_string(&mut img, message, secret.as_ref())
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    img_to_png_bytes(img)
}

#[wasm_bindgen]
pub fn encode_max_capacity(image_bytes: &[u8]) -> Result<usize, JsValue> {
    let img =
        image::load_from_memory(image_bytes).map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(stng::encoder::Encoder::max_capacity(&img))
}

#[wasm_bindgen]
pub fn decode_string(image_bytes: &[u8]) -> Result<String, JsValue> {
    decode_string_secure(image_bytes, "none", &[])
}

#[wasm_bindgen]
pub fn decode_string_secure(
    image_bytes: &[u8],
    encryption: &str,
    key: &[u8],
) -> Result<String, JsValue> {
    let img =
        image::load_from_memory(image_bytes).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let secret = parse_secret(encryption, key);
    stng::decoder::Decoder::decode_string(&img, secret.as_ref())
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

// ── Multi-payload API ─────────────────────────────────────────────────────────
//
// JS <-> WASM exchange format (JSON array):
// [
//   { "name": "message", "type": "text",   "value": "plain text content" },
//   { "name": "file.png","type": "binary", "value": "<base64>" }
// ]

#[wasm_bindgen]
pub fn encode_payload(
    image_bytes: &[u8],
    entries_json: &str,
    encryption: &str,
    key: &[u8],
) -> Result<Vec<u8>, JsValue> {
    let entries: Vec<serde_json::Value> = serde_json::from_str(entries_json)
        .map_err(|e| JsValue::from_str(&format!("JSON parse error: {e}")))?;

    let mut data = Data::new();
    for entry in entries {
        let name = entry["name"].as_str().unwrap_or("entry").to_string();
        let typ  = entry["type"].as_str().unwrap_or("text");
        let val  = entry["value"].as_str().unwrap_or("");
        match typ {
            "binary" => {
                let bytes = B64.decode(val)
                    .map_err(|e| JsValue::from_str(&format!("Base64 decode error: {e}")))?;
                data.push(DataElement::binary(name, bytes));
            }
            _ => {
                data.push(DataElement::text(name, val));
            }
        }
    }

    let mut img =
        image::load_from_memory(image_bytes).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let secret = parse_secret(encryption, key);
    stng::encoder::Encoder::encode_payload(&mut img, &data, secret.as_ref())
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    img_to_png_bytes(img)
}

#[wasm_bindgen]
pub fn decode_payload(
    image_bytes: &[u8],
    encryption: &str,
    key: &[u8],
) -> Result<String, JsValue> {
    let img =
        image::load_from_memory(image_bytes).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let secret = parse_secret(encryption, key);
    let data = stng::decoder::Decoder::decode_payload(&img, secret.as_ref())
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let mut arr = Vec::new();
    for elem in &data.elements {
        let typ = match elem.data_type {
            DataType::Text   => "text",
            DataType::Binary => "binary",
        };
        let value = match elem.data_type {
            DataType::Text   => std::str::from_utf8(&elem.value)
                .unwrap_or("")
                .to_string(),
            DataType::Binary => B64.encode(&elem.value),
        };
        arr.push(serde_json::json!({
            "name":  elem.name,
            "type":  typ,
            "value": value,
        }));
    }

    serde_json::to_string(&arr).map_err(|e| JsValue::from_str(&e.to_string()))
}

