//! Heap bounds for Native's descriptive receipt JSON, never receipt authority.
use super::RelationalCommitReceipt;
use crate::branch::RelationalRecordedComponentRefusal as Denial;
use serde::de::{SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::value::RawValue;
#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug)]
pub struct RecordedRelationalJsonHeapQuote {
    retained: u64,
    temporary: u64,
}
impl RecordedRelationalJsonHeapQuote {
    pub fn retained_bytes(self) -> u64 {
        self.retained
    }
    pub fn temporary_bytes(self) -> u64 {
        self.temporary
    }
}
fn malformed() -> Denial {
    Denial::PayloadEncoding {
        detail: "recorded Native receipt JSON is malformed or exceeds allocation limits".into(),
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fields<'a> {
    #[serde(rename = "commit_id")]
    _commit_id: serde::de::IgnoredAny,
    #[serde(rename = "version_id")]
    _version_id: serde::de::IgnoredAny,
    #[serde(borrow)]
    branch_id: &'a RawValue,
    #[serde(borrow)]
    parents: &'a RawValue,
}
pub(crate) fn string_heap_bound(value: &RawValue) -> Result<usize, Denial> {
    let bytes = value.get().as_bytes();
    if bytes.first() != Some(&b'"') || bytes.last() != Some(&b'"') {
        return Err(malformed());
    }
    bytes.len().checked_sub(2).ok_or_else(malformed)
}
struct ParentCount;
impl<'de> Visitor<'de> for ParentCount {
    type Value = usize;
    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("Native parent ordinals")
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut input: A) -> Result<usize, A::Error> {
        use serde::de::Error;
        let mut count = 0usize;
        while input.next_element::<u64>()?.is_some() {
            count = count
                .checked_add(1)
                .ok_or_else(|| A::Error::custom("parent overflow"))?;
        }
        Ok(count)
    }
}
pub(crate) fn parent_heap_bound(value: &RawValue) -> Result<usize, Denial> {
    let mut input = serde_json::Deserializer::from_slice(value.get().as_bytes());
    let count = input
        .deserialize_seq(ParentCount)
        .map_err(|_| malformed())?;
    input.end().map_err(|_| malformed())?;
    if count == 0 {
        return Ok(0);
    }
    count
        .checked_next_power_of_two()
        .map(|count| count.max(4))
        .and_then(|count| count.checked_mul(std::mem::size_of::<super::CommitId>()))
        .ok_or_else(malformed)
}
impl RelationalCommitReceipt {
    /// Quote only this receipt's nested heaps; the surrounding owner quotes its
    /// actual struct/Arc. The caller must first fund serde parser scratch and
    /// retain the returned bounds through decoding. No Native admission occurs.
    pub fn recorded_json_heap_quote(
        bytes: &[u8],
    ) -> Result<RecordedRelationalJsonHeapQuote, Denial> {
        let fields: Fields<'_> = serde_json::from_slice(bytes).map_err(|_| malformed())?;
        let parents = parent_heap_bound(fields.parents)?;
        let retained = parents
            .checked_add(string_heap_bound(fields.branch_id)?)
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(malformed)?;
        Ok(RecordedRelationalJsonHeapQuote {
            retained,
            temporary: u64::try_from(parents).map_err(|_| malformed())?,
        })
    }
}
