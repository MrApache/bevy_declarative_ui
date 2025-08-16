pub(super) fn is_name_start_char(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_' || byte == b':'
}

pub(super) fn is_name_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_' | b':')
}

pub(super) fn is_valid_xml_text_char(c: char) -> bool {
    match c {
        '\u{9}'
        | '\u{A}'
        | '\u{D}'
        | '\u{20}'..='\u{D7FF}'
        | '\u{E000}'..='\u{FFFD}'
        | '\u{10000}'..='\u{10FFFF}' => true,
        _ => false,
    }
}
