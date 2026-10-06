//! Carries verified native recovery lineage across one validated, settled commit.
mod outcome;
#[cfg(test)]
mod tests;
pub use outcome::{
    DeferredRecoveredCheckpointTransition, RecoveredCheckpointTransition,
    RecoveredCheckpointTransitionDenial, RecoveredCheckpointTransitionError,
    RecoveredCheckpointTransitionRepairError, RefusedRecoveredCheckpointTransition,
};

use crate::branch::RelationalBranchBasisDescriptor;
use crate::mvcc::{PreparedRelationalCommitCandidate, RelationalPublicationOutcome};

use super::{DurabilityRecoveryAuthority, RecoveredRelationalRuntimeAuthority};

impl DurabilityRecoveryAuthority<'_> {
    /// Publishes a candidate prepared against an exact recovered image and
    /// carries recovery authority forward only after native settlement succeeds.
    ///
    /// This attests native lineage, not application migration meaning. The
    /// application owner must validate its predecessor and target semantics
    /// before it prepares the candidate. Ordinary commits do not refresh recovery
    /// authority. Refusal returns the original authority; durability deferral
    /// retains the exact performed transition for explicit repair.
    ///
    /// ```
    /// use worth_relational::facade::{
    ///     durability::{RecoveredRelationalBranchBasis, RecoveredRelationalRuntimeAuthority},
    ///     mvcc::PreparedRelationalCommitCandidate,
    ///     runtime::RelationalRuntime,
    /// };
    ///
    /// fn carry_recovered_image(
    ///     runtime: &mut RelationalRuntime,
    ///     recovered: RecoveredRelationalRuntimeAuthority,
    ///     validated_candidate: PreparedRelationalCommitCandidate,
    /// ) -> Result<RecoveredRelationalBranchBasis, &'static str> {
    ///     let acknowledged = runtime.durability_recovery()
    ///         .commit_checkpoint_transition(recovered, validated_candidate)
    ///         .map_err(|_| "transition needs refusal or settlement handling")?;
    ///     let (_receipt, successor_authority) = acknowledged.into_parts();
    ///     let (_, basis) = runtime.observe_branch(&runtime.main_branch_identity())
    ///         .map_err(|_| "branch unavailable")?;
    ///     successor_authority.admit_basis(basis).map_err(|_| "branch moved")
    /// }
    /// ```
    pub fn commit_checkpoint_transition(
        &mut self,
        recovered: RecoveredRelationalRuntimeAuthority,
        candidate: PreparedRelationalCommitCandidate,
    ) -> Result<RecoveredCheckpointTransition, RecoveredCheckpointTransitionError> {
        if let Err(denial) = self.admit_checkpoint_transition(&recovered, &candidate) {
            return Err(refused(recovered, denial));
        }
        self.runtime
            .history
            .record_publication_attempt(candidate.branch());
        let performed = match self
            .runtime
            .publication_port()
            .compare_and_publish(candidate)
        {
            RelationalPublicationOutcome::Performed(performed) => performed,
            outcome => {
                return Err(refused(
                    recovered,
                    RecoveredCheckpointTransitionDenial::Publication(Box::new(outcome)),
                ))
            }
        };
        // This descriptor comes from the native performed witness, never an
        // ambient post-commit observation which could select a later root.
        let successor = performed.next_basis().descriptor().clone();
        match self.runtime.settle_performed_publication(performed) {
            Ok(committed) => {
                self.runtime
                    .snapshots()
                    .release_snapshot(&committed.snapshot)
                    .expect(
                        "checkpoint transition releases its settled native snapshot exactly once",
                    );
                Ok(acknowledged(recovered, successor, committed.commit.clone()))
            }
            Err(cause) if cause.deferred_settlement().is_some() => {
                Err(RecoveredCheckpointTransitionError::DurabilityDeferred(
                    Box::new(DeferredRecoveredCheckpointTransition {
                        recovered,
                        successor,
                        cause,
                    }),
                ))
            }
            Err(cause) => Err(RecoveredCheckpointTransitionError::SettlementFailed(
                Box::new(cause),
            )),
        }
    }

    /// Repairs the exact already-performed transition without republishing it
    /// or reconstructing its successor from current runtime state.
    pub fn repair_checkpoint_transition(
        &mut self,
        transition: DeferredRecoveredCheckpointTransition,
    ) -> Result<RecoveredCheckpointTransition, RecoveredCheckpointTransitionRepairError> {
        let settlement = transition
            .cause
            .deferred_settlement()
            .expect("only a native durability deferral constructs transition repair custody");
        match self
            .runtime
            .repair_deferred_publication_settlement(settlement)
        {
            Ok(receipt) => Ok(acknowledged(
                transition.recovered,
                transition.successor,
                receipt,
            )),
            Err(cause) => Err(RecoveredCheckpointTransitionRepairError {
                transition: Box::new(transition),
                cause,
            }),
        }
    }

    fn admit_checkpoint_transition(
        &self,
        recovered: &RecoveredRelationalRuntimeAuthority,
        candidate: &PreparedRelationalCommitCandidate,
    ) -> Result<(), RecoveredCheckpointTransitionDenial> {
        let expected = self.runtime.runtime_instance_id();
        if recovered.runtime_instance_id != expected {
            return Err(
                RecoveredCheckpointTransitionDenial::ForeignRecoveryRuntime {
                    expected,
                    actual: recovered.runtime_instance_id,
                },
            );
        }
        if candidate.runtime_instance_id() != expected {
            return Err(
                RecoveredCheckpointTransitionDenial::ForeignCandidateRuntime {
                    expected,
                    actual: candidate.runtime_instance_id(),
                },
            );
        }
        let basis = candidate
            .expected_basis()
            .ok_or(RecoveredCheckpointTransitionDenial::CandidateUnavailable)?;
        if !recovered.owns_image(&basis) {
            return Err(RecoveredCheckpointTransitionDenial::SourceNotRecovered(
                Box::new(basis),
            ));
        }
        Ok(())
    }
}

fn refused(
    recovered: RecoveredRelationalRuntimeAuthority,
    denial: RecoveredCheckpointTransitionDenial,
) -> RecoveredCheckpointTransitionError {
    RecoveredCheckpointTransitionError::Refused(Box::new(RefusedRecoveredCheckpointTransition {
        recovered,
        denial,
    }))
}

fn acknowledged(
    mut recovered: RecoveredRelationalRuntimeAuthority,
    successor: RelationalBranchBasisDescriptor,
    commit: crate::history::data::RelationalCommitReceipt,
) -> RecoveredCheckpointTransition {
    recovered.recovered_branch_images.insert(
        successor.branch_id().clone(),
        (successor.reference().clone(), successor.truth_version()),
    );
    RecoveredCheckpointTransition { recovered, commit }
}
