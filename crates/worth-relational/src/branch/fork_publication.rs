//! One original source-rechecked fork kernel, shared by all Native fork profiles.
use super::source_validation::ForkSourceInput;
use super::{RelationalForkDenial, RelationalForkOutcome};
use crate::branch::fork_source_basis::AdmittedRelationalForkSourceBasis;
use crate::branch::{
    AdmittedRelationalBranchBasis, RelationalBranchTransitionDurableStep, RelationalForkPort,
    RelationalForkTargetReservation,
};
use crate::runtime::BranchTransitionRecordError;
type ForkFailure<S> = BranchTransitionRecordError<RelationalForkDenial, S>;

pub(super) struct ForkInstallationOutcome {
    pub(super) outcome: RelationalForkOutcome,
    pub(super) basis: Option<AdmittedRelationalBranchBasis>,
    pub(super) evidence:
        Option<worth_proof::DurablyCommitted<crate::branch::RelationalCommittedTransitionEvidence>>,
}

impl RelationalForkPort {
    /// The fork transition, in phase order: custody and source validation,
    /// target preparation, head retention and basis issuance (every fallible
    /// step), then the durable step with the stale-source re-check inside the
    /// reservation, and only then the target's installation.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn fork_reserved_with_hooks<S: RelationalBranchTransitionDurableStep>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        step: &S,
        reservation: RelationalForkTargetReservation,
        source: AdmittedRelationalForkSourceBasis,
        pause_before_install: impl FnOnce(),
        pause_after_install: impl FnOnce(),
        return_basis: bool,
    ) -> Result<ForkInstallationOutcome, ForkFailure<S::Stop>> {
        self.fork_reserved_input_with_hooks(
            execution,
            step,
            reservation,
            ForkSourceInput::Current(source),
            pause_before_install,
            pause_after_install,
            return_basis,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn fork_reserved_input_with_hooks<S: RelationalBranchTransitionDurableStep>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        step: &S,
        reservation: RelationalForkTargetReservation,
        source: ForkSourceInput,
        pause_before_install: impl FnOnce(),
        pause_after_install: impl FnOnce(),
        return_basis: bool,
    ) -> Result<ForkInstallationOutcome, ForkFailure<S::Stop>> {
        let denied = BranchTransitionRecordError::Denied;
        if !self.owner.owns_reservation(&reservation) {
            return Err(denied(RelationalForkDenial::ForeignRuntime));
        }
        // Declared before every Native guard: all early preparation failures
        // release their returned participant-owned lookup outside those guards.
        let mut lookup_cleanup = None;
        let _operation = self
            .lifecycle
            .admit()
            .ok_or(denied(RelationalForkDenial::OwnerUnavailable))?;
        let publication = self.owner.begin_branch_transition();
        let validated = self.validate_fork_input(source).map_err(denied)?;
        // The fork association uses the actual current registration through
        // durable installation. A rebind cannot turn its pending lookup live.
        let source_epoch = if validated.source_root.managed_source_image.get().is_some() {
            Some(self.publication.companion_epoch().map_err(|_| {
                denied(RelationalForkDenial::ManagedSource(
                    crate::mvcc::CompanionPreflightStop::RegistrationChanged,
                ))
            })?)
        } else {
            None
        };
        let companion = source_epoch
            .as_ref()
            .map(|epoch| {
                epoch
                    .active()
                    .map_err(|_| {
                        denied(RelationalForkDenial::ManagedSource(
                            crate::mvcc::CompanionPreflightStop::RegistrationChanged,
                        ))
                    })?
                    .ok_or_else(|| {
                        denied(RelationalForkDenial::ManagedSource(
                            crate::mvcc::CompanionPreflightStop::RegistrationChanged,
                        ))
                    })
            })
            .transpose()?;
        pause_before_install();
        let mut prepared = self
            .prepare_fork_target(
                execution,
                reservation,
                &validated,
                companion,
                &mut lookup_cleanup,
            )
            .map_err(denied)?;
        prepared.indexes = self
            .indexes
            .prepare_fork_selection(
                execution,
                validated.descriptor.source_branch(),
                prepared.reservation.branch_id(),
                &validated.source_root,
            )
            .map_err(|stop| denied(RelationalForkDenial::IndexPreparation(Box::new(stop))))?;
        let mut installable = Some(
            self.prepare_installation(prepared, return_basis)
                .map_err(denied)?,
        );
        let source_cell = self
            .owner
            .branch_cell(validated.descriptor.source_branch())
            .ok_or(denied(RelationalForkDenial::SourceBranchMissing))?
            .publication_cell();
        let _coordination = source_cell.coordination().enter();
        let source_state = source_cell.enter_state();
        let transition = validated.transition(
            installable
                .as_ref()
                .expect("prepared fork target")
                .target_branch()
                .clone(),
        );
        let installed = publication.record_with_cutover(
            execution,
            step,
            validated.source_root.retained_schema_authority(),
            || {
                self.recheck_fork_source(&validated, &source_state)?;
                // Coordination still excludes source writers. Readers must not
                // wait on source state across durable I/O or registry insertion.
                drop(source_state);
                Ok(transition)
            },
            |_, evidence| {
                let (outcome, basis, cleanup) = self.install_fork_target(
                    &validated,
                    installable.take().expect("one durable fork cutover"),
                );
                (outcome, basis, cleanup, evidence)
            },
        );
        drop(_coordination);
        drop(publication);
        drop(source_epoch);
        // Refused/unresolved callbacks never own this preparation. Successful
        // cutover also returns its derived cleanup instead of destroying it
        // inside Native coordination or the canonical position reservation.
        drop(installable);
        let (outcome, basis, cleanup, evidence) = installed?;
        drop(cleanup);
        drop(lookup_cleanup);
        pause_after_install();
        Ok(ForkInstallationOutcome {
            outcome,
            basis,
            evidence,
        })
    }
}

pub(super) fn with_basis(
    installed: ForkInstallationOutcome,
) -> (RelationalForkOutcome, AdmittedRelationalBranchBasis) {
    (
        installed.outcome,
        installed
            .basis
            .expect("basis-enabled fork installation returns its original basis"),
    )
}
