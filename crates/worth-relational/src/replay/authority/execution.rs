use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::shared::SharedCanonicalCommitEnvelopeOwnerPort as _;
use worth_relational_durability::internal_port::replay::data::verification::RelationalReplayOutcomeOwnerPort as _;
mod completed_comparison;

use crate::durability::data::RelationalRecoveryStop;
use std::ops::ControlFlow;
mod comparison;
mod diagnostic_comparison;
mod envelope_self_audit;
mod surface_audit;

use crate::history::data::{BranchId, CommitId, HistoryDriftClass};
use crate::replay::data::{
    CanonicalCommitEnvelope, RelationalReplayOutcome, RelationalReplayRequest, ReplayExecutionMode,
    ReplayFailureClass, ReplayMismatch, ReplayMismatchClass, ReplayObservableSurface,
    ReplayVerificationLayer, ReplayVerificationMode, ReplayVerificationPlan,
};
use crate::runtime::RelationalRuntime;

use super::super::planning::{
    demand_replay_envelopes, load_replay_envelope, promised_replay_surfaces,
};
use super::commit_closure::replay_commit_closure_by_commit_id_order;
use super::continuity::validated_replay_continuity_envelope;
use super::recovery_plan::replay_recovery_plan_for_chain;
use super::snapshot_surface::snapshot_surface_is_authoritative;
use super::ReplayAuthority;

use self::comparison::{replay_comparison_outcome, select_replay_lineage_authority};
use self::envelope_self_audit::audit_retained_envelope_authority;
use self::surface_audit::compare_replay_surfaces;

struct ReplayAdmission {
    envelope: std::sync::Arc<CanonicalCommitEnvelope>,
    commit_closure: Vec<CommitId>,
    verification_plan: ReplayVerificationPlan,
}

struct ReplayReconstruction {
    runtime: RelationalRuntime,
    envelope: std::sync::Arc<CanonicalCommitEnvelope>,
}

impl<'runtime> ReplayAuthority<'runtime> {
    fn fail_and_record(
        &self,
        request: RelationalReplayRequest,
        envelope: Option<&CanonicalCommitEnvelope>,
        chain: Option<&[CommitId]>,
        failure: ReplayFailureClass,
        mismatch: Option<ReplayMismatch>,
    ) -> RelationalReplayOutcome {
        match mismatch {
            Some(mismatch) => RelationalReplayOutcome::fail(request, envelope, chain, failure)
                .with_mismatch(mismatch),
            None => RelationalReplayOutcome::fail(request, envelope, chain, failure),
        }
    }

