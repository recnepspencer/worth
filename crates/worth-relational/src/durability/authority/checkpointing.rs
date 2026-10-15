mod append;
use std::fs;

use super::diagnostics::{
    durable_segment_append_succeeded, durable_store_compacted, in_memory_checkpoint_created,
    persisted_checkpoint_created,
};
use super::DurabilityAuthority;
use crate::capabilities::{
    DurabilityRead, RuntimeConfigSource, RuntimeIdentitySource, SchemaVersionSource,
};
use crate::diagnostics::data::{DiagnosticsArtifactKind, DiagnosticsScope};
use crate::durability::data::{
    CompactionOutcome, CompactionPlan, DurabilityError, DurabilityMode, DurableCheckpoint,
    DurableCheckpointId, DurableCheckpointManifest, DurableIntegrityStatus, DurableSegmentId,
    DurableSegmentManifest,
};
use crate::durability::log::local_store::{
    checkpoint_file_path, current_segment_ids, ensure_loaded_store, persist_store_manifest,
    segment_file_path,
};
use crate::durability::log::native_file_codec::write_checkpoint_file;

impl<'runtime> DurabilityAuthority<'runtime> {
    pub fn native_checkpoint(
        &self,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        crate::durability::data::RelationalNativeCheckpoint,
        crate::durability::data::RelationalNativeCheckpointCaptureDenial,
    > {
        policy.check_live()?;
        let checkpoint = self.capture_checkpoint_basis()?.build_checkpoint_image()?;
        let (bytes, sections) =
            crate::durability::log::native_file_codec::encode_checkpoint(&checkpoint, policy)?;
        Ok(
            crate::durability::data::RelationalNativeCheckpoint::from_captured_bytes(
                bytes, sections,
            ),
        )
    }

    pub fn checkpoint(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<DurableCheckpoint, crate::durability::data::RelationalRecoveryStop> {
        execution.check_live().map_err(|stop| {
            crate::durability::data::RelationalRecoveryStop::Request(stop.into())
        })?;
        let captured = self.capture_checkpoint_basis()?;
        self.finalize_captured_checkpoint(execution, captured)
    }

    pub(super) fn capture_checkpoint_basis(
        &self,
    ) -> Result<super::checkpoint_capture::CapturedCheckpointBasis, DurabilityError> {
        let checkpoint_routes = self.runtime.history.canonical_checkpoint_gate();
        let selection = checkpoint_routes
            .checkpoint_selection()
            .map_err(checkpoint_admission_error)?;
        super::checkpoint_capture::CapturedCheckpointBasis::capture(
            self.runtime,
            &checkpoint_routes,
            selection,
        )
    }

    pub(super) fn finalize_captured_checkpoint(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        captured: super::checkpoint_capture::CapturedCheckpointBasis,
    ) -> Result<DurableCheckpoint, crate::durability::data::RelationalRecoveryStop> {
        let checkpoint = captured.build_checkpoint_image()?;
        if self.runtime.runtime_config().durability.policy.mode
            == DurabilityMode::PersistedSegmentedLocalFs
        {
            let manifest = self.persist_checkpoint_file(execution, &checkpoint)?;
            self.runtime
                .publication_authority()
                .push_bounded_diagnostic(
                    DiagnosticsScope::History,
                    DiagnosticsArtifactKind::MinimalSummary,
                    vec![persisted_checkpoint_created(&manifest, &checkpoint)],
                );
        } else {
            self.runtime
                .publication_authority()
                .push_bounded_diagnostic(
                    DiagnosticsScope::History,
                    DiagnosticsArtifactKind::MinimalSummary,
                    vec![in_memory_checkpoint_created(&checkpoint)],
                );
        }
        self.runtime.durability.push_checkpoint(checkpoint.clone());
        Ok(checkpoint)
    }

    pub fn compact_store(&self) -> Result<CompactionOutcome, DurabilityError> {
        if self.runtime.runtime_config().durability.policy.mode
            != DurabilityMode::PersistedSegmentedLocalFs
        {
            return Ok(CompactionOutcome {
                removed_segments: Vec::new(),
                retained_segments: Vec::new(),
            });
        }
        let _store_io = crate::durability::log::local_store::lock_store(self.runtime)?;
        let Some(checkpoint) = self.runtime.durability.latest_checkpoint() else {
            return Ok(CompactionOutcome {
                removed_segments: Vec::new(),
                retained_segments: current_segment_ids(self.runtime.durable_store().as_deref()),
            });
        };
        let Some(up_to_commit) = checkpoint.coverage.up_to_commit.as_ref() else {
            return Ok(CompactionOutcome {
                removed_segments: Vec::new(),
                retained_segments: current_segment_ids(self.runtime.durable_store().as_deref()),
            });
        };
        let mut store = ensure_loaded_store(self.runtime)?;
        let plan = CompactionPlan {
            checkpoint_id: store
                .checkpoints
                .last()
                .map(|manifest| manifest.checkpoint_id)
                .unwrap_or(DurableCheckpointId(0)),
            removable_segments: store
                .segments
                .iter()
                .filter(|segment| {
                    segment
                        .last_commit_id
                        .map(|commit_id| commit_id <= up_to_commit.commit_id)
                        .unwrap_or(false)
                })
                .map(|segment| segment.segment_id)
                .collect(),
        };
        let mut retained_segments = Vec::new();
        let mut removed_segments = Vec::new();
        store.segments.retain(|segment| {
            if plan.removable_segments.contains(&segment.segment_id) {
                let _ = fs::remove_file(&segment.path);
                removed_segments.push(segment.segment_id);
                false
            } else {
                retained_segments.push(segment.segment_id);
                true
            }
        });
        persist_store_manifest(&store)?;
        self.runtime.durability.set_store(Some(store));
        self.runtime
            .publication_authority()
            .push_bounded_diagnostic(
                DiagnosticsScope::History,
                DiagnosticsArtifactKind::MinimalSummary,
                vec![durable_store_compacted(
                    plan.checkpoint_id,
                    &removed_segments,
                )],
            );
        Ok(CompactionOutcome {
            removed_segments,
            retained_segments,
        })
    }

    pub(crate) fn compact_log_if_needed(&self) {
        use crate::config::data::DurableLogRetentionMode;

        let policy = self.runtime.runtime_config().durability.policy.log.clone();
        if self.runtime.durability.log_len() <= policy.max_in_memory_envelopes {
            return;
        }

        match policy.retention_mode {
            DurableLogRetentionMode::RetainAllInMemory => {}
            DurableLogRetentionMode::CompactAfterCheckpoint => {
                if let Some(checkpoint) = self.runtime.durability.latest_checkpoint() {
                    if let Some(commit) = checkpoint.coverage.up_to_commit.as_ref() {
                        self.runtime.durability.retain_log_after(commit.commit_id);
                    }
                }
                let log_len = self.runtime.durability.log_len();
                if log_len > policy.max_in_memory_envelopes {
                    self.runtime
                        .durability
                        .trim_log_front(log_len - policy.max_in_memory_envelopes);
                }
            }
        }
        if self.runtime.runtime_config().durability.policy.mode
            == DurabilityMode::PersistedSegmentedLocalFs
            && self
                .runtime
                .runtime_config()
                .durability
                .policy
                .checkpoints
                .compact_after_checkpoint
        {
            let _ = self.compact_store();
        }
    }

    #[cfg(test)]
    pub(crate) fn remove_durable_envelope_for_test(
        &self,
        commit_id: crate::history::data::CommitId,
    ) -> bool {
        self.runtime.durability.remove_log_commit(commit_id)
    }

    fn persist_checkpoint_file(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        checkpoint: &DurableCheckpoint,
    ) -> Result<DurableCheckpointManifest, crate::durability::data::RelationalRecoveryStop> {
        let _store_io =
            crate::durability::log::local_store::lock_store_for_recovery(self.runtime, execution)?;
        let mut store = ensure_loaded_store(self.runtime)?;
        let checkpoint_id = DurableCheckpointId(
            store
                .checkpoints
                .last()
                .map(|manifest| manifest.checkpoint_id.0)
                .unwrap_or(0)
                + 1,
        );
        let path = checkpoint_file_path(&store.layout, checkpoint_id);
        write_checkpoint_file(execution, &path, checkpoint)?;
        let manifest = DurableCheckpointManifest {
            checkpoint_id,
            path,
            coverage: checkpoint.coverage.clone(),
            partition_count: checkpoint.partition_images.len(),
            runtime_name: self.runtime.runtime_name().to_string(),
            profile: self.runtime.runtime_profile(),
            schema_version: self.runtime.primary_schema_version_id(),
            integrity: DurableIntegrityStatus::Verified,
        };
        store.checkpoints.push(manifest.clone());
        persist_store_manifest(&store)?;
        self.runtime.durability.set_store(Some(store));
        Ok(manifest)
    }
}

pub(super) fn checkpoint_admission_error(
    denial: crate::runtime::CanonicalCheckpointAdmissionError,
) -> DurabilityError {
    match denial {
        crate::runtime::CanonicalCheckpointAdmissionError::PublicationInFlight => {
            DurabilityError::new(
                crate::durability::data::RecoveryFailureClass::CheckpointPublicationInFlight,
                "checkpoint denied while canonical publication is in flight",
            )
        }
        crate::runtime::CanonicalCheckpointAdmissionError::PerformedPublicationRequiresSettlement(
            commit_id,
        ) => DurabilityError::new(
            crate::durability::data::RecoveryFailureClass::PerformedPublicationRequiresSettlement,
            format!(
                "checkpoint denied until performed commit {} completes owner settlement",
                commit_id.0
            ),
        ),
    }
}

#[cfg(test)]
#[path = "checkpoint_concurrency_tests.rs"]
mod checkpoint_concurrency_tests;
