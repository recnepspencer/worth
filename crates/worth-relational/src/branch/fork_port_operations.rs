#[path = "fork_port_operations/source_observation.rs"]
mod source_observation;
use super::{RelationalForkDenial, RelationalForkOutcome};
use crate::branch::authority::admit_relational_fork_source;
use crate::branch::fork_source_basis::{
    AdmittedRelationalForkSourceBasis, RelationalForkSourceDescriptor,
};
use crate::branch::{
    AdmittedRelationalBranchBasis, LiveBranchTransitionStep, RelationalBranchLifecyclePosture,
    RelationalBranchTransitionDurableStep, RelationalBranchTransitionStop, RelationalForkPort,
    RelationalForkTargetReservation,
};
use crate::history::data::BranchId;
use crate::runtime::BranchTransitionRecordError;

type ForkStop = RelationalBranchTransitionStop<RelationalForkDenial>;
use super::publication::with_basis;
use super::source_validation::ForkSourceInput;

impl RelationalForkPort {
    /// Consume a retained source into a new independently writable branch.
    /// This process-local profile refuses attached durability before reserving
    /// the destination. It grants no model-navigation or application permission.
    pub fn fork_retained_branch(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        target: BranchId,
        source: crate::branch::AdmittedRelationalRetainedForkSource,
    ) -> Result<(RelationalForkOutcome, AdmittedRelationalBranchBasis), ForkStop> {
        self.validate_retained_fork_profile()
            .map_err(RelationalBranchTransitionStop::Denied)?;
        let reservation = self
            .reserve_fork_target(target)
            .map_err(RelationalBranchTransitionStop::Denied)?;
        self.fork_reserved_retained_with_basis(execution, reservation, source)
    }

    pub fn fork_reserved_retained_with_basis(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        reservation: RelationalForkTargetReservation,
        source: crate::branch::AdmittedRelationalRetainedForkSource,
    ) -> Result<(RelationalForkOutcome, AdmittedRelationalBranchBasis), ForkStop> {
        self.validate_retained_fork_profile()
            .map_err(RelationalBranchTransitionStop::Denied)?;
        let step = LiveBranchTransitionStep::new(self.medium.clone(), execution);
        self.fork_reserved_input_with_hooks(
            execution,
            &step,
            reservation,
            ForkSourceInput::Retained(source),
            || {},
            || {},
            true,
        )
        .map(with_basis)
        .map_err(Into::into)
    }

    /// Consume a fork-only source token and create a fresh branch reference,
    /// durably before it is live. The immutable source artifact is shared by
    /// identity; no envelope copy or source-cell mutation is performed.
    pub fn fork_branch(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        target_branch: BranchId,
        source: AdmittedRelationalForkSourceBasis,
    ) -> Result<RelationalForkOutcome, ForkStop> {
        let reservation = self
            .reserve_fork_target(target_branch)
            .map_err(RelationalBranchTransitionStop::Denied)?;
        self.fork_reserved(execution, reservation, source)
    }

    /// Reserve one exact destination without installing or moving a branch.
    /// The returned custody is consumed by [`Self::fork_reserved`] or releases
    /// the reservation exactly once when dropped.
    pub fn reserve_fork_target(
        &self,
        target_branch: BranchId,
    ) -> Result<RelationalForkTargetReservation, RelationalForkDenial> {
        self.admit_home_program_fork()?;
        let _operation = self
            .lifecycle
            .admit()
            .ok_or(RelationalForkDenial::OwnerUnavailable)?;
        self.owner
            .reserve_target(target_branch)
            .map_err(|denial| match denial {
                crate::branch::RelationalForkTargetReservationDenial::Duplicate => {
                    RelationalForkDenial::DuplicateTarget
                }
                crate::branch::RelationalForkTargetReservationDenial::Retired => {
                    RelationalForkDenial::RetiredTarget
                }
            })
    }

    /// Consume owner-issued destination custody and source evidence through
    /// the canonical fork authority path.
    pub fn fork_reserved(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        reservation: RelationalForkTargetReservation,
        source: AdmittedRelationalForkSourceBasis,
    ) -> Result<RelationalForkOutcome, ForkStop> {
        self.admit_home_program_fork()
            .map_err(RelationalBranchTransitionStop::Denied)?;
        let step = LiveBranchTransitionStep::new(self.medium.clone(), execution);
        self.fork_reserved_with_hooks(execution, &step, reservation, source, || {}, || {}, false)
            .map(|installed| installed.outcome)
            .map_err(Into::into)
    }

