//! Contiguous staging preserves the legacy map grammar before tree admission.
use super::{AspectValue, FieldKey};
use serde::{
    de::{Error, MapAccess, Visitor},
    Deserializer,
};
use std::fmt;
#[cfg(test)]
mod tests;

pub(super) struct DecodedField {
    key: FieldKey,
    value: AspectValue,
    ordinal: usize,
}

pub(super) fn deserialize<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Box<[(FieldKey, AspectValue)]>, D::Error> {
    deserializer.deserialize_map(FieldMapVisitor)
}

struct FieldMapVisitor;
impl<'de> Visitor<'de> for FieldMapVisitor {
    type Value = Box<[(FieldKey, AspectValue)]>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a map of struct field keys to aspect values")
    }

    fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
        let mut rows: Vec<DecodedField> = Vec::new();
        // size_hint is supplied by the input. Growth follows actual entries.
        while let Some((key, value)) = map.next_entry()? {
            if rows.len() == rows.capacity() {
                let capacity = if rows.capacity() == 0 {
                    1
                } else {
                    rows.capacity()
                        .checked_mul(2)
                        .ok_or_else(|| M::Error::custom("struct field capacity overflow"))?
                };
                rows.try_reserve_exact(capacity - rows.len())
                    .map_err(M::Error::custom)?;
            }
            let ordinal = rows.len();
            rows.push(DecodedField {
                key,
                value,
                ordinal,
            });
        }
        canonical_fields(rows)
    }
}

fn canonical_fields<E: Error>(
    mut rows: Vec<DecodedField>,
) -> Result<Box<[(FieldKey, AspectValue)]>, E> {
    rows.sort_unstable_by(|left, right| {
        left.key
            .cmp(&right.key)
            .then_with(|| left.ordinal.cmp(&right.ordinal))
    });
    let count = usize::from(!rows.is_empty())
        + rows
            .windows(2)
            .filter(|pair| pair[0].key != pair[1].key)
            .count();
    let mut fields = Vec::new();
    fields.try_reserve_exact(count).map_err(E::custom)?;
    let mut rows = rows.into_iter();
    if let Some(mut first) = rows.next() {
        for row in rows {
            if row.key == first.key {
                // BTreeMap insertion retains the first key and the last value.
                first.value = row.value;
            } else {
                fields.push((first.key, first.value));
                first = row;
            }
        }
        fields.push((first.key, first.value));
    }
    Ok(fields.into_boxed_slice())
}
