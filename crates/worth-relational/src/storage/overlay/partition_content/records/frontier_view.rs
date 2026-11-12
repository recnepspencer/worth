//! Borrow source-bound history openings instead of collecting copied frontiers.
use super::*;
use serde::{
    ser::{Error, SerializeSeq},
    Serialize, Serializer,
};
pub(in crate::storage::overlay::partition_content) struct Frontiers<'a, K: RecordKind> {
    content: &'a RecordContentIndex,
    arena: &'a RecordArena<K>,
}
impl RecordContentIndex {
    pub(in crate::storage::overlay::partition_content) fn frontier_view<'a, K: RecordKind>(
        &'a self,
        arena: &'a RecordArena<K>,
    ) -> Frontiers<'a, K> {
        Frontiers {
            content: self,
            arena,
        }
    }
}
impl<K: RecordKind> Serialize for Frontiers<'_, K>
where
    K::Meta: Serialize,
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut sequence = serializer.serialize_seq(Some(self.content.histories.len()))?;
        for (&slot, history) in self.content.histories.iter() {
            let metadata = self
                .arena
                .metadata_history_at(usize::try_from(slot).map_err(S::Error::custom)?)
                .ok_or_else(|| S::Error::custom("history frontier has no original row"))?;
            let last_value = metadata
                .len()
                .checked_sub(1)
                .and_then(|index| metadata.get(index))
                .map(|value| hash_value(b"metadata", value))
                .transpose()
                .map_err(|_| S::Error::custom("history frontier original row hash refused"))?;
            let frontier = history
                .append_frontier_view(last_value)
                .ok_or_else(|| S::Error::custom("history frontier original spine refused"))?;
            sequence.serialize_element(&(slot, frontier))?;
        }
        sequence.end()
    }
}
