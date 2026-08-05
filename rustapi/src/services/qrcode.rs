use image::{ImageBuffer, Luma};
use qrcode::QrCode;
use reqwest::Client;

pub fn render_png(text: &str, size: u32) -> anyhow::Result<Vec<u8>> {
    let code = QrCode::new(text.as_bytes())?;
    let img = code
        .render::<Luma<u8>>()
        .max_dimensions(size.max(64), size.max(64))
        .build();
    let mut buf = Vec::new();
    let dynimg = image::DynamicImage::ImageLuma8(ImageBuffer::from(img));
    dynimg.write_to(
        &mut std::io::Cursor::new(&mut buf),
        image::ImageFormat::Png,
    )?;
    Ok(buf)
}

pub async fn read_qr_from_url(client: &Client, url: &str) -> Option<String> {
    let bytes = client.get(url).send().await.ok()?.bytes().await.ok()?;
    decode_qr_bytes(&bytes)
}

pub fn decode_qr_bytes(bytes: &[u8]) -> Option<String> {
    let img = image::load_from_memory(bytes).ok()?.to_luma8();
    let mut decoder = quircs::Quirc::default();
    let codes = decoder.identify(
        img.width() as usize,
        img.height() as usize,
        img.as_raw(),
    );
    for code in codes.flatten() {
        if let Ok(decoded) = code.decode() {
            let payload = decoded.payload;
            if let Ok(s) = String::from_utf8(payload.clone()) {
                return Some(s);
            }
            return Some(String::from_utf8_lossy(&payload).into_owned());
        }
    }
    None
}
