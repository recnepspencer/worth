//! Reuse a record's history tree only while its omitted prefix is unchanged.
use super::{hash_value, PartitionContentDigestError};
use crate::storage::substrate::DigestIndex;
use crate::storage::substrate::{RecordArena, RecordHistoryPrefix, RecordKind, SharedColumn};

pub(super) fn update<K: RecordKind>(
    arena: &RecordArena<K>,
    previous: &RecordArena<K>,
    slot: usize,
    cached: &DigestIndex,
) -> Result<(DigestIndex, u64), PartitionContentDigestError>
where
    K::Meta: serde::Serialize,
{
    let physical = arena
        .physical_index(slot)
        .ok_or(PartitionContentDigestError::InvalidHistoryCoverage)?;
    let prefix = arena.history_prefixes[physical].as_ref();
    let old_physical = previous.physical_index(slot);
    let old_prefix = old_physical.and_then(|physical| previous.history_prefixes[physical].as_ref());
    let empty = SharedColumn::default();
    let (mut history, old, offset) = if same_basis(prefix, old_prefix) {
        let old = previous.metadata_history_at(slot).unwrap_or(&empty);
        let offset = cached
            .next_history_index()
            .ok_or(PartitionContentDigestError::InvalidHistoryCoverage)?
            .checked_sub(old.len() as u128)
            .ok_or(PartitionContentDigestError::InvalidHistoryCoverage)?;
        (cached.clone(), old, offset)
    } else if let Some(prefix) = prefix {
        let history = DigestIndex::from_append_frontier(&prefix.history_opening.frontier)
            .ok_or(PartitionContentDigestError::InvalidHistoryCoverage)?;
        let offset = history
            .next_history_index()
            .ok_or(PartitionContentDigestError::InvalidHistoryCoverage)?
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
    let mut failure = None;
    current.visit_changed_values(old, &mut |index, value, _| {
        if failure.is_some() {
            return;
        }
        let digest = match value
            .map(|value| hash_value(b"metadata", value))
            .transpose()
        {
            Ok(digest) => digest,
            Err(denial) => {
                failure = Some(denial);
                return;
            }
        };
        if let Err(denial) = history.set(offset + index as u128, digest) {
            failure = Some(denial.into());
        }
        hashed += u64::from(value.is_some());
    });
    if let Some(denial) = failure {
        return Err(denial);
    }
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
                && (current.history_opening.id() == old.history_opening.id()
                    || current.history_opening.as_ref() == old.history_opening.as_ref())
        }
        _ => false,
    }
}

/// Rebuild a record history only inside explicit cold admission.
pub(super) fn rebuild<K: RecordKind>(
    lane: &super::ReconstructionLane<'_>,
    work: &mut super::ReconstructionWork<'_, '_>,
    request: super::request_admission::Request<'_, '_, '_>,
    arena: &RecordArena<K>,
    slot: usize,
) -> Result<(DigestIndex, u64), PartitionContentDigestError>
where
    K::Meta: serde::Serialize,
{
    let physical = arena
        .physical_index(slot)
        .ok_or(PartitionContentDigestError::InvalidHistoryCoverage)?;
    let prefix = arena.history_prefixes[physical].as_ref();
    let current = &arena.metadata_history[physical];
    if prefix.is_none() {
        work.prepare(
            current.len() as u64,
            (current.len() * size_of::<(u128, [u8; 32])>()) as u64,
        )?;
        let mut entries = Vec::with_capacity(current.len());
        for (index, value) in current.iter().enumerate() {
            entries.push((
                index as u128,
                super::admission::hash_controlled(b"metadata", value, work)?,
            ));
        }
        let (units, _) = DigestIndex::construction_quote(entries.len())
            .ok_or(PartitionContentDigestError::AllocationAccountingOverflow)?;
        work.prepare(units, 0)?;
        return Ok((
            DigestIndex::from_sorted_entries_for_request(lane, &entries, request)?,
            current.len() as u64,
        ));
    }
    let (mut history, offset) = if let Some(prefix) = prefix {
        let (units, _) = DigestIndex::construction_quote(
            prefix
                .history_opening
                .frontier
                .siblings
                .len()
                .checked_add(current.len())
                .and_then(|n| n.checked_add(1))
                .ok_or(PartitionContentDigestError::AllocationAccountingOverflow)?,
        )
        .ok_or(PartitionContentDigestError::AllocationAccountingOverflow)?;
        work.prepare(units, 0)?;
        let history =
            DigestIndex::from_append_frontier_using(&prefix.history_opening.frontier, request)?;
        let offset = history
            .next_history_index()
            .and_then(|n| n.checked_sub(1))
            .ok_or(PartitionContentDigestError::InvalidHistoryCoverage)?;
        (history, offset)
    } else {
        (DigestIndex::default(), 0)
    };
    offset
        .checked_add(current.len() as u128)
        .ok_or(PartitionContentDigestError::InvalidHistoryCoverage)?;
    if prefix.is_some() && current.is_empty() {
        return Err(PartitionContentDigestError::InvalidHistoryCoverage);
    }
    let mut hashed = 0;
    for (index, value) in current.iter().enumerate() {
        let key = offset + index as u128;
        let (units, _) = history.edit_quote(key);
        work.prepare(units, 0)?;
        let digest = super::admission::hash_controlled(b"metadata", value, work)?;
        history.set(key, Some(digest))?;
        hashed += 1;
    }
    Ok((history, hashed))
}
