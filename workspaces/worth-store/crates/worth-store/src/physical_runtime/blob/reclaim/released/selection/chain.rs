//! Current selected release closure: older controls may be checkpoint-pruned.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    verify_release_custody_head_controls, BlobReclaimDescriptorV3, BlobReclaimSourceBasisV1,
    BlobReclaimSourceKind, BlobRecordKind, DropSetManifestV3, ReleaseCustodyHeadControlIdentityV1,
    ReleaseCustodyHeadEntryV1, ReleasedDropPredecessorV1, SelectedRecordContentClass,
};

use crate::physical_runtime::{durability::SelectedReleaseHeadBasis, PhysicalRecordReader};

use super::super::super::{scan, BlobReclaimDeferral, BlobReclaimFailure, BlobReclaimLimits};
use super::inventory::SelectedReleaseInventory;

pub(super) struct ValidatedReleaseChain {
    pub(super) predecessor: Option<ReleasedDropPredecessorV1>,
    pub(super) cumulative_dropped: u64,
    pub(super) terminal: bool,
    pub(super) settled_absence_authority: bool,
}

pub(super) fn validate(
    reader: &PhysicalRecordReader,
    inventory: &SelectedReleaseInventory,
    selected_head: SelectedReleaseHeadBasis,
    limits: BlobReclaimLimits,
    scratch: &mut [u8],
    work: &mut scan::ReclaimInspectionWork,
) -> Result<ValidatedReleaseChain, BlobReclaimFailure> {
    if selected_head.root() != reader.protected_root().root() {
        return Err(BlobReclaimFailure::ConflictingSelectedFate);
    }
    let Some(head) = selected_head.head() else {
        if let Some(retired) = retired_terminal_tip(inventory) {
            return Ok(retired);
        }
        if !inventory.publication_selected
            || !inventory.descriptors.is_empty()
            || !inventory.manifests.is_empty()
            || !inventory.reservations.is_empty()
        {
            return Err(BlobReclaimFailure::Deferred(
                BlobReclaimDeferral::CompetingReclaim,
            ));
        }
        return Ok(ValidatedReleaseChain {
            predecessor: None,
            cumulative_dropped: 0,
            terminal: false,
            settled_absence_authority: false,
        });
    };
    if inventory.publication_selected {
        return Err(BlobReclaimFailure::ConflictingSelectedFate);
    }
    validate_current_controls(reader, inventory, head, limits, scratch, work)?;
    Ok(ValidatedReleaseChain {
        predecessor: ReleasedDropPredecessorV1::new(
            head.descriptor_record(),
            head.descriptor_frame_sha256(),
        )
        .ok(),
        cumulative_dropped: head.cumulative_dropped(),
        terminal: head.terminal(),
        settled_absence_authority: selected_head.authorizes_settled_absence(),
    })
}

/// A terminal head retired leaves its terminal descriptor selected while the
/// owner-attested roster for this exact root no longer holds the key. The
/// root that routes a descriptor also installs its head, and only a terminal
/// head retired removes one, so this absence is settled: the release is
/// complete, and a further reclaim proves no effect and mints no head.
fn retired_terminal_tip(inventory: &SelectedReleaseInventory) -> Option<ValidatedReleaseChain> {
    if inventory.publication_selected {
        return None;
    }
    let tip = inventory.descriptors.iter().find(|link| {
        link.descriptor.terminal()
            && link.descriptor.source_kind() == BlobReclaimSourceKind::ReleasedGeneration
            && inventory.fact(link.record).is_some_and(|fact| {
                fact.class == SelectedRecordContentClass::Blob(BlobRecordKind::ReclaimDescriptorV3)
                    && fact.frame_sha256 == link.frame_sha256
            })
    })?;
    Some(ValidatedReleaseChain {
        predecessor: ReleasedDropPredecessorV1::new(tip.record, tip.frame_sha256).ok(),
        cumulative_dropped: tip.descriptor.cumulative_dropped(),
        terminal: true,
        settled_absence_authority: true,
    })
}

