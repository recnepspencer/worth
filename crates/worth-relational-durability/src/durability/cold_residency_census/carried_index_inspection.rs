//! Inspect carried index selections through the checkpoint's cold member source.
use super::{RelationalCompleteCarriedIndexBasis, RelationalCompleteIndexSelectionInspection};
use crate::durability::data::{PreparedCheckpointDirectory, RelationalNativeCheckpoint};
use crate::durability::derived_index_artifacts::{
    captured_decode, DerivedIndexCheckpointArtifacts,
};
use contracts::durability::data::recovery_stop::RelationalRecoveryStop;
use contracts::indexes::data::definition::DerivedIndexId;
use contracts::storage::data::storage_unavailable::RelationalStorageSubject;
use std::sync::Arc;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_storage::internal_port::durability::medium::cold_source::RelationalDurableColdSource;
use worth_relational_storage::internal_port::durability::medium::owned_read_stop::owned_stop;
use worth_relational_storage::internal_port::storage::data::member_frame::{self, MemberKind};

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
            owned_stop(cause, subject)
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
