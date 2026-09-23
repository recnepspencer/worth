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
    append_segment_entry, checkpoint_file_path, current_segment_ids, ensure_loaded_store,
    persist_store_manifest, segment_file_path, segment_requires_recovery_readmission,
    DurableCheckpointFile,
};
use crate::durability::log::native_file_codec::write_checkpoint_file;

impl<'runtime> DurabilityAuthority<'runtime> {
    pub fn native_checkpoint(
        &self,
    ) -> Result<crate::durability::data::RelationalNativeCheckpoint, DurabilityError> {
        let checkpoint = self.capture_checkpoint_basis()?.build_checkpoint_image()?;
        let bytes = crate::durability::log::native_file_codec::encode_checkpoint(checkpoint)?;
        Ok(crate::durability::data::RelationalNativeCheckpoint::from_captured_bytes(bytes))
    }

    pub fn checkpoint(&self) -> Result<DurableCheckpoint, DurabilityError> {
        let captured = self.capture_checkpoint_basis()?;
        self.finalize_captured_checkpoint(captured)
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
        captured: super::checkpoint_capture::CapturedCheckpointBasis,
    ) -> Result<DurableCheckpoint, DurabilityError> {
        let checkpoint = captured.build_checkpoint_image()?;
        if self.runtime.runtime_config().durability.policy.mode
            == DurabilityMode::PersistedSegmentedLocalFs
        {
            let manifest = self.persist_checkpoint_file(&checkpoint)?;
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

    pub(crate) fn append_commit(
        &self,
        authority: super::DurableAppendAuthority,
        positioned: &crate::history::data::PositionedCanonicalCommit,
    ) -> Result<(), DurabilityError> {
        let envelope = positioned.envelope();
        authority.validate(self.runtime.runtime_instance_id(), envelope)?;
        #[cfg(any(test, feature = "test-durability-faults"))]
        if self.runtime.durability.take_armed_append_failure() {
            return Err(DurabilityError::new(
                crate::durability::data::RecoveryFailureClass::DurableIoFailure,
                "test-injected durable append failure",
            ));
        }
        match self.runtime.runtime_config().durability.policy.mode {
            DurabilityMode::InMemoryCanonical => {
                self.runtime
                    .durability
                    .push_log_envelope(positioned.clone());
                Ok(())
            }
            DurabilityMode::PersistedSegmentedLocalFs => {
                let mut store = ensure_loaded_store(self.runtime)?;
                let segment_capacity = store.layout.segment_commit_capacity.max(1);
                let active_segment = store.segments.last().cloned();
                let active_requires_recovery = match active_segment.as_ref() {
                    Some(segment) if segment.commit_count < segment_capacity => {
                        segment_requires_recovery_readmission(&segment.path)?
                    }
                    _ => false,
                };
                let segment_id = match active_segment.as_ref() {
                    Some(segment)
                        if segment.commit_count < segment_capacity && !active_requires_recovery =>
                    {
                        segment.segment_id
                    }
                    _ => DurableSegmentId(
                        store
                            .segments
                            .last()
                            .map(|segment| segment.segment_id.0)
                            .unwrap_or(0)
                            + 1,
                    ),
                };
                let segment_path = segment_file_path(&store.layout, segment_id);
                let existing_manifest = store
                    .segments
                    .iter()
                    .find(|segment| segment.segment_id == segment_id)
                    .cloned();
                let first_commit_id = existing_manifest
                    .as_ref()
                    .and_then(|segment| segment.first_commit_id)
                    .unwrap_or(envelope.commit.commit_id);
                let last_commit_id = envelope.commit.commit_id;
                let commit_count = existing_manifest
                    .as_ref()
                    .map(|segment| segment.commit_count + 1)
                    .unwrap_or(1);
                append_segment_entry(
                    &segment_path,
                    positioned,
                    &self.runtime.runtime_config().schema.registry,
                )?;
                if let Some(existing) = store
                    .segments
                    .iter_mut()
                    .find(|segment| segment.segment_id == segment_id)
                {
                    existing.first_commit_id = Some(first_commit_id);
                    existing.last_commit_id = Some(last_commit_id);
                    existing.commit_count = commit_count;
                    existing.integrity = DurableIntegrityStatus::Verified;
                } else {
                    store.segments.push(DurableSegmentManifest {
                        segment_id,
                        path: segment_path,
                        first_commit_id: Some(first_commit_id),
                        last_commit_id: Some(last_commit_id),
                        commit_count,
                        runtime_name: self.runtime.runtime_name().to_string(),
                        profile: self.runtime.runtime_profile(),
                        schema_version: self.runtime.primary_schema_version_id(),
                        integrity: DurableIntegrityStatus::Verified,
                    });
                }
                self.runtime.durability.set_store(Some(store));
                self.runtime
                    .durability
                    .push_log_envelope(positioned.clone());
                let latest_commit_id = self
                    .runtime
                    .durability
                    .log()
                    .last()
                    .map(|entry| entry.envelope().commit.commit_id)
                    .unwrap_or(envelope.commit.commit_id);
                self.runtime
                    .publication_authority()
                    .push_bounded_diagnostic(
                        DiagnosticsScope::History,
                        DiagnosticsArtifactKind::MinimalSummary,
                        vec![durable_segment_append_succeeded(
                            segment_id,
                            latest_commit_id,
                        )],
                    );
                Ok(())
            }
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
        checkpoint: &DurableCheckpoint,
    ) -> Result<DurableCheckpointManifest, DurabilityError> {
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
        write_checkpoint_file(
            &path,
            &DurableCheckpointFile {
                checkpoint: checkpoint.clone(),
            },
        )?;
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