#[allow(clippy::too_many_arguments)]
fn validate_current_controls(
    reader: &PhysicalRecordReader,
    inventory: &SelectedReleaseInventory,
    head: ReleaseCustodyHeadEntryV1,
    limits: BlobReclaimLimits,
    scratch: &mut [u8],
    work: &mut scan::ReclaimInspectionWork,
) -> Result<(), BlobReclaimFailure> {
    let denied = || BlobReclaimFailure::ConflictingSelectedFate;
    let link = inventory
        .descriptors
        .binary_search_by_key(&head.descriptor_record(), |value| value.record)
        .ok()
        .and_then(|index| inventory.descriptors.get(index))
        .ok_or_else(denied)?;
    let base = link.descriptor;
    let descriptor_fact = inventory.fact(link.record).ok_or_else(denied)?;
    let basis = inventory.basis;
    let store = basis.publication().store();
    let root_generation = reader.protected_root().root().generation().get();
    if link.frame_sha256 != head.descriptor_frame_sha256()
        || descriptor_fact.class
            != SelectedRecordContentClass::Blob(BlobRecordKind::ReclaimDescriptorV3)
        || head.key().object() != basis.object()
        || head.key().generation() != basis.generation()
        || base.source_kind() != BlobReclaimSourceKind::ReleasedGeneration
        || base.store() != store
        || base.source_basis_digest() != head.source_basis_digest()
        || base.source_basis_digest()
            != BlobReclaimSourceBasisV1::ReleasedGeneration(basis).digest(store)
        || base.manifest_record() != head.manifest_record()
        || base.manifest_frame_sha256() != head.manifest_frame_sha256()
        || base.predecessor() != head.predecessor()
        || base.source_root_generation() != head.source_root_generation()
        || base.cumulative_dropped() != head.cumulative_dropped()
        || base.terminal() != head.terminal()
        || base.candidate_root_generation() > root_generation
    {
        return Err(denied());
    }
    work.records = work
        .records
        .checked_add(1)
        .ok_or(BlobReclaimFailure::ScanBoundExhausted)?;
    let used = scan::read_selected(
        reader,
        link.record,
        descriptor_fact.payload_bytes,
        scratch,
        limits,
        work,
    )?;
    let descriptor_bytes = &scratch[..used];
    if Sha256::digest(descriptor_bytes).as_slice() != link.frame_sha256 {
        return Err(denied());
    }
    let descriptor =
        BlobReclaimDescriptorV3::decode(descriptor_bytes).map_err(BlobReclaimFailure::Format)?;
    if descriptor.base() != base {
        return Err(denied());
    }
    let manifest_link = inventory
        .manifests
        .binary_search_by_key(&head.manifest_record(), |value| value.record)
        .ok()
        .and_then(|index| inventory.manifests.get(index))
        .ok_or_else(denied)?;
    let manifest_fact = inventory.fact(manifest_link.record).ok_or_else(denied)?;
    if manifest_link.frame_sha256 != head.manifest_frame_sha256()
        || manifest_fact.class
            != SelectedRecordContentClass::Blob(BlobRecordKind::DropSetManifestV3)
        || manifest_link.reclaim_attempt != base.reclaim_attempt()
        || manifest_link.count != base.manifest_count()
        || manifest_link.manifest_selected_generation > base.source_root_generation()
    {
        return Err(denied());
    }
    work.records = work
        .records
        .checked_add(1)
        .ok_or(BlobReclaimFailure::ScanBoundExhausted)?;
    let used = scan::read_selected(
        reader,
        manifest_link.record,
        manifest_fact.payload_bytes,
        scratch,
        limits,
        work,
    )?;
    let bytes = &scratch[..used];
    if Sha256::digest(bytes).as_slice() != manifest_link.frame_sha256 {
        return Err(denied());
    }
    let manifest = DropSetManifestV3::decode(bytes).map_err(BlobReclaimFailure::Format)?;
    if manifest.source_basis() != BlobReclaimSourceBasisV1::ReleasedGeneration(basis)
        || manifest.reclaim_attempt() != base.reclaim_attempt()
        || manifest.count() != base.manifest_count()
        || manifest.dropped().contains(&basis.publication_record()) != head.predecessor().is_none()
    {
        return Err(denied());
    }
    let reservation = inventory
        .reservations
        .iter()
        .find(|value| value.record == head.reservation_record())
        .ok_or(BlobReclaimFailure::Deferred(
            BlobReclaimDeferral::CompetingReclaim,
        ))?;
    if reservation.frame_sha256 != head.reservation_frame_sha256()
        || inventory.fact(reservation.record).map(|fact| fact.class)
            != Some(SelectedRecordContentClass::Blob(
                BlobRecordKind::OriginalDropReserved,
            ))
        || reservation.reservation.reclaim_attempt() != base.reclaim_attempt()
        || reservation.reservation.manifest_record() != head.manifest_record()
        || reservation.reservation.manifest_frame_sha256() != head.manifest_frame_sha256()
        || reservation.reservation.source_basis_digest() != head.source_basis_digest()
        || reservation.reservation.manifest_selected_generation()
            != manifest_link.manifest_selected_generation
        || reservation
            .reservation
            .manifest_selected_generation()
            .checked_add(1)
            != Some(reservation.reservation.reserved_selected_generation())
        || reservation.reservation.reserved_selected_generation() != base.source_root_generation()
    {
        return Err(denied());
    }
    let identities = ReleaseCustodyHeadControlIdentityV1::new(
        link.record,
        link.frame_sha256,
        reservation.record,
        reservation.frame_sha256,
        manifest_link.record,
        manifest_link.frame_sha256,
    )
    .map_err(|_| denied())?;
    verify_release_custody_head_controls(
        head,
        identities,
        descriptor,
        reservation.reservation,
        &manifest,
    )
    .map_err(|_| denied())?;
    // The selected keyed head was installed only after a completed C9 effect,
    // and V2 reopen independently rejoins that head to the attested selected
    // source. A process-local completion map need not survive checkpoint prune.
    Ok(())
}
