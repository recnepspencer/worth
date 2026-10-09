//! Planning's one owner of "this release extends the checkpoint-source head
//! of its object". Store applies the same rule in
//! `pending_wal_release::lineage`; both deny a terminal head, a head over a
//! different source basis, and a cumulative count that does not advance by
//! exactly the extending manifest.

use worth_store_physical_format::{
    BlobReclaimDescriptorV2, ReleaseCustodyHeadEntryV1, ReleaseCustodyHeadKeyV1,
    ReleasedGenerationReclaimBasisV1,
};

/// The checkpoint-source head keyed by the released object, when it has one.
pub(super) fn checkpoint_head(
    heads: &[ReleaseCustodyHeadEntryV1],
    source: ReleasedGenerationReclaimBasisV1,
) -> Option<ReleaseCustodyHeadEntryV1> {
    let key = ReleaseCustodyHeadKeyV1::new(source.object(), source.generation())?;
    heads
        .binary_search_by_key(&key, |entry| entry.key())
        .ok()
        .and_then(|index| heads.get(index))
        .copied()
}

/// `descriptor` extends `head` as the checkpoint-source head of its object: it
/// is built on the checkpoint source root itself or on a later root, never on
/// a root the checkpoint had already left behind.
pub(super) fn extends_checkpoint_head(
    head: ReleaseCustodyHeadEntryV1,
    descriptor: BlobReclaimDescriptorV2,
    count: u16,
    checkpoint_source_root_generation: u64,
) -> bool {
    checkpoint_source_root_generation <= descriptor.source_root_generation()
        && head_predecessor_matches(head, descriptor, count)
}

/// `descriptor` names `head` as its exact per-object predecessor and advances
/// the head's cumulative count by `count` records.
pub(super) fn head_predecessor_matches(
    head: ReleaseCustodyHeadEntryV1,
    descriptor: BlobReclaimDescriptorV2,
    count: u16,
) -> bool {
    !head.terminal()
        && head.source_basis_digest() == descriptor.source_basis_digest()
        && head.source_root_generation() < descriptor.source_root_generation()
        && descriptor.predecessor().is_some_and(|prior| {
            prior.descriptor_record() == head.descriptor_record()
                && prior.descriptor_frame_sha256() == head.descriptor_frame_sha256()
        })
        && head.cumulative_dropped().checked_add(u64::from(count))
            == Some(descriptor.cumulative_dropped())
}
