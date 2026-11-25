use super::{RelationalCompleteCarriedIndexBasis, RelationalCompleteIndexSelectionInspection};
use crate::durability::data::{
    PreparedCheckpointDirectory, RelationalNativeCheckpoint, RelationalRecoveryStop,
};
use crate::durability::derived_index_artifacts::{
    captured_decode, DerivedIndexCheckpointArtifacts,
};
use crate::durability::log::native_checkpoint_codec::member_frame::{self, MemberKind};
use crate::durability::medium::RelationalDurableColdSource;
use crate::indexes::data::DerivedIndexId;
use crate::storage::data::RelationalStorageSubject;
use std::sync::Arc;

pub(super) fn inspect_carried_index_selections(
    checkpoint: &RelationalNativeCheckpoint,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    source: Arc<dyn RelationalDurableColdSource>,
) -> Result<Option<RelationalCompleteIndexSelectionInspection>, RelationalRecoveryStop> {
    let (manifest, history_members) =
        crate::durability::log::native_checkpoint_codec::decode_manifest_directory(
            execution,
            checkpoint.bytes(),
        )?;
    let directory = PreparedCheckpointDirectory {
        manifest,
        source,
        history_members,
    }
    .readmit(execution)?;
    let key = directory.manifest.derived_indexes;
    let subject = RelationalStorageSubject::Index(DerivedIndexId(0));
    let bytes = directory
        .canonical_read
        .read_member(&directory.source, execution, key, key.length.min(1 << 30))
        .map_err(|cause| {
            crate::durability::checkpoint_history::capture::owned_stop(cause, subject)
        })?;
    let payload = member_frame::decode_for_request(
        execution,
        bytes.bytes(),
        key,
        MemberKind::DerivedIndex,
        subject,
    )?;
    let Some(inspected) = captured_decode::inspect_carried_bases(payload, execution)? else {
        return Ok(None);
    };
    let all_bases_beside_roots = inspected.bases.iter().all(|basis| {
        directory
            .manifest
            .roots
            .iter()
            .any(|root| root.commit_id == basis.commit)
    });
    let bases = inspected
        .bases
        .into_iter()
        .map(|basis| RelationalCompleteCarriedIndexBasis {
            branch: basis.branch,
            commit: basis.commit,
            version: basis.version,
            bindings: basis.bindings,
        })
        .collect();
    Ok(Some(RelationalCompleteIndexSelectionInspection {
        format_version: inspected.format_version,
        expected_format_version: DerivedIndexCheckpointArtifacts::FORMAT_VERSION,
        legacy_artifact_count: inspected.legacy_artifact_count,
        all_bases_beside_roots,
        bases,
    }))
}
