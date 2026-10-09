//! Exact WAL target and selected V3 controls mint a narrow historical-skip
//! witness. The result is still subject to C8's full release completion gate.

use sha2::{Digest, Sha256};
use worth_store_physical_format::{
    decode_blob_record, BlobReclaimSourceBasisV1, BlobRecordKind, BlobRecordV1,
    PersistedPhysicalRecoveryOperation, PersistedRecordIdentity, RecordFrameCoordinate,
};

use super::*;
use crate::{
    HistoricalReleasedDropTargetWitness, PhysicalSourceSelection,
    VerifiedAddressedReleasedControlFrame, VerifiedOrderedRootEdge, VerifiedOrderedRootHistory,
    VerifiedReleasedRootEdge,
};

struct ControlFrames<'a> {
    descriptor_bytes: &'a [u8],
    descriptor_record: PersistedRecordIdentity,
    manifest_bytes: &'a [u8],
    manifest_record: PersistedRecordIdentity,
}

impl AdmittedPhysicalRedoMembers {
    pub fn admit_historical_released_drop_target_with_ordered_history(
        &self,
        selection: &PhysicalSourceSelection,
        target: &PhysicalRedoTarget,
        descriptor_operation: [u8; 32],
        descriptor_frame: &VerifiedAddressedReleasedControlFrame,
        manifest_frame: &VerifiedAddressedReleasedControlFrame,
        history: &VerifiedOrderedRootHistory,
    ) -> Option<HistoricalReleasedDropTargetWitness> {
        let mut matches = history.edges().iter().filter_map(|edge| match edge {
            VerifiedOrderedRootEdge::Released(edge) if edge.operation() == descriptor_operation => {
                Some(edge)
            }
            _ => None,
        });
        let edge = matches.next()?;
        if matches.next().is_some()
            || descriptor_frame.kind() != BlobRecordKind::ReclaimDescriptorV3
            || manifest_frame.kind() != BlobRecordKind::DropSetManifestV3
            || descriptor_frame.candidate_root_frame_sha256() != edge.result_root_frame_sha256()
            || manifest_frame.candidate_root_frame_sha256() != edge.result_root_frame_sha256()
            || descriptor_frame.record() != edge.descriptor_record()
            || descriptor_frame.payload_sha256() != edge.descriptor_frame_sha256()
        {
            return None;
        }
        let controls = ControlFrames {
            descriptor_bytes: descriptor_frame.bytes(),
            descriptor_record: descriptor_frame.record(),
            manifest_bytes: manifest_frame.bytes(),
            manifest_record: manifest_frame.record(),
        };
        self.admit_historical_released_drop_target_inner(
            selection,
            target,
            descriptor_operation,
            controls,
            edge,
            history,
        )
    }

