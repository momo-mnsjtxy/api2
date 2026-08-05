use image::{ImageBuffer, Luma};
use qrcode::QrCode;

pub fn render_png(text: &str, size: u32) -> anyhow::Result<Vec<u8>> {
    let code = QrCode::new(text.as_bytes())?;
    let img = code
        .render::<Luma<u8>>()
        .max_dimensions(size.max(64), size.max(64))
        .build();
    let mut buf = Vec::new();
    let dynimg = image::DynamicImage::ImageLuma8(ImageBuffer::from(img));
    dynimg.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)?;
    Ok(buf)
}

pub async fn read_qr_from_url(_client: &reqwest::Client, _url: &str) -> Option<String> {
    // Full zxing port not bundled; keep API shape.
    None
}
