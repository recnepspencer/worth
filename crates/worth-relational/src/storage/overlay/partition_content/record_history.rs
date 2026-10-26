//! Reuse a record's history tree only while its omitted prefix is unchanged.
use super::{digest_index::DigestIndex, hash_value, PartitionContentDigestError};
use crate::storage::substrate::{RecordArena, RecordHistoryPrefix, RecordKind, SharedColumn};

pub(super) fn update<K: RecordKind>(
    arena: &RecordArena<K>,
    previous: Option<&RecordArena<K>>,
    slot: usize,
    cached: &DigestIndex,
) -> Result<(DigestIndex, u64), PartitionContentDigestError>
where
    K::Meta: serde::Serialize,
{
    let physical = arena.physical_index(slot).expect("selected record exists");
    let prefix = arena.history_prefixes[physical].as_ref();
    let old_physical = previous.and_then(|old| old.physical_index(slot));
    let old_prefix = previous
        .zip(old_physical)
        .and_then(|(old, physical)| old.history_prefixes[physical].as_ref());
    let empty = SharedColumn::default();
    let (mut history, old, offset) = if previous.is_some() && same_basis(prefix, old_prefix) {
        let old = previous
            .and_then(|old| old.metadata_history_at(slot))
            .unwrap_or(&empty);
        let offset = cached
            .next_history_index()
            .checked_sub(old.len() as u128)
            .ok_or(PartitionContentDigestError::InvalidHistoryCoverage)?;
        (cached.clone(), old, offset)
    } else if let Some(prefix) = prefix {
        let history = DigestIndex::from_append_frontier(&prefix.history_frontier)
            .ok_or(PartitionContentDigestError::InvalidHistoryCoverage)?;
        let offset = history
            .next_history_index()
            .checked_sub(1)
            .ok_or(PartitionContentDigestError::InvalidHistoryCoverage)?;
        (history, &empty, offset)
    } else {
        (DigestIndex::default(), &empty, 0)
    };
    let current = &arena.metadata_history[physical];
    offset
        .checked_add(current.len().max(old.len()) as u128)
        .ok_or(PartitionContentDigestError::InvalidHistoryCoverage)?;
    if prefix.is_some() && current.is_empty() {
        return Err(PartitionContentDigestError::InvalidHistoryCoverage);
    }
    let mut hashed = 0;
    current.visit_changed_values(old, &mut |index, value, _| {
        history.set(
            offset + index as u128,
            value.map(|value| hash_value(b"metadata", value)),
        );
        hashed += u64::from(value.is_some());
    });
    Ok((history, hashed))
}

fn same_basis(current: Option<&RecordHistoryPrefix>, old: Option<&RecordHistoryPrefix>) -> bool {
    match (current, old) {
        (None, None) => true,
        (Some(current), Some(old)) => {
            current.member == old.member
                && current.partition_id == old.partition_id
                && current.schema_carrier_digest == old.schema_carrier_digest
                && current.row_index == old.row_index
                && current.history_frontier.as_ref() == old.history_frontier.as_ref()
        }
        _ => false,
    }
}
