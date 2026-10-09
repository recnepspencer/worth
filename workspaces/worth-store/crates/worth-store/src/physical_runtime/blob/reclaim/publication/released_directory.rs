//! Pre-effect, selected-source proof for a released publication's directory
//! rebinding. The old frame is read under its protected C.5 root; its family
//! roots and quarantine watermark are copied, while only the watermark naming
//! the released publication is cleared. The root's own latest hints are kept.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    DerivedFamilyRootDirectoryV1, SelectedRecordContentClass, MAX_DERIVED_FAMILY_ROOTS,
};

use crate::physical_runtime::{
    durability::AdmittedReleasedGenerationDrop,
    layout::{PhysicalLayoutPagePort, PhysicalLayoutPageReadFailure},
    record_serving::PreparedReleasedDirectoryRebinding,
    PhysicalRecordId, ServingPhysicalRuntime,
};

use super::super::{contracts::RELEASED_DIRECTORY_REBINDING_BYTES, BlobReclaimFailure};

const MAX_DIRECTORY_FRAME_BYTES: usize = 101 + MAX_DERIVED_FAMILY_ROOTS * 26;
// The reclaim envelope covers the decoded source directory and the new
// canonical frame, which overlap until the Drop member consumes the frame.
const _: () = assert!(MAX_DIRECTORY_FRAME_BYTES * 4 < RELEASED_DIRECTORY_REBINDING_BYTES as usize);

pub(super) struct PlannedReleasedDirectoryRebinding {
    basis: PreparedReleasedDirectoryRebinding,
    encoded: Vec<u8>,
}

impl PlannedReleasedDirectoryRebinding {
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

pub(super) fn prepare_before_first_effect(
    runtime: &ServingPhysicalRuntime,
    admitted: &AdmittedReleasedGenerationDrop,
) -> Result<Option<PlannedReleasedDirectoryRebinding>, BlobReclaimFailure> {
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
    // The read itself is capped at the canonical directory maximum, so the
    // envelope constant bounds the source bytes before they are buffered.
    let old_frame = PhysicalLayoutPagePort::read_node_from_protected_reader(
        runtime,
        reader,
        MAX_DIRECTORY_FRAME_BYTES as u32,
        old_binding.directory_record(),
    )
    .map_err(|failure| match failure {
        PhysicalLayoutPageReadFailure::NodeTooWide => BlobReclaimFailure::ConflictingSelectedFate,
        failure => BlobReclaimFailure::ReleasedDirectoryRead(failure),
    })?;
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
    }))
}
