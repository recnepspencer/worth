use worth_proof::TransitionOutcome;

use super::{RelationalRuntime, RelationalRuntimeOwnerBinding, RelationalRuntimeTenure};
use crate::history::data::CommitId;
use crate::runtime::CanonicalCheckpointAdmissionError;

/// Why the owner could not establish a quiescent admission hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalRuntimeAdmissionHoldDenial {
    /// An admitted operation or a retrying waiter is in flight.
    AdmissionsActive,
    /// This handle carries an operation admission rather than close authority.
    NotOwner,
    /// The owner has already stopped admission permanently.
    AlreadySealed,
    /// The canonical publication gate is occupied.
    PublicationInFlight,
    /// The performed commit must finish settlement before admission can be held.
    PerformedPublicationRequiresSettlement(CommitId),
    /// A prepared candidate or its detached cleanup still owns reservations.
    PreparedCandidatesOutstanding,
}

/// One exclusive, quiescent owner borrow with exactly two exits: seal or release.
#[derive(Debug)]
#[must_use = "seal the hold or drop it to release waiting admissions"]
pub struct RelationalRuntimeAdmissionHold<'owner> {
    runtime: &'owner mut RelationalRuntime,
    lifecycle: Option<RelationalRuntimeOwnerBinding>,
}

pub type RelationalRuntimeAdmissionHoldOutcome<'owner> =
    TransitionOutcome<RelationalRuntimeAdmissionHold<'owner>, RelationalRuntimeAdmissionHoldDenial>;

impl RelationalRuntime {
    /// Hold admission without waiting for existing work. A refusal releases
    /// any temporary hold before returning and leaves settlement custody intact.
    /// Immediately after release, a woken waiter may be retrying admission; a
    /// new hold correctly refuses `AdmissionsActive` while that retry is in flight.
    pub fn try_hold_admission(&mut self) -> RelationalRuntimeAdmissionHoldOutcome<'_> {
        let lifecycle = match &self.tenure {
            RelationalRuntimeTenure::Owner(close) => match close.try_hold_admission() {
                Ok(lifecycle) => lifecycle,
                Err(denial) => return TransitionOutcome::denied(denial),
            },
            RelationalRuntimeTenure::Sealed(_) => {
                return TransitionOutcome::denied(
                    RelationalRuntimeAdmissionHoldDenial::AlreadySealed,
                )
            }
            RelationalRuntimeTenure::Admitted(_) => {
                return TransitionOutcome::denied(RelationalRuntimeAdmissionHoldDenial::NotOwner)
            }
        };
        let hold = RelationalRuntimeAdmissionHold {
            runtime: self,
            lifecycle: Some(lifecycle),
        };
        #[cfg(test)]
        hold.lifecycle
            .as_ref()
            .unwrap()
            .pause_after_test_hold_start();
        match hold.check_quiescence() {
            Ok(()) => TransitionOutcome::success(hold),
            Err(denial) => TransitionOutcome::denied(denial),
        }
    }
}

impl RelationalRuntimeAdmissionHold<'_> {
    /// Capture the quiescent closing image without admitting an operation.
    pub fn native_checkpoint(
        &self,
    ) -> Result<
        crate::durability::data::RelationalNativeCheckpoint,
        crate::durability::data::DurabilityError,
    > {
        self.runtime.durability_authority().native_checkpoint()
    }

    /// Read registered and retired branch names without admitting an operation.
    pub fn branch_names(&self) -> crate::branch::RelationalBranchNames {
        self.runtime.branch_names()
    }

    /// Freeze the proven quiescent state and resolve publication exactly once.
    pub fn seal(mut self) {
        // The existing read implementations need the whole runtime borrow, so
        // close authority cannot also be borrowed into this guard from its tenure.
        let RelationalRuntimeTenure::Owner(close) = &self.runtime.tenure else {
            unreachable!("only an owner can hold admission");
        };
        let seal = close.seal_held();
        self.lifecycle.take();
        let spent = std::mem::replace(
            &mut self.runtime.tenure,
            RelationalRuntimeTenure::Sealed(seal),
        );
        if let RelationalRuntimeTenure::Owner(close) = spent {
            close.resolve_sealed_publication();
        }
    }

    fn check_quiescence(&self) -> Result<(), RelationalRuntimeAdmissionHoldDenial> {
        self.runtime
            .history
            .canonical_checkpoint_gate()
            .check_quiescent_publication()
            .map_err(|error| match error {
                CanonicalCheckpointAdmissionError::PublicationInFlight => {
                    RelationalRuntimeAdmissionHoldDenial::PublicationInFlight
                }
                CanonicalCheckpointAdmissionError::PerformedPublicationRequiresSettlement(
                    commit,
                ) => RelationalRuntimeAdmissionHoldDenial::PerformedPublicationRequiresSettlement(
                    commit,
                ),
            })?;
        if !self
            .runtime
            .publication_binding()
            .candidates_are_quiescent()
        {
            return Err(RelationalRuntimeAdmissionHoldDenial::PreparedCandidatesOutstanding);
        }
        Ok(())
    }
}

impl Drop for RelationalRuntimeAdmissionHold<'_> {
    fn drop(&mut self) {
        if let Some(lifecycle) = self.lifecycle.take() {
            lifecycle.release_hold();
        }
    }
}
