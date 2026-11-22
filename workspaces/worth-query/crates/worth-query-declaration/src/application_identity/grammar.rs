//! Const identity grammar shared by roots, overrides and slot fields.

/// Longest identity a namespace derives or accepts as an override.
pub const MAXIMUM_APPLICATION_IDENTITY_BYTES: usize = 256;

/// A dotted identity: printable ASCII, non-empty `.`-separated segments.
pub(super) const fn is_dotted_identity(text: &str) -> bool {
    let bytes = text.as_bytes();
    let mut index = 0;
    let mut segment_open = false;
    while index < bytes.len() {
        let byte = bytes[index];
        if !byte.is_ascii_graphic() {
            return false;
        }
        if byte == b'.' {
            if !segment_open {
                return false;
            }
            segment_open = false;
        } else {
            segment_open = true;
        }
        index += 1;
    }
    segment_open
}

/// One segment: printable ASCII without a `.`.
pub(super) const fn is_segment(text: &str) -> bool {
    let bytes = text.as_bytes();
    if bytes.is_empty() {
        return false;
    }
    let mut index = 0;
    while index < bytes.len() {
        if !bytes[index].is_ascii_graphic() || bytes[index] == b'.' {
            return false;
        }
        index += 1;
    }
    true
}

pub(super) const fn same_text(left: &str, right: &str) -> bool {
    let (left, right) = (left.as_bytes(), right.as_bytes());
    if left.len() != right.len() {
        return false;
    }
    let mut index = 0;
    while index < left.len() {
        if left[index] != right[index] {
            return false;
        }
        index += 1;
    }
    true
}

/// Whether `text` is `head.tail...`: `head`, a `.`, then `tail` as a prefix.
pub(super) const fn extends(text: &str, head: &str, tail: &str) -> bool {
    let (text, head, tail) = (text.as_bytes(), head.as_bytes(), tail.as_bytes());
    if text.len() < head.len() + 1 + tail.len() {
        return false;
    }
    let mut index = 0;
    while index < head.len() {
        if text[index] != head[index] {
            return false;
        }
        index += 1;
    }
    if text[index] != b'.' {
        return false;
    }
    let mut offset = 0;
    while offset < tail.len() {
        if text[index + 1 + offset] != tail[offset] {
            return false;
        }
        offset += 1;
    }
    true
}
