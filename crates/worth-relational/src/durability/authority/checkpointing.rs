mod append;

use super::diagnostics::checkpoint_created;
use super::DurabilityAuthority;
use crate::capabilities::RuntimeConfigSource;
use crate::diagnostics::data::{DiagnosticsArtifactKind, DiagnosticsScope};
use crate::durability::data::{DurabilityError, DurableCheckpoint};

impl<'runtime> DurabilityAuthority<'runtime> {
    pub fn native_checkpoint(
        &self,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        crate::durability::data::RelationalNativeCheckpoint,
        crate::durability::data::RelationalNativeCheckpointCaptureDenial,
    > {
        self.native_checkpoint_with_policy(
            policy,
            crate::durability::data::RelationalCheckpointArtifactPolicy::Embedded,
        )
    }

    pub fn native_checkpoint_with_policy(
        &self,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
        artifact_policy: crate::durability::data::RelationalCheckpointArtifactPolicy,
    ) -> Result<
        crate::durability::data::RelationalNativeCheckpoint,
        crate::durability::data::RelationalNativeCheckpointCaptureDenial,
    > {
        policy.check_live()?;
        let captured = self.capture_checkpoint_basis()?.build_native_capture(
            policy,
            self.runtime.durability.cold_source(),
            artifact_policy,
        )?;
        let manifest_publication = if artifact_policy
            == crate::durability::data::RelationalCheckpointArtifactPolicy::ExternalMembers
        {
            self.runtime.durability.cold_source().map(|source| {
                self.runtime
                    .durability
                    .checkpoint_selection()
                    .prepare(captured.manifest.clone(), source)
            })
        } else {
            None
        };
        let coverage = captured.manifest.covered_position;
        let transitions = captured.manifest.coverage.up_to_branch_transition;
        let (bytes, sections, members, publications) =
            crate::durability::log::native_checkpoint_codec::encode_captured(
                captured,
                policy,
                artifact_policy,
            )?;
        if self.runtime.durability.medium().is_some() {
            self.runtime
                .history
                .compact_branch_transitions_through(transitions);
        }
        Ok(
            crate::durability::data::RelationalNativeCheckpoint::from_captured_bytes(
                bytes,
                sections,
                coverage,
                transitions,
            )
            .with_captured_members(members)
            .with_region_publications(if manifest_publication.is_some() {
                publications
            } else {
                Vec::new()
            })
            .with_manifest_publication(manifest_publication),
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
    ) -> Result<DurableCheckpoint, crate::durability::data::RelationalRecoveryStop> {
        let checkpoint = captured.build_checkpoint_image()?;
        self.runtime
            .publication_authority()
            .push_bounded_diagnostic(
                DiagnosticsScope::History,
                DiagnosticsArtifactKind::MinimalSummary,
                vec![checkpoint_created(&checkpoint)],
            );
        self.runtime.durability.push_checkpoint(checkpoint.clone());
        // Memory-only history has no external copy, including forks needed
        // by replay chains older than any retained checkpoint.
        if self.runtime.durability.medium().is_some() {
            self.runtime
                .history
                .compact_branch_transitions_through(checkpoint.coverage.up_to_branch_transition);
        }
        Ok(checkpoint)
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
                if let Some(checkpoint) = self.runtime.durability.latest_checkpoint()
                    && let Some(commit) = checkpoint.coverage.up_to_commit.as_ref()
                {
                    self.runtime.durability.retain_log_after(commit.commit_id);
                }
                let log_len = self.runtime.durability.log_len();
                if log_len > policy.max_in_memory_envelopes {
                    self.runtime
                        .durability
                        .trim_log_front(log_len - policy.max_in_memory_envelopes);
                }
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
