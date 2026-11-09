use crate::WorthServerCompatibilityRead;
use serde_json::{Map, Number, Value};
use worth_foundational::facade::AspectValue;

#[derive(Debug)]
pub(super) struct WorthServerStreamCursor {
    row_count: usize,
    row_index: usize,
    pending_fragment: WorthServerPendingFragment,
    emitted_open_bracket: bool,
    emitted_close_bracket: bool,
    need_row_separator: bool,
}

#[derive(Debug)]
enum WorthServerPendingFragment {
    None,
    Static { bytes: &'static [u8], offset: usize },
    Row { bytes: Vec<u8>, offset: usize },
}

impl WorthServerStreamCursor {
    pub(super) fn from_read(read: &WorthServerCompatibilityRead) -> Self {
        Self {
            row_count: read.read_result().rows().len(),
            row_index: 0,
            pending_fragment: WorthServerPendingFragment::None,
            emitted_open_bracket: false,
            emitted_close_bracket: false,
            need_row_separator: false,
        }
    }

    pub(super) fn next_chunk(
        &mut self,
        read: &WorthServerCompatibilityRead,
        chunk_bytes: usize,
    ) -> Result<Option<Vec<u8>>, serde_json::Error> {
        if self.is_done() {
            return Ok(None);
        }
        let mut chunk = Vec::new();
        while chunk.len() < chunk_bytes && !self.is_done() {
            self.ensure_pending_fragment(read)?;
            let piece = self.take_from_pending(chunk_bytes - chunk.len());
            chunk.extend_from_slice(&piece);
        }
        Ok(Some(chunk))
    }

    pub(super) fn is_done(&self) -> bool {
        self.emitted_open_bracket
            && self.emitted_close_bracket
            && matches!(self.pending_fragment, WorthServerPendingFragment::None)
    }

    fn ensure_pending_fragment(
        &mut self,
        read: &WorthServerCompatibilityRead,
    ) -> Result<(), serde_json::Error> {
        if !matches!(self.pending_fragment, WorthServerPendingFragment::None) {
            return Ok(());
        }
        if !self.emitted_open_bracket {
            self.pending_fragment = WorthServerPendingFragment::Static {
                bytes: b"[",
                offset: 0,
            };
            self.emitted_open_bracket = true;
            return Ok(());
        }
        if self.row_index < self.row_count {
            if self.need_row_separator {
                self.pending_fragment = WorthServerPendingFragment::Static {
                    bytes: b",",
                    offset: 0,
                };
                self.need_row_separator = false;
                return Ok(());
            }
            let row = &read.read_result().rows()[self.row_index];
            self.pending_fragment = WorthServerPendingFragment::Row {
                bytes: serde_json::to_vec(&external_row_json(
                    row.terminal_field_value_projection(),
                ))?,
                offset: 0,
            };
            self.row_index += 1;
            return Ok(());
        }
        if !self.emitted_close_bracket {
            self.pending_fragment = WorthServerPendingFragment::Static {
                bytes: b"]",
                offset: 0,
            };
            self.emitted_close_bracket = true;
        }
        Ok(())
    }

    fn take_from_pending(&mut self, max_bytes: usize) -> Vec<u8> {
        match &mut self.pending_fragment {
            WorthServerPendingFragment::Static { bytes, offset } => {
                let take = (bytes.len() - *offset).min(max_bytes);
                let piece = bytes[*offset..*offset + take].to_vec();
                *offset += take;
                if *offset == bytes.len() {
                    self.pending_fragment = WorthServerPendingFragment::None;
                }
                piece
            }
            WorthServerPendingFragment::Row { bytes, offset } => {
                let take = (bytes.len() - *offset).min(max_bytes);
                let piece = bytes[*offset..*offset + take].to_vec();
                *offset += take;
                if *offset == bytes.len() {
                    self.pending_fragment = WorthServerPendingFragment::None;
                    self.need_row_separator = self.row_index < self.row_count;
                }
                piece
            }
            WorthServerPendingFragment::None => Vec::new(),
        }
    }
}

pub(super) fn estimate_payload_bytes(
    read: &WorthServerCompatibilityRead,
) -> Result<usize, serde_json::Error> {
    let mut bytes = 2usize;
    for (index, row) in read.read_result().rows().iter().enumerate() {
        if index > 0 {
            bytes += 1;
        }
        bytes +=
            serde_json::to_vec(&external_row_json(row.terminal_field_value_projection()))?.len();
    }
    Ok(bytes)
}

pub(super) fn materialize_payload_bytes(
    read: &WorthServerCompatibilityRead,
) -> Result<Vec<u8>, serde_json::Error> {
    let mut payload = Vec::with_capacity(estimate_payload_bytes(read)?);
    payload.push(b'[');
    for (index, row) in read.read_result().rows().iter().enumerate() {
        if index > 0 {
            payload.push(b',');
        }
        payload.extend_from_slice(&serde_json::to_vec(&external_row_json(
            row.terminal_field_value_projection(),
        ))?);
    }
    payload.push(b']');
    Ok(payload)
}

fn external_row_json(
    terminal_projection: std::collections::BTreeMap<String, AspectValue>,
) -> Value {
    let mut root = Map::new();
    for (field_path, value) in terminal_projection {
        insert_field_value(
            &mut root,
            field_path.split('.').collect(),
            aspect_value_json(&value),
        );
    }
    Value::Object(root)
}

fn insert_field_value(target: &mut Map<String, Value>, path: Vec<&str>, value: Value) {
    if path.is_empty() {
        return;
    }
    if path.len() == 1 {
        target.insert(path[0].to_string(), value);
        return;
    }
    let entry = target
        .entry(path[0].to_string())
        .or_insert_with(|| Value::Object(Map::new()));
    match entry {
        Value::Object(object) => insert_field_value(object, path[1..].to_vec(), value),
        _ => {
            let mut object = Map::new();
            insert_field_value(&mut object, path[1..].to_vec(), value);
            *entry = Value::Object(object);
        }
    }
}

fn aspect_value_json(value: &AspectValue) -> Value {
    match value {
        AspectValue::Null => Value::Null,
        AspectValue::Bool(value) => Value::Bool(*value),
        AspectValue::Int8(value) => Value::Number(Number::from(*value)),
        AspectValue::Int16(value) => Value::Number(Number::from(*value)),
        AspectValue::Int32(value) => Value::Number(Number::from(*value)),
        AspectValue::Int64(value) => Value::Number(Number::from(*value)),
        AspectValue::UInt8(value) => Value::Number(Number::from(*value)),
        AspectValue::UInt16(value) => Value::Number(Number::from(*value)),
        AspectValue::UInt32(value) => Value::Number(Number::from(*value)),
        AspectValue::UInt64(value) => Value::Number(Number::from(*value)),
        AspectValue::String(value) => Value::String(match value {
            worth_foundational::facade::InternedString::Raw(text) => text.clone(),
            worth_foundational::facade::InternedString::Stored(text) => text.as_str().to_owned(),
            worth_foundational::facade::InternedString::Symbol(symbol) => {
                format!("symbol:{}", symbol.0)
            }
        }),
        other => Value::String(format!("{other:?}")),
    }
}
