pub fn get_bro(ua: &str) -> String {
    let sys = ua;
    if let Some(cap) = regex_first(sys, r"(?i)Firefox/([^;)]+)") {
        return format!("Firefox({cap})");
    }
    if let Some(cap) = regex_first(sys, r"(?i)Maxthon/([\d\.]+)") {
        return format!("傲游({cap})");
    }
    if let Some(cap) = regex_first(sys, r"(?i)MSIE\s+([^;)]+)") {
        return format!("IE({cap})");
    }
    if let Some(cap) = regex_first(sys, r"(?i)OPR/([\d\.]+)") {
        return format!("Opera({cap})");
    }
    if let Some(cap) = regex_first(sys, r"(?i)Edge/([\d\.]+)") {
        return format!("Edge({cap})");
    }
    if let Some(cap) = regex_first(sys, r"(?i)Chrome/([\d\.]+)") {
        return format!("Chrome({cap})");
    }
    if sys.to_ascii_lowercase().contains("rv:") && sys.to_ascii_lowercase().contains("gecko") {
        if let Some(cap) = regex_first(sys, r"(?i)rv:([\d\.]+)") {
            return format!("IE({cap})");
        }
    }
    if let Some(cap) = regex_first(sys, r"(?i)Safari/([a-zA-Z0-9.]+)") {
        return format!("Safari({cap})");
    }
    "未知(版本)".into()
}

pub fn get_os_info(ua: &str) -> (String, String) {
    let ua_l = ua.to_ascii_lowercase();
    if ua_l.contains("win") {
        if ua.contains("Windows NT 10.0") {
            return ("Windows 10".into(), "windows_win10".into());
        }
        if ua.contains("Windows NT 6.1") {
            return ("Windows 7".into(), "windows_win7".into());
        }
        if ua.contains("Windows NT 5.1") {
            return ("Windows XP".into(), "windows".into());
        }
        if ua.contains("Windows NT 6.2") {
            return ("Windows 8".into(), "windows_win8".into());
        }
        if ua.contains("Windows NT 6.3") {
            return ("Windows 8.1".into(), "windows_win8".into());
        }
        if ua.contains("Windows NT 6.0") {
            return ("Windows Vista".into(), "windows_vista".into());
        }
        return ("Windows".into(), "windows".into());
    }
    if ua_l.contains("android") {
        return ("Android".into(), "android".into());
    }
    if ua_l.contains("iphone") {
        return ("iPhone".into(), "iPhone".into());
    }
    if ua_l.contains("ipad") {
        return ("iPad".into(), "ipad".into());
    }
    if ua_l.contains("mac os") || ua_l.contains("macintosh") {
        return ("Mac OS".into(), "macos".into());
    }
    if ua_l.contains("linux") {
        return ("Linux".into(), "linux".into());
    }
    ("未知".into(), "未知".into())
}

fn regex_first(text: &str, pattern: &str) -> Option<String> {
    // lightweight without regex crate dependency for hot path alternatives:
    // use simple contains + split for common cases; fallback manual.
    // For fidelity use `regex` — add later if needed. Manual extract:
    extract_after(text, pattern)
}

fn extract_after(text: &str, pattern: &str) -> Option<String> {
    // Very small subset parser for patterns we emit above.
    let markers = [
        ("Firefox/", true),
        ("Maxthon/", true),
        ("MSIE ", true),
        ("OPR/", true),
        ("Edge/", true),
        ("Chrome/", true),
        ("rv:", true),
        ("Safari/", true),
    ];
    for (m, _) in markers {
        if pattern
            .to_ascii_lowercase()
            .contains(&m.to_ascii_lowercase())
        {
            if let Some(pos) = text.find(m) {
                let rest = &text[pos + m.len()..];
                let end = rest
                    .find(|c: char| c == ' ' || c == ';' || c == ')' || c == '"')
                    .unwrap_or(rest.len());
                return Some(rest[..end].to_string());
            }
        }
    }
    None
}
