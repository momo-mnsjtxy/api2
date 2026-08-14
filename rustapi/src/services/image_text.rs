use ab_glyph::{Font, FontRef, PxScale, ScaleFont};
use image::{Rgba, RgbaImage};

/// Draw UTF-8 text onto an RGBA image using a TTF/OTF font (ab_glyph).
pub fn draw_text(
    img: &mut RgbaImage,
    font_data: &[u8],
    text: &str,
    x: f32,
    y: f32,
    size: f32,
    color: [u8; 4],
) -> anyhow::Result<()> {
    let font = FontRef::try_from_slice(font_data)?;
    let scale = PxScale::from(size);
    let scaled = font.as_scaled(scale);
    let mut caret_x = x;
    let baseline = y;

    for ch in text.chars() {
        if ch == '\n' {
            continue;
        }
        let glyph = scaled.scaled_glyph(ch);
        let Some(outlined) = font.outline_glyph(glyph) else {
            caret_x += scaled.h_advance(scaled.glyph_id(ch));
            continue;
        };
        let bounds = outlined.px_bounds();
        outlined.draw(|gx, gy, cov| {
            let px = (bounds.min.x + gx as f32) as i32 + caret_x as i32;
            let py = (bounds.min.y + gy as f32) as i32 + baseline as i32;
            if px < 0 || py < 0 {
                return;
            }
            let (px, py) = (px as u32, py as u32);
            if px >= img.width() || py >= img.height() {
                return;
            }
            let src = img.get_pixel(px, py).0;
            let a = (cov * color[3] as f32) as u8;
            let blended = [
                blend(src[0], color[0], a),
                blend(src[1], color[1], a),
                blend(src[2], color[2], a),
                255,
            ];
            img.put_pixel(px, py, Rgba(blended));
        });
        caret_x += scaled.h_advance(scaled.glyph_id(ch));
    }
    Ok(())
}

fn blend(dst: u8, src: u8, a: u8) -> u8 {
    let a = a as u16;
    (((src as u16) * a + (dst as u16) * (255 - a)) / 255) as u8
}
