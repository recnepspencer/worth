use std::io::{self, Write};

/// One simulation's NDJSON rows in scenario order. The scenario supplies
/// schema and field order, and the writer preserves those bytes for TLC.
#[derive(Default)]
pub struct Trace {
    bytes: Vec<u8>,
}

impl Trace {
    /// Appends one single-line JSON object and its terminating newline.
    ///
    /// TLC schemas use integers, booleans, strings, records, and arrays. Encode
    /// sets as `{"__set__":[...]}`, with members in scenario-defined order.
    /// Encode fields in the schema's order to reproduce identical trace bytes.
    pub fn push_json(&mut self, row: &str) -> io::Result<()> {
        if row.trim() != row
            || row.contains(['\n', '\r'])
            || !serde_json::from_str::<serde_json::Value>(row).is_ok_and(|value| value.is_object())
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "expected a single JSON object without surrounding whitespace",
            ));
        }
        self.bytes.extend_from_slice(row.as_bytes());
        self.bytes.push(b'\n');
        Ok(())
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn write(&self, mut writer: impl Write) -> io::Result<()> {
        writer.write_all(&self.bytes)
    }
}

#[cfg(test)]
mod tests;
