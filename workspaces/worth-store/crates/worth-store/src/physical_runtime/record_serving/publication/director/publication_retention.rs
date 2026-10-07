//! Reconstructs publication retention from exact physical WAL groups and roots.
//! Reachable record payload is not excess retention; its WAL copy is charged.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::physical_runtime::durability::{
    PhysicalPublicationAdmission, PhysicalRetentionProfile, PhysicalWalRuntimeOwner,
    ReopenedWalPublicationGroup, WalPublicationReservation,
};
use crate::physical_runtime::record_serving::{
    admission::ReconstructedPublicationMetadata, planning::publication_metadata_reservation,
};

/// Complete reconstruction, admitted before the serving owners are installed.
pub(in crate::physical_runtime) struct AdmittedPublicationRetention(
    Arc<PhysicalPublicationAdmission>,
);

impl AdmittedPublicationRetention {
    pub(in crate::physical_runtime) fn admit(
        wal: &PhysicalWalRuntimeOwner,
        roots: &[ReconstructedPublicationMetadata],
        profile: PhysicalRetentionProfile,
        allocation: &worth_store_buffer_pool::OperationAllocationGrant,
    ) -> Result<Self, ()> {
        let admission = Arc::new(PhysicalPublicationAdmission::new(profile));
        restore(&admission, wal, roots, allocation.bytes())?;
        Ok(Self(admission))
    }

    pub(super) fn into_admission(self) -> Arc<PhysicalPublicationAdmission> {
        self.0
    }
}

fn restore(
    owner: &PhysicalPublicationAdmission,
    wal: &PhysicalWalRuntimeOwner,
    roots: &[ReconstructedPublicationMetadata],
    admitted_bytes: u64,
) -> Result<(), ()> {
    let groups = wal.take_reopened_publication_groups();
    let release_metadata = wal.take_reopened_release_metadata();
    let input_bytes = (groups.capacity() as u64)
        .checked_mul(std::mem::size_of::<ReopenedWalPublicationGroup>() as u64)
        .and_then(|bytes| {
            bytes.checked_add(
                (release_metadata.capacity() as u64).checked_mul(std::mem::size_of::<(
                    u64,
                    u64,
                    u64,
                    u64,
                )>() as u64)?,
            )
        })
        .ok_or(())?;
    let input_bytes = groups
        .iter()
        .try_fold(input_bytes, |bytes, group| {
            bytes.checked_add(group.retained_metadata_capacity_bytes())
        })
        .ok_or(())?;
    let peak = PhysicalPublicationAdmission::reopened_metadata_memory_bound(
        // Release-publication roots also enter the attribution set. Counting
        // them as full group slots conservatively covers both owner indexes.
        (groups.len() as u64)
            .checked_add(release_metadata.len() as u64)
            .ok_or(())?,
        u64::from(wal.observation().active_segment_count()),
    )
    .and_then(|bytes| bytes.checked_add(input_bytes))
    .ok_or(())?;
    if peak > admitted_bytes {
        return Err(());
    }
    let mut other_wal: BTreeMap<_, _> = wal.reopened_wal_segments().into_iter().collect();
    let mut published_roots = BTreeSet::new();
    for group in groups {
        let remaining = other_wal.get_mut(&group.segment()).ok_or(())?;
        *remaining = remaining.checked_sub(group.encoded_wal_bytes()).ok_or(())?;
        let (reservation, published) = price_group(&group, roots)?;
        if let Some((generation, _)) = published {
            if !published_roots.insert(generation) {
                return Err(());
            }
        }
        owner.restore_wal_publication(reservation, published.map(|(_, bytes)| bytes))?;
    }
    for (root, bytes, segment, generation) in release_metadata {
        if bytes == 0 {
            continue;
        }
        if let Some(published) = lookup_root(roots, root) {
            if published.bytes != bytes || !published_roots.insert(root) {
                return Err(());
            }
        }
        let remaining = other_wal.get_mut(&(segment, generation)).ok_or(())?;
        *remaining = remaining.checked_add(bytes).ok_or(())?;
    }
    // All remaining frames are maintenance/copy records. Retain their actual
    // physical bytes and release them with the same whole-segment owner.
    for ((segment, generation), bytes) in other_wal {
        owner.restore_sealed_publication(segment, generation, bytes)?;
    }
    Ok(())
}

fn price_group(
    group: &ReopenedWalPublicationGroup,
    roots: &[ReconstructedPublicationMetadata],
) -> Result<(WalPublicationReservation, Option<(u64, u64)>), ()> {
    let source_generation = group.members().first().ok_or(())?.source_root_generation();
    let source = lookup_root(roots, source_generation).ok_or(())?;
    let mut ceiling = 0_u64;
    for member in group.members() {
        if member.source_root_generation() != source_generation {
            return Err(());
        }
        let entries = source
            .record_count
            .checked_add(member.inserted_records())
            .ok_or(())?;
        let capacity = member
            .successor_manifest_capacity()
            .unwrap_or(source.node_capacity);
        if capacity < 2 {
            return Err(());
        }
        ceiling = ceiling
            .checked_add(publication_metadata_reservation(capacity, entries))
            .ok_or(())?;
    }
    let result_generation = source_generation.checked_add(1).ok_or(())?;
    let published = lookup_root(roots, result_generation).map(|root| (root.generation, root.bytes));
    let reservation = WalPublicationReservation::new(
        group.segment(),
        group.lsn_range(),
        group.encoded_wal_bytes(),
        ceiling,
    )
    .ok_or(())?;
    Ok((reservation, published))
}

fn lookup_root(
    roots: &[ReconstructedPublicationMetadata],
    generation: u64,
) -> Option<&ReconstructedPublicationMetadata> {
    roots
        .binary_search_by_key(&generation, |root| root.generation)
        .ok()
        .map(|index| &roots[index])
}
