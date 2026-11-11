//! Canonical owned diagnostic fields with explicit contiguous decode storage.
use super::RelationalDiagnosticValue;
use serde::{
    de::{Error, MapAccess, Visitor},
    ser::SerializeMap,
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::{alloc::Layout, collections::BTreeMap, fmt};

type Field = (String, RelationalDiagnosticValue);

/// Ordered diagnostic object fields. Construction and cloning own their data;
/// borrowed access cannot mutate the canonical key order.
#[derive(Clone, Default, PartialEq, Eq)]
pub struct DiagnosticObjectFields {
    fields: Box<[Field]>,
}

/// Contiguous storage only; decoded key strings and value-owned heaps are
/// separate obligations of the requesting Native decoder.
pub(crate) struct DiagnosticObjectFieldsStorageBounds {
    /// Maximum final pair backing; duplicate input names can reduce it.
    pub(crate) retained_fields_bytes: usize,
    /// Additional backing above the retained bound at the decode peak.
    pub(crate) transient_decode_bytes: usize,
}

impl DiagnosticObjectFields {
    pub fn get(&self, key: &str) -> Option<&RelationalDiagnosticValue> {
        let index = self
            .fields
            .binary_search_by(|field| field.0.as_str().cmp(key))
            .ok()?;
        Some(&self.fields[index].1)
    }

    pub fn len(&self) -> usize {
        self.fields.len()
    }
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    pub fn iter(
        &self,
    ) -> impl DoubleEndedIterator<Item = (&String, &RelationalDiagnosticValue)> + ExactSizeIterator
    {
        self.fields.iter().map(field_refs)
    }

    pub fn keys(&self) -> impl DoubleEndedIterator<Item = &String> + ExactSizeIterator {
        self.fields.iter().map(|field| &field.0)
    }

    /// Checked storage for an actual verified input entry count, including
    /// controlled row growth, rows/output overlap and boxing overlap.
    pub(crate) fn decode_storage_bounds(
        count: usize,
    ) -> Option<DiagnosticObjectFieldsStorageBounds> {
        if count == 0 {
            return Some(DiagnosticObjectFieldsStorageBounds {
                retained_fields_bytes: 0,
                transient_decode_bytes: 0,
            });
        }
        let capacity = count.checked_next_power_of_two()?;
        let rows = Layout::array::<DecodedField>(capacity).ok()?.size();
        let old_rows = Layout::array::<DecodedField>(capacity / 2).ok()?.size();
        let fields = Layout::array::<Field>(count).ok()?.size();
        let growth_peak = rows.checked_add(old_rows)?;
        // Output conversion may briefly hold both the Vec and final box.
        // Input rows remain owned while their fields move into that output.
        let boxing_peak = rows.checked_add(fields.checked_mul(2)?)?;
        Some(DiagnosticObjectFieldsStorageBounds {
            retained_fields_bytes: fields,
            transient_decode_bytes: growth_peak.max(boxing_peak).checked_sub(fields)?,
        })
    }
}

impl From<BTreeMap<String, RelationalDiagnosticValue>> for DiagnosticObjectFields {
    fn from(fields: BTreeMap<String, RelationalDiagnosticValue>) -> Self {
        Self {
            fields: fields.into_iter().collect::<Vec<_>>().into_boxed_slice(),
        }
    }
}

impl FromIterator<Field> for DiagnosticObjectFields {
    fn from_iter<T: IntoIterator<Item = Field>>(fields: T) -> Self {
        let mut rows = Vec::new();
        for (key, value) in fields {
            if rows.len() == rows.capacity() {
                let capacity = next_capacity(rows.capacity()).expect(/* Proof: DecodedField is nonzero and larger than two bytes; a Vec of these rows cannot have capacity above isize::MAX / size_of::<DecodedField>(), so doubling its capacity is representable in usize. Ordinary allocation refusal retains the standard unbudgeted constructor posture. */ "diagnostic field capacity overflow");
                rows.reserve_exact(capacity - rows.len());
            }
            let ordinal = rows.len();
            rows.push(DecodedField {
                key,
                value,
                ordinal,
            });
        }
        let count = sort_and_count(&mut rows);
        canonical_fields(rows, Vec::with_capacity(count))
    }
}

impl fmt::Debug for DiagnosticObjectFields {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_map().entries(self.iter()).finish()
    }
}

