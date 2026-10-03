//! Pre-effect, selected-source proof for a released publication's directory
//! rebinding. The old frame is read under its protected C.5 root; its family
//! roots and quarantine watermark are copied, while only the watermark naming
//! the released publication is cleared. The root's own latest hints are kept.

use std::num::NonZeroU64;

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    DerivedFamilyRootDirectoryV1, SelectedRecordContentClass, MAX_DERIVED_FAMILY_ROOTS,
};

use crate::physical_runtime::{
    durability::AdmittedReleasedGenerationDrop, layout::PhysicalLayoutPagePort,
    record_serving::PreparedReleasedDirectoryRebinding, BlobPhysicalAllocation, PhysicalRecordId,
    ServingPhysicalRuntime,
};

use super::super::BlobReclaimFailure;

const MAX_DIRECTORY_FRAME_BYTES: usize = 101 + MAX_DERIVED_FAMILY_ROOTS * 26;
const REBINDING_HEAP_CHARGE_BYTES: u64 = 16 * 1024;
const _: () = assert!(MAX_DIRECTORY_FRAME_BYTES * 4 < REBINDING_HEAP_CHARGE_BYTES as usize);

pub(super) struct PlannedReleasedDirectoryRebinding<'runtime> {
    basis: PreparedReleasedDirectoryRebinding,
    encoded: Vec<u8>,
    _charge: BlobPhysicalAllocation<'runtime>,
}

impl PlannedReleasedDirectoryRebinding<'_> {
    pub(super) const fn basis(&self) -> PreparedReleasedDirectoryRebinding {
        self.basis
    }

    pub(super) fn encoded_bytes(&self) -> u64 {
        self.encoded.len() as u64
    }

    pub(super) fn take_encoded(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.encoded)
    }
}

pub(super) fn prepare_before_first_effect<'runtime>(
    runtime: &'runtime ServingPhysicalRuntime,
    admitted: &AdmittedReleasedGenerationDrop,
) -> Result<Option<PlannedReleasedDirectoryRebinding<'runtime>>, BlobReclaimFailure> {
    let reader = admitted.protected_reader();
    if Some(reader.protected_root().root()) != admitted.attempt().expected_root() {
        return Err(BlobReclaimFailure::FenceLost);
    }
    let Some(old_binding) = reader.selected_derived_family_directory() else {
        return Ok(None);
    };
    let Some(indexed) = old_binding.indexed_through_blob_publication() else {
        return Ok(None);
    };
    if admitted.dropped().binary_search(&indexed.record()).is_err() {
        return Ok(None);
    }
    if admitted
        .dropped()
        .binary_search(&old_binding.directory_record())
        .is_ok()
        || reader
            .selected_content_class(PhysicalRecordId::from_persisted(
                old_binding.directory_record(),
            ))
            .map_err(BlobReclaimFailure::Read)?
            != SelectedRecordContentClass::DerivedDirectory
    {
        return Err(BlobReclaimFailure::ConflictingSelectedFate);
    }
    // This grant overlaps the decoded source directory and new canonical
    // frame, and remains live until the Drop member has consumed that frame.
    let charge = runtime
        .physical_allocations()
        .admit_blob(NonZeroU64::new(REBINDING_HEAP_CHARGE_BYTES).expect("positive charge"))
        .map_err(BlobReclaimFailure::Allocation)?;
    let old_frame = PhysicalLayoutPagePort::read_node_from_protected_reader(
        runtime,
        reader,
        runtime.maximum_inline_record_bytes(),
        old_binding.directory_record(),
    )
    .map_err(BlobReclaimFailure::ReleasedDirectoryRead)?;
    if old_frame.bytes().len() > MAX_DIRECTORY_FRAME_BYTES {
        return Err(BlobReclaimFailure::ConflictingSelectedFate);
    }
    let old = DerivedFamilyRootDirectoryV1::decode(old_frame.bytes())
        .map_err(|_| BlobReclaimFailure::ConflictingSelectedFate)?;
    if old.indexed_through_blob_publication() != Some(indexed) {
        return Err(BlobReclaimFailure::ConflictingSelectedFate);
    }
    let old_sha: [u8; 32] = Sha256::digest(old_frame.bytes()).into();
    let next = DerivedFamilyRootDirectoryV1::new(old.entries().to_vec())
        .map_err(|_| BlobReclaimFailure::ConflictingSelectedFate)?
        .with_indexed_through_quarantine(old.indexed_through_quarantine());
    let encoded = next.encode();
    if encoded.len() > MAX_DIRECTORY_FRAME_BYTES {
        return Err(BlobReclaimFailure::ConflictingSelectedFate);
    }
    let next_sha: [u8; 32] = Sha256::digest(&encoded).into();
    Ok(Some(PlannedReleasedDirectoryRebinding {
        basis: PreparedReleasedDirectoryRebinding::from_selected_source(
            old_binding,
            old_sha,
            next_sha,
            old.indexed_through_quarantine(),
        ),
        encoded,
        _charge: charge,
    }))
}

pub(super) fn revalidate_before_drop(
    runtime: &ServingPhysicalRuntime,
    admitted: &AdmittedReleasedGenerationDrop,
    basis: PreparedReleasedDirectoryRebinding,
) -> Result<(), BlobReclaimFailure> {
    let reader = admitted.protected_reader();
    if admitted.attempt().expected_root().is_none()
        || reader.selected_derived_family_directory() != Some(basis.expected_previous())
    {
        return Err(BlobReclaimFailure::FenceLost);
    }
    let frame = PhysicalLayoutPagePort::read_node_from_protected_reader(
        runtime,
        reader,
        runtime.maximum_inline_record_bytes(),
        basis.expected_previous().directory_record(),
    )
    .map_err(BlobReclaimFailure::ReleasedDirectoryRead)?;
    if <[u8; 32]>::from(Sha256::digest(frame.bytes())) != basis.expected_previous_payload_sha256() {
        return Err(BlobReclaimFailure::ConflictingSelectedFate);
    }
    Ok(())
}