    pub fn replay_commit(
        &self,
        request: RelationalReplayRequest,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<RelationalReplayOutcome, RelationalRecoveryStop> {
        crate::execution::check_request(execution).map_err(RelationalRecoveryStop::Request)?;
        let _source_envelopes = demand_replay_envelopes(
            &self.runtime.history,
            execution,
            self.runtime
                .history()
                .inspect_commit_ancestry(request.commit_id)
                .commit_ids(),
        )?;
        let admission = match self.admit_replay_target(&request) {
            Ok(admission) => admission,
            Err(outcome) => return Ok(outcome),
        };
        let reconstruction =
            match self.reconstruct_replay_target(execution, &request, &admission)? {
                ControlFlow::Continue(reconstruction) => reconstruction,
                ControlFlow::Break(outcome) => return Ok(outcome),
            };
        self.compare_replay_reconstruction(request, admission, reconstruction, execution)
    }

    fn admit_replay_target(
        &self,
        request: &RelationalReplayRequest,
    ) -> Result<ReplayAdmission, RelationalReplayOutcome> {
        #[cfg(test)]
        replay_contacts::observe();
        let Some(envelope) =
            load_replay_envelope(self.runtime.history.canonical_read(), request.commit_id)
        else {
            return Err(self.fail_and_record(
                request.clone(),
                None,
                None,
                ReplayFailureClass::MissingCommit,
                None,
            ));
        };
        if envelope.branch_context != request.branch_id {
            return Err(self.fail_and_record(
                request.clone(),
                Some(&envelope),
                None,
                ReplayFailureClass::BranchMismatch,
                None,
            ));
        }
        let commit_closure =
            match replay_commit_closure_by_commit_id_order(self.runtime, request.commit_id) {
                Ok(chain) => chain,
                Err(failure) => {
                    return Err(self.fail_and_record(
                        request.clone(),
                        Some(&envelope),
                        None,
                        failure,
                        None,
                    ));
                }
            };
        let verification_plan = ReplayVerificationPlan::from_mode(request.verification_mode);
        if let Err(mismatch) =
            validated_replay_continuity_envelope(self.runtime, &envelope, &verification_plan)
        {
            return Err(self.record_continuity_rejection(
                request.clone(),
                &envelope,
                &commit_closure,
                mismatch,
            ));
        }
        let compared_surfaces = promised_replay_surfaces(
            snapshot_surface_is_authoritative(self.runtime, &envelope, &commit_closure),
            &envelope,
            None,
        );
        let mut mismatches = Vec::new();
        audit_retained_envelope_authority(self.runtime, &mut mismatches, &envelope, None);
        if !mismatches.is_empty() {
            let outcome = RelationalReplayOutcome {
                requested: request.clone(),
                commit: Some(envelope.commit.clone()),
                reconstructed_commit_closure: commit_closure.clone(),
                snapshot_version: Some(envelope.commit.version_id),
                lineage_authority_basis: None,
                compared_surfaces,
                mismatches,
                failure: Some(ReplayFailureClass::ObservableMismatch),
            };
            return Err(outcome);
        }
        Ok(ReplayAdmission {
            envelope,
            commit_closure,
            verification_plan,
        })
    }

    fn reconstruct_replay_target(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        request: &RelationalReplayRequest,
        admission: &ReplayAdmission,
    ) -> Result<ControlFlow<RelationalReplayOutcome, ReplayReconstruction>, RelationalRecoveryStop>
    {
        let replay_plan = replay_recovery_plan_for_chain(
            execution,
            self.runtime,
            &admission.commit_closure,
            request.verification_mode,
        )?;
        let runtime = match self
            .runtime
            .rebuild_runtime_from_plan(execution, replay_plan)
        {
            Ok(runtime) => runtime,
            Err(RelationalRecoveryStop::Request(cause)) => {
                return Err(RelationalRecoveryStop::Request(cause))
            }
            Err(RelationalRecoveryStop::AdmissionImage(cause)) => {
                return Err(RelationalRecoveryStop::AdmissionImage(cause))
            }
            Err(RelationalRecoveryStop::Content(cause)) => {
                return Err(RelationalRecoveryStop::Content(cause));
            }
            Err(RelationalRecoveryStop::Allocation(error)) => {
                return Err(RelationalRecoveryStop::Allocation(error))
            }
            Err(RelationalRecoveryStop::Durability(error)) => {
                return Ok(ControlFlow::Break(self.fail_and_record(
                    request.clone(),
                    Some(&admission.envelope),
                    Some(&admission.commit_closure),
                    ReplayFailureClass::ObservableMismatch,
                    Some(ReplayMismatch {
                        class: ReplayMismatchClass::HistoryDrift,
                        history_drift_class: Some(HistoryDriftClass::ReplayAuthorityDrift),
                        surface: ReplayObservableSurface::History,
                        verification_layer: ReplayVerificationLayer::DeepArtifactParity,
                        detail: format!("replay recovery reconstruction failed: {}", error.detail),
                        expected: Some("reconstructible canonical commit closure".to_owned()),
                        observed: Some(format!("{:?}", error.class)),
                    }),
                )));
            }
        };
        let Some(envelope) = runtime
            .replay()
            .canonical_commit_envelope(execution, request.commit_id)?
        else {
            return Ok(ControlFlow::Break(self.fail_and_record(
                request.clone(),
                Some(&admission.envelope),
                Some(&admission.commit_closure),
                ReplayFailureClass::ObservableMismatch,
                Some(ReplayMismatch {
                    class: ReplayMismatchClass::HistoryDrift,
                    history_drift_class: Some(HistoryDriftClass::ReplayAuthorityDrift),
                    surface: ReplayObservableSurface::History,
                    verification_layer: ReplayVerificationLayer::DeepArtifactParity,
                    detail: "replayed target envelope was not reconstructed".to_string(),
                    expected: Some(format!("{:?}", admission.envelope.commit.commit_id)),
                    observed: None,
                }),
            )));
        };
        if let Err(mismatch) = validated_replay_continuity_envelope(
            self.runtime,
            &envelope,
            &admission.verification_plan,
        ) {
            return Ok(ControlFlow::Break(self.record_continuity_rejection(
                request.clone(),
                &admission.envelope,
                &admission.commit_closure,
                mismatch,
            )));
        }
        Ok(ControlFlow::Continue(ReplayReconstruction {
            runtime,
            envelope: envelope.into_arc(),
        }))
    }

    pub fn replay_range(
        &self,
        branch_id: BranchId,
        commits: &[CommitId],
        verification_mode: ReplayVerificationMode,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Vec<RelationalReplayOutcome>, RelationalRecoveryStop> {
        commits
            .iter()
            .copied()
            .map(|commit_id| {
                self.replay_commit(
                    RelationalReplayRequest {
                        commit_id,
                        branch_id: branch_id.clone(),
                        execution_mode: ReplayExecutionMode::SerialDeterministic,
                        verification_mode,
                    },
                    execution,
                )
            })
            .collect()
    }

    fn record_continuity_rejection(
        &self,
        request: RelationalReplayRequest,
        envelope: &CanonicalCommitEnvelope,
        commit_closure: &[CommitId],
        mismatch: ReplayMismatch,
    ) -> RelationalReplayOutcome {
        if mismatch.class == ReplayMismatchClass::DescriptorVersionDrift {
            self.runtime
                .performance_access()
                .count_descriptor_version_mismatch();
        }
        self.runtime
            .performance_access()
            .count_replay_verification_layer(mismatch.verification_layer);
        self.fail_and_record(
            request,
            Some(envelope),
            Some(commit_closure),
            ReplayFailureClass::ObservableMismatch,
            Some(mismatch),
        )
    }
}

#[cfg(test)]
mod replay_contacts {
    use std::cell::Cell;
    thread_local! { static CONTACTS: Cell<usize> = const { Cell::new(0) }; }
    pub(super) fn observe() {
        CONTACTS.with(|count| count.set(count.get() + 1));
    }
    pub(super) fn count() -> usize {
        CONTACTS.with(Cell::get)
    }
}

#[cfg(test)]
mod entry_stop_tests;