    fn admit_historical_released_drop_target_inner(
        &self,
        selection: &PhysicalSourceSelection,
        target: &PhysicalRedoTarget,
        descriptor_operation: [u8; 32],
        controls: ControlFrames<'_>,
        edge: &VerifiedReleasedRootEdge,
        history: &VerifiedOrderedRootHistory,
    ) -> Option<HistoricalReleasedDropTargetWitness> {
        let old_operation = unique_indeterminate_target_operation(&self.members, target)?;
        let mut descriptors = self
            .members
            .iter()
            .filter(|member| member.operation == descriptor_operation);
        let descriptor_member = descriptors.next()?;
        let PersistedPhysicalRecoveryOperation::RecordsDropped {
            binding,
            directory_replacement,
            ..
        } = descriptor_member.projection.operation()
        else {
            return None;
        };
        // The descriptor record, plus exactly one replacement directory frame
        // when this drop rebinds a derived directory.
        let expected_records = 1 + usize::from(directory_replacement.is_some());
        let selected_root_identity = history_anchors_selection(history, selection)?;
        if descriptors.next().is_some()
            || descriptor_member.fate == RecoveryOperationFate::ProvenNoEffect
            || descriptor_member.records.len() != expected_records
            || edge.descriptor_record() != binding.record()
            || edge.descriptor_frame_sha256() != binding.record_payload_sha256()
            || edge.candidate_root_generation() != binding.candidate_root_generation()
            || descriptor_member
                .projection
                .source_root_generation()
                .checked_add(1)
                != Some(binding.candidate_root_generation())
            || descriptor_member.records[0].bytes() != controls.descriptor_bytes
            || <[u8; 32]>::from(Sha256::digest(controls.descriptor_bytes))
                != binding.record_payload_sha256()
        {
            return None;
        }
        let BlobRecordV1::ReclaimDescriptorV3(descriptor) =
            decode_blob_record(controls.descriptor_bytes).ok()?
        else {
            return None;
        };
        let base = descriptor.base();
        if base.source_root_generation() != descriptor_member.projection.source_root_generation()
            || base.candidate_root_generation() != binding.candidate_root_generation()
            || descriptor.custody().request().idempotency() != descriptor_operation
            || controls.descriptor_record != binding.record()
        {
            return None;
        }
        let manifest_bytes = controls.manifest_bytes;
        let BlobRecordV1::DropSetManifestV3(manifest) = decode_blob_record(manifest_bytes).ok()?
        else {
            return None;
        };
        if controls.manifest_record != base.manifest_record()
            || <[u8; 32]>::from(Sha256::digest(manifest_bytes)) != base.manifest_frame_sha256()
            || manifest.store() != base.store()
            || manifest.reclaim_attempt() != base.reclaim_attempt()
            || manifest.source_basis_digest() != base.source_basis_digest()
            || manifest.count() != base.manifest_count()
            || !matches!(
                manifest.source_basis(),
                BlobReclaimSourceBasisV1::ReleasedGeneration(_)
            )
        {
            return None;
        }
        let record = target_record(target)?;
        if manifest.dropped().binary_search(&record).is_err()
            || selection
                .page_facts()
                .placements()
                .iter()
                .any(|route| route.record() == record)
        {
            return None;
        }
        Some(HistoricalReleasedDropTargetWitness {
            selected_root_identity,
            descriptor_operation,
            old_operation,
            target: target.identity(),
            wal_target_digest: target.resulting_digest(),
            target_coordinate: RecordFrameCoordinate::new(
                target.artifact(),
                target.artifact_offset(),
                target.artifact_length(),
            )?,
        })
    }
}

/// The ordered walk must start at the selected checkpoint and end at the
/// selected root; returns that selected root's frame identity.
pub(super) fn history_anchors_selection(
    history: &VerifiedOrderedRootHistory,
    selection: &PhysicalSourceSelection,
) -> Option<[u8; 32]> {
    let selected = selection.root().selected();
    let selected_root_identity: [u8; 32] =
        Sha256::digest(selected.manifest().encode(selected.selector().format())).into();
    let checkpoint = selection
        .checkpoint()
        .map(|checkpoint| checkpoint.source_root_frame_sha256())
        .unwrap_or([0; 32]);
    anchored_root_identity(
        RootAnchors {
            checkpoint_root_frame_sha256: history.checkpoint_root_frame_sha256(),
            selected_root_frame_sha256: history.selected_root_frame_sha256(),
        },
        RootAnchors {
            checkpoint_root_frame_sha256: checkpoint,
            selected_root_frame_sha256: selected_root_identity,
        },
    )
}

/// The checkpoint a root walk starts from and the root it ends at.
#[derive(Debug, Clone, Copy)]
pub(super) struct RootAnchors {
    pub(super) checkpoint_root_frame_sha256: [u8; 32],
    pub(super) selected_root_frame_sha256: [u8; 32],
}

/// The selected root identity, only when the history's walk starts at the
/// selection's checkpoint and ends at its selected root.
pub(super) fn anchored_root_identity(
    history: RootAnchors,
    selection: RootAnchors,
) -> Option<[u8; 32]> {
    (history.checkpoint_root_frame_sha256 == selection.checkpoint_root_frame_sha256
        && history.selected_root_frame_sha256 == selection.selected_root_frame_sha256)
        .then_some(selection.selected_root_frame_sha256)
}

pub(super) fn unique_indeterminate_target_operation(
    members: &[AdmittedPhysicalRedoMember],
    target: &PhysicalRedoTarget,
) -> Option<[u8; 32]> {
    let mut matching = members.iter().filter(|member| {
        member.fate == RecoveryOperationFate::Indeterminate
            && member
                .records
                .iter()
                .any(|record| record.targets().contains(target))
    });
    let first = matching.next()?;
    matching.next().is_none().then_some(first.operation)
}

pub(super) fn target_record(target: &PhysicalRedoTarget) -> Option<PersistedRecordIdentity> {
    let PhysicalRedoTargetIdentity::ExtentChunk { .. } = target.identity() else {
        return None;
    };
    let coordinate = target.extent_coordinate()?;
    PersistedRecordIdentity::new(coordinate.allocation_epoch(), coordinate.record_ordinal())
}
