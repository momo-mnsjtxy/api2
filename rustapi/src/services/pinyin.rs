/// Minimal hanzi→pinyin map (same approach as oldapi common.php).
static PINYIN_DICT: &str = include_str!("../../data/pinyin_dict.txt");

pub fn pinyin_char(ch: &str) -> String {
    if let Some(pos) = PINYIN_DICT.find(ch) {
        let chunk = &PINYIN_DICT[pos..];
        if let Some(end) = chunk.find(',') {
            return chunk[ch.len()..end].to_string();
        }
    }
    String::new()
}

pub fn pinyin_text(text: &str) -> String {
    let mut out = String::new();
    for ch in text.chars() {
        let s = ch.to_string();
        if ('\u{4e00}'..='\u{9fa5}').contains(&ch) {
            let pin = pinyin_char(&s);
            if pin.is_empty() {
                out.push(ch);
            } else {
                out.push_str(&pin);
                out.push(' ');
            }
        } else {
            out.push(ch);
        }
    }
    out
}
