//! Native-request decoding. Standalone representation decoding remains separate.
use super::{
    CanonicalValue, CanonicalValueSeed, ValueCodecDenial as D, ValueDecodeAdmission,
    ValueDecodeBounds,
};
use serde::de::DeserializeSeed;
use std::io::Write;

/// Decode through one borrowed admission callback supplied by the native owner.
/// The callback is not authority itself. The first refused grant wins, including
/// when an opaque seed catches its error and returns a value anyway.
pub fn decode_canonical_value_with_admission<Value: CanonicalValue>(
    raw: &str,
    bounds: ValueDecodeBounds,
    admission: &mut ValueDecodeAdmission<'_>,
) -> Result<Value, D> {
    if raw.len() > bounds.maximum_wire_bytes() {
        return Err(D::WireBytes);
    }
    if !Value::SUPPORTS_ADMISSION {
        return Err(D::UnsupportedAdmission);
    }
    let mut refusal = None;
    let mut latched = |work, bytes| {
        if let Some(denial) = refusal {
            return Err(denial);
        }
        let result = admission(work, bytes);
        if let Err(denial) = result {
            refusal = Some(denial);
        }
        result
    };
    let result = decode::<Value>(raw, bounds, &mut latched);
    if let Some(denial) = refusal {
        return Err(denial);
    }
    result
}

fn decode<Value: CanonicalValue>(
    raw: &str,
    bounds: ValueDecodeBounds,
    admission: &mut ValueDecodeAdmission<'_>,
) -> Result<Value, D> {
    admission(raw.len(), json_decode_storage(raw.len(), bounds)?)?;
    let mut decoder = serde_json::Deserializer::from_str(raw);
    let value = CanonicalValueSeed::<Value>::with_admission(bounds, admission)
        .deserialize(&mut decoder)
        .map_err(|_| D::Malformed)?;
    decoder.end().map_err(|_| D::Malformed)?;
    value.validate_bounds(bounds)?;
    // Compare canonical serialization directly against the admitted source.
    // This is the actual serializer, with no second owned encoding allocation.
    let mut comparison = CanonicalComparison {
        raw: raw.as_bytes(),
        cursor: 0,
        admission,
        denial: None,
    };
    if serde_json::to_writer(&mut comparison, &value).is_err() {
        return Err(comparison.denial.unwrap_or(D::Malformed));
    }
    if let Some(denial) = comparison.denial {
        return Err(denial);
    }
    if comparison.cursor != raw.len() {
        return Err(D::NonCanonical);
    }
    Ok(value)
}

fn json_decode_storage(input_bytes: usize, bounds: ValueDecodeBounds) -> Result<usize, D> {
    // Locked serde_json 1.0.154: src/de.rs:33,62 owns one reused scratch Vec.
    // src/read.rs:520-560 copies only on escapes; parse_escape at :925 contracts
    // input, and push_wtf8_codepoint at :1035 reserves four bytes per codepoint.
    // Thus required scratch <= input bytes + 4. Vec<u8> amortized growth doubles
    // capacity (minimum 8); the sum of all allocations, including coexistence
    // during reallocation, is < 4*(input+4)+16. Reserve that conservative entire
    // session charge BEFORE constructing the parser. No per-string reset/refund.
    let scratch = input_bytes
        .checked_add(4)
        .and_then(|n| n.checked_mul(4))
        .and_then(|n| n.checked_add(16))
        .ok_or(D::Admission)?;
    // serde_json src/error.rs:435,484 formats one custom diagnostic, converts
    // its String to Box<str>, then boxes ErrorImpl. serde_core 1.0.228
    // src/de/mod.rs:401-423 Debug-formats unexpected strings: <=6 bytes per
    // input byte. 128 covers JSON labels and our fixed Expected/denial text.
    // Generated errors omit static field names/lists, whose size is unrelated
    // to input. String growth costs <4*message bytes; shrinking to Box<str>
    // may allocate another message-sized block while the old one still lives.
    let message = input_bytes
        .checked_mul(6)
        .and_then(|n| n.checked_add(128))
        .ok_or(D::Admission)?;
    let diagnostic = message.checked_mul(5).ok_or(D::Admission)?;
    // ErrorCode's largest payload is Box<str> or io::Error; allow one word for
    // its discriminant and two for line/column (error.rs:230-241). An initial
    // error, fix_position replacement (:338), and one end_map/end_seq error
    // per enclosing visitor (de.rs:1754,1807,1852) can coexist over unwinding.
    let word = std::mem::size_of::<usize>();
    let error_impl = std::mem::size_of::<std::io::Error>().max(2 * word) + 3 * word;
    let error_boxes = (bounds.maximum_depth() + 3)
        .checked_mul(error_impl)
        .ok_or(D::Admission)?;
    // The comparison writer's single failing write boxes the denial plus
    // io::Error's kind and boxed error payload; its JSON box is covered above.
    let comparison_error = 4 * word + std::mem::size_of::<D>();
    scratch
        .checked_add(diagnostic)
        .and_then(|n| n.checked_add(error_boxes))
        .and_then(|n| n.checked_add(comparison_error))
        .ok_or(D::Admission)
}

struct CanonicalComparison<'a, 'b> {
    raw: &'a [u8],
    cursor: usize,
    admission: &'a mut ValueDecodeAdmission<'b>,
    denial: Option<D>,
}
impl Write for CanonicalComparison<'_, '_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let result = if let Some(denial) = self.denial {
            Err(denial)
        } else {
            (self.admission)(bytes.len(), 0).and_then(|()| {
                let end = self
                    .cursor
                    .checked_add(bytes.len())
                    .ok_or(D::NonCanonical)?;
                if self.raw.get(self.cursor..end) != Some(bytes) {
                    return Err(D::NonCanonical);
                }
                self.cursor = end;
                Ok(())
            })
        };
        if let Err(denial) = result {
            self.denial = Some(denial);
            return Err(std::io::Error::other(denial));
        }
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