    /// Consume owner-issued fork custody and return the authentic admitted
    /// basis of the installed destination in the same owner operation.
    pub fn fork_reserved_with_basis(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        reservation: RelationalForkTargetReservation,
        source: AdmittedRelationalForkSourceBasis,
    ) -> Result<(RelationalForkOutcome, AdmittedRelationalBranchBasis), ForkStop> {
        self.admit_home_program_fork()
            .map_err(RelationalBranchTransitionStop::Denied)?;
        let step = LiveBranchTransitionStep::new(self.medium.clone(), execution);
        self.fork_reserved_with_hooks(execution, &step, reservation, source, || {}, || {}, true)
            .map(with_basis)
            .map_err(Into::into)
    }

    /// Bind the actual World creation association before the original durable fork.
    /// A memory-only receiver refuses before any target installation.
    pub fn fork_reserved_associated_with_basis(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        reservation: RelationalForkTargetReservation,
        source: AdmittedRelationalForkSourceBasis,
        association: crate::history::data::RelationalCallerAssociation,
    ) -> Result<super::RelationalAssociatedFork, ForkStop> {
        self.admit_home_program_fork()
            .map_err(RelationalBranchTransitionStop::Denied)?;
        let medium = self
            .medium
            .clone()
            .ok_or(RelationalBranchTransitionStop::Denied(
                RelationalForkDenial::DurableAssociationRequiresMedium,
            ))?;
        let bytes = crate::history::data::RelationalCallerAssociation::retained_allocation_bytes(
            association.as_bytes().len(),
        )
        .ok_or(RelationalBranchTransitionStop::Denied(
            RelationalForkDenial::RetentionCapacityExhausted,
        ))?;
        let memory = execution
            .reserve_retained_memory(bytes)
            .map_err(RelationalBranchTransitionStop::RequestStopped)?;
        let association = association.with_decoded_memory(memory);
        let step = LiveBranchTransitionStep::associated(medium, execution, association);
        self.fork_reserved_with_hooks(execution, &step, reservation, source, || {}, || {}, true)
            .map(super::RelationalAssociatedFork::from_installation)
            .map_err(Into::into)
    }

    fn admit_home_program_fork(&self) -> Result<(), RelationalForkDenial> {
        if let Some(medium) = &self.medium {
            medium.admit_home_program_write(None, None).map_err(|cause|
                RelationalForkDenial::DurablePublicationRefused(
                    crate::durability::medium::RelationalDurablePublicationRefusal::HomeProgramTransition(cause)))?;
        }
        Ok(())
    }

    /// Replay a recorded fork of `target_branch` from `source_branch` through
    /// `step`: the same observation, reservation and fork a live caller
    /// performs, from the recovering runtime's own registry.
    pub(crate) fn fork_branch_with<S: RelationalBranchTransitionDurableStep>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        step: &S,
        target_branch: BranchId,
        source_branch: &BranchId,
    ) -> Result<RelationalForkOutcome, BranchTransitionRecordError<RelationalForkDenial, S::Stop>>
    {
        let denied = BranchTransitionRecordError::Denied;
        let (_, source) = self.observe_fork_source(source_branch).map_err(denied)?;
        let reservation = self.reserve_fork_target(target_branch).map_err(denied)?;
        self.fork_reserved_with_hooks(execution, step, reservation, source, || {}, || {}, false)
            .map(|installed| installed.outcome)
    }

    #[cfg(test)]
    pub(crate) fn fork_branch_with_test_pause(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        target_branch: BranchId,
        source: AdmittedRelationalForkSourceBasis,
        reached: &std::sync::Barrier,
        release: &std::sync::Barrier,
    ) -> Result<RelationalForkOutcome, ForkStop> {
        let reservation = self
            .reserve_fork_target(target_branch)
            .map_err(RelationalBranchTransitionStop::Denied)?;
        let step = LiveBranchTransitionStep::new(self.medium.clone(), execution);
        let pause = || {
            reached.wait();
            release.wait();
        };
        self.fork_reserved_with_hooks(execution, &step, reservation, source, pause, || {}, false)
            .map(|installed| installed.outcome)
            .map_err(Into::into)
    }

    #[cfg(test)]
    pub(crate) fn fork_branch_with_post_install_test_pause(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        target_branch: BranchId,
        source: AdmittedRelationalForkSourceBasis,
        reached: &std::sync::Barrier,
        release: &std::sync::Barrier,
    ) -> Result<(RelationalForkOutcome, AdmittedRelationalBranchBasis), ForkStop> {
        let reservation = self
            .reserve_fork_target(target_branch)
            .map_err(RelationalBranchTransitionStop::Denied)?;
        let step = LiveBranchTransitionStep::new(self.medium.clone(), execution);
        let pause = || {
            reached.wait();
            release.wait();
        };
        self.fork_reserved_with_hooks(execution, &step, reservation, source, || {}, pause, true)
            .map(with_basis)
            .map_err(Into::into)
    }
}