impl<'a> IntoIterator for &'a DiagnosticObjectFields {
    type Item = (&'a String, &'a RelationalDiagnosticValue);
    type IntoIter = std::iter::Map<std::slice::Iter<'a, Field>, fn(&'a Field) -> Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.fields.iter().map(field_refs)
    }
}

impl IntoIterator for DiagnosticObjectFields {
    type Item = Field;
    type IntoIter = std::vec::IntoIter<Field>;
    fn into_iter(self) -> Self::IntoIter {
        self.fields.into_vec().into_iter()
    }
}

impl Serialize for DiagnosticObjectFields {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.fields.len()))?;
        for (key, value) in self {
            map.serialize_entry(key, value)?;
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for DiagnosticObjectFields {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(ObjectVisitor)
    }
}

struct DecodedField {
    key: String,
    value: RelationalDiagnosticValue,
    ordinal: usize,
}

struct ObjectVisitor;
impl<'de> Visitor<'de> for ObjectVisitor {
    type Value = DiagnosticObjectFields;
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a map of diagnostic field names to typed values")
    }
    fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
        let mut rows = Vec::new();
        // Untrusted map hints never decide allocation. Grow from actual rows.
        while let Some((key, value)) = map.next_entry()? {
            if rows.len() == rows.capacity() {
                let capacity = next_capacity(rows.capacity())
                    .ok_or_else(|| M::Error::custom("diagnostic field capacity overflow"))?;
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
        let count = sort_and_count(&mut rows);
        let mut fields = Vec::new();
        fields.try_reserve_exact(count).map_err(M::Error::custom)?;
        Ok(canonical_fields(rows, fields))
    }
}

fn next_capacity(capacity: usize) -> Option<usize> {
    if capacity == 0 {
        Some(1)
    } else {
        capacity.checked_mul(2)
    }
}

fn sort_and_count(rows: &mut [DecodedField]) -> usize {
    rows.sort_unstable_by(|left, right| {
        left.key
            .cmp(&right.key)
            .then_with(|| left.ordinal.cmp(&right.ordinal))
    });
    usize::from(!rows.is_empty())
        + rows
            .windows(2)
            .filter(|pair| pair[0].key != pair[1].key)
            .count()
}

fn canonical_fields(rows: Vec<DecodedField>, mut fields: Vec<Field>) -> DiagnosticObjectFields {
    let mut rows = rows.into_iter();
    if let Some(mut first) = rows.next() {
        for row in rows {
            if row.key == first.key {
                // Match BTreeMap: the first owned key survives; the last value wins.
                first.value = row.value;
            } else {
                fields.push((first.key, first.value));
                first = row;
            }
        }
        fields.push((first.key, first.value));
    }
    DiagnosticObjectFields {
        fields: fields.into_boxed_slice(),
    }
}

fn field_refs(field: &Field) -> (&String, &RelationalDiagnosticValue) {
    (&field.0, &field.1)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn diagnostic_object_storage_bounds_cover_typed_growth_overlap_and_refuse_overflow() {
        let empty = DiagnosticObjectFields::decode_storage_bounds(0).unwrap();
        assert_eq!(
            (empty.retained_fields_bytes, empty.transient_decode_bytes),
            (0, 0)
        );
        for count in [1usize, 2, 3, 4, 5, 8, 9] {
            let bounds = DiagnosticObjectFields::decode_storage_bounds(count).unwrap();
            assert_eq!(
                bounds.retained_fields_bytes,
                Layout::array::<Field>(count).unwrap().size()
            );
            let rows = Layout::array::<DecodedField>(count.next_power_of_two())
                .unwrap()
                .size();
            let peak = bounds.retained_fields_bytes + bounds.transient_decode_bytes;
            assert!(peak >= rows + 2 * bounds.retained_fields_bytes);
        }
        assert!(DiagnosticObjectFields::decode_storage_bounds(usize::MAX).is_none());
        let beyond_layout = isize::MAX as usize / std::mem::size_of::<DecodedField>() + 1;
        assert!(DiagnosticObjectFields::decode_storage_bounds(beyond_layout).is_none());
    }
}
