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
type ForkFailure<S> = BranchTransitionRecordError<RelationalForkDenial, S>;

impl RelationalForkPort {
    /// Observe an exact live source and issue the owner-sealed fork-only
    /// token. Empty branches do not produce a fork source.
    pub fn observe_fork_source(
        &self,
        source_branch: &BranchId,
    ) -> Result<
        (
            RelationalForkSourceDescriptor,
            AdmittedRelationalForkSourceBasis,
        ),
        RelationalForkDenial,
    > {
        let _operation = self
            .lifecycle
            .admit()
            .ok_or(RelationalForkDenial::OwnerUnavailable)?;
        let source_cell = self
            .owner
            .branch_cell(source_branch)
            .ok_or(RelationalForkDenial::SourceBranchMissing)?;
        let source_snapshot = source_cell.atomic_snapshot();
        match source_snapshot.lifecycle_posture() {
            RelationalBranchLifecyclePosture::Live => {}
            RelationalBranchLifecyclePosture::Archived => {
                return Err(RelationalForkDenial::SourceArchived);
            }
            RelationalBranchLifecyclePosture::Deleting => {
                return Err(RelationalForkDenial::SourceDeleting);
            }
        }
        let observation = source_snapshot.observation();
        let truth_version = source_snapshot.truth_version();
        self.owner.record_lookup();
        if observation.target().is_empty() {
            return Err(RelationalForkDenial::EmptySource);
        }
        let descriptor = RelationalForkSourceDescriptor::new(
            self.runtime_instance_id,
            observation,
            source_branch.clone(),
            truth_version,
        );
        let token = admit_relational_fork_source(descriptor.clone());
        Ok((descriptor, token))
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
        let step = LiveBranchTransitionStep::new(self.medium.clone(), execution);
        self.fork_reserved_with_hooks(execution, &step, reservation, source, || {}, || {}, false)
            .map(|(outcome, _)| outcome)
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
        let step = LiveBranchTransitionStep::new(self.medium.clone(), execution);
        self.fork_reserved_with_hooks(execution, &step, reservation, source, || {}, || {}, true)
            .map(with_basis)
            .map_err(Into::into)
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
    ) -> Result<RelationalForkOutcome, ForkFailure<S::Stop>> {
        let denied = BranchTransitionRecordError::Denied;
        let (_, source) = self.observe_fork_source(source_branch).map_err(denied)?;
        let reservation = self.reserve_fork_target(target_branch).map_err(denied)?;
        self.fork_reserved_with_hooks(execution, step, reservation, source, || {}, || {}, false)
            .map(|(outcome, _)| outcome)
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
            .map(|(outcome, _)| outcome)
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

    /// The fork transition, in phase order: custody and source validation,
    /// target preparation, head retention and basis issuance (every fallible
    /// step), then the durable step with the stale-source re-check inside the
    /// reservation, and only then the target's installation.
    #[allow(clippy::too_many_arguments)]
    fn fork_reserved_with_hooks<S: RelationalBranchTransitionDurableStep>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        step: &S,
        reservation: RelationalForkTargetReservation,
        source: AdmittedRelationalForkSourceBasis,
        pause_before_install: impl FnOnce(),
        pause_after_install: impl FnOnce(),
        return_basis: bool,
    ) -> Result<(RelationalForkOutcome, Option<AdmittedRelationalBranchBasis>), ForkFailure<S::Stop>>
    {
        let denied = BranchTransitionRecordError::Denied;
        if !self.owner.owns_reservation(&reservation) {
            return Err(denied(RelationalForkDenial::ForeignRuntime));
        }
        let _operation = self
            .lifecycle
            .admit()
            .ok_or(denied(RelationalForkDenial::OwnerUnavailable))?;
        let (descriptor, _authority) = source.into_parts();
        let validated = self.validate_fork_source(descriptor).map_err(denied)?;
        pause_before_install();
        let prepared = self
            .prepare_fork_target(reservation, &validated)
            .map_err(denied)?;
        let installable = self
            .prepare_installation(prepared, return_basis)
            .map_err(denied)?;
        let source_cell = self
            .owner
            .branch_cell(validated.descriptor.source_branch())
            .ok_or(denied(RelationalForkDenial::SourceBranchMissing))?
            .publication_cell();
        let _coordination = source_cell.coordination().enter();
        let source_state = source_cell.enter_state();
        let transition = validated.transition(installable.target_branch().clone());
        let installed = self.owner.record_transition_with_cutover(
            execution,
            step,
            || {
                self.recheck_fork_source(&validated, &source_state)
                    .map(|()| transition)
            },
            |_| self.install_fork_target(&validated, installable),
        )?;
        drop(source_state);
        drop(_coordination);
        pause_after_install();
        Ok(installed)
    }
}

fn with_basis(
    (outcome, basis): (RelationalForkOutcome, Option<AdmittedRelationalBranchBasis>),
) -> (RelationalForkOutcome, AdmittedRelationalBranchBasis) {
    (
        outcome,
        basis.expect("basis-enabled fork installation returns its basis"),
    )
}
