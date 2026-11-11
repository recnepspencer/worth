use crate::branch::{
    ComponentBranchTarget, RelationalBranchCreationPlan, RuntimeWorldBranchAdmissionDenial,
    SignalBranchCreationPlan,
};
use crate::publication::{
    RelationalAttemptProgress, ReservedBranchCreationAttempt, RuntimeWorldCancellationToken,
    SignalAttemptProgress,
};

use super::super::super::super::RuntimeWorldOwnerRoot;
use super::{install_custody, CreationDestination};

use worth_relational::facade::branch::{RelationalBranchTransitionStop, RelationalForkDenial};

/// One owner fork was refused. A creation fork is the first effect its owner
/// performs, so the denial always names why no branch exists.
pub(super) struct ForkFailure {
    pub(super) denial: RuntimeWorldBranchAdmissionDenial,
}

pub(super) enum RelationalForkFailure {
    Denied(ForkFailure),
    Unresolved(worth_relational::facade::durability::RelationalDurabilityUnresolved),
}

impl From<ForkFailure> for RelationalForkFailure {
    fn from(denial: ForkFailure) -> Self {
        Self::Denied(denial)
    }
}

/// Ask the Relational owner for the exact destination this plan named, from a
/// source Runtime World observes for itself and proves is still the admitted
/// one. Custody is recorded only after the owner reports the branch exists.
pub(super) fn fork_relational<D, I, E, Ctx, T>(
    owner: &RuntimeWorldOwnerRoot<D, I, E, Ctx, T>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    attempt: &mut ReservedBranchCreationAttempt,
    destination: &CreationDestination,
) -> Result<RelationalAttemptProgress, RelationalForkFailure>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    let target = match attempt.plan().relational() {
        RelationalBranchCreationPlan::ReuseExact => {
            return Ok(RelationalAttemptProgress::untouched())
        }
        RelationalBranchCreationPlan::ForkExact { target } => target.clone(),
    };
    // Observing the fork source is already Relational-owner-facing work, so the
    // contact is charged before it, not after the fork returns: a denial from
    // any of this leg's port calls still cost the owner one contact.
    attempt.counters_mut().record_relational_owner_contact();
    let retained = match attempt.plan().source_kind() {
        crate::branch::BranchCreationSourceKind::Current => None,
        crate::branch::BranchCreationSourceKind::Retained => Some(
            attempt
                .take_retained_native()
                .expect("one retained source is consumed by its owner leg"),
        ),
    };
    let source = if retained.is_none() {
        Some(
            owner
                .observe_creation_native_source(attempt.source())
                .map_err(|denial| ForkFailure { denial })?,
        )
    } else {
        None
    };
    let custody = attempt.take_relational_custody().ok_or(ForkFailure {
        denial: RuntimeWorldBranchAdmissionDenial::CustodyCapacityExhausted,
    })?;
    let fork_port = owner.state.relational_services().fork_port();
    let reservation = fork_port
        .reserve_fork_target(target.clone())
        .map_err(|denial| relational_fork_failure(&denial))?;
    let (fork, target_basis) = match retained {
        Some(retained) => {
            fork_port.fork_reserved_retained_with_basis(execution, reservation, retained)
        }
        None => fork_port.fork_reserved_with_basis(
            execution,
            reservation,
            source.expect("current receiver observed its exact source"),
        ),
    }
    .map_err(relational_fork_stop)?;
    let target_identity = fork.target_identity().clone();
    let progress = RelationalAttemptProgress::forked(fork, target_basis);
    attempt.record_progress(&crate::publication::CompositeAttemptProgress::new(
        progress.retained_image(),
        SignalAttemptProgress::untouched(),
    ));
    install_custody(
        custody,
        destination,
        ComponentBranchTarget::Relational {
            target,
            identity: target_identity,
        },
    );
    Ok(progress)
}

/// Ask the Signal owner for the exact destination this plan named. The
/// reservation and the fork both run under the Runtime World token's own
/// Signal token; no detached cancellation source exists on this path.
pub(super) fn fork_signal<D, I, E, Ctx, T>(
    owner: &RuntimeWorldOwnerRoot<D, I, E, Ctx, T>,
    attempt: &mut ReservedBranchCreationAttempt,
    destination: &CreationDestination,
    cancellation: &RuntimeWorldCancellationToken,
) -> Result<SignalAttemptProgress, ForkFailure>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    let target = match attempt.plan().signal() {
        SignalBranchCreationPlan::ReuseExact => return Ok(SignalAttemptProgress::untouched()),
        SignalBranchCreationPlan::ForkExact { target } => target.clone(),
    };
    attempt.counters_mut().record_signal_owner_contact();
    let custody = attempt.take_signal_custody().ok_or(ForkFailure {
        denial: RuntimeWorldBranchAdmissionDenial::CustodyCapacityExhausted,
    })?;
    let services = owner.state.signal.memory_services().ok_or(ForkFailure {
        denial: RuntimeWorldBranchAdmissionDenial::DurableBranchPublicationRequired,
    })?;
    let mutation_port = services.mutation_port();
    let reservation = mutation_port
        .reserve_fork_exact(target.clone(), attempt.source().basis().signal_basis())
        .map_err(|denial| signal_fork_failure(&denial))?;
    #[cfg(test)]
    super::boundary_control::pause_at_signal_fork_cutoff(owner.owner_identity());
    let fork = mutation_port
        .fork_reserved_exact(reservation, cancellation.signal_token())
        .map_err(|denial| signal_fork_failure(&denial))?;
    // SignalBranchForkReservation::consume_for returns only owner_issued
    // outcomes, whose required retirement reference it obtains before return.
    // This owner-service path never uses Signal's unowned fork constructor.
    let retirement_reference = fork
        .retirement_reference()
        .expect("Runtime World forks through the owner service that issues cleanup authority")
        .clone();
    let progress = SignalAttemptProgress::forked(fork);
    let (relational, _) = attempt.progress().retained_image().into_parts();
    attempt.record_progress(&crate::publication::CompositeAttemptProgress::new(
        relational,
        progress.retained_image(),
    ));
    install_custody(
        custody,
        destination,
        ComponentBranchTarget::Signal {
            target,
            reference: retirement_reference,
        },
    );
    Ok(progress)
}

/// The Relational fork did not take effect live. A fork whose durable fate is
/// unknown is a possible durable effect, retained by World until reopening
/// resolves it. Only definite refusals become admission denials.
fn relational_fork_stop(
    stop: RelationalBranchTransitionStop<RelationalForkDenial>,
) -> RelationalForkFailure {
    use worth_execution::{MapKernelStop, WorkCeilingDenial};
    let denial = match stop {
        RelationalBranchTransitionStop::Denied(denial) => {
            return relational_fork_failure(&denial).into()
        }
        RelationalBranchTransitionStop::RequestStopped(WorkCeilingDenial::Stopped(
            MapKernelStop::DeadlineElapsed,
        )) => RuntimeWorldBranchAdmissionDenial::DeadlineBeforeEffect,
        RelationalBranchTransitionStop::RequestStopped(WorkCeilingDenial::Stopped(_)) => {
            RuntimeWorldBranchAdmissionDenial::CancelledBeforeEffect
        }
        RelationalBranchTransitionStop::RequestStopped(_) => {
            RuntimeWorldBranchAdmissionDenial::OwnerUnavailable
        }
        RelationalBranchTransitionStop::OriginalInputsStopped(stop) => {
            return ForkFailure {
                denial: original_input_denial(stop),
            }
            .into();
        }
        RelationalBranchTransitionStop::DurabilityUnresolved(unresolved) => {
            return RelationalForkFailure::Unresolved(unresolved);
        }
    };
    ForkFailure { denial }.into()
}

fn relational_fork_failure(denial: &RelationalForkDenial) -> ForkFailure {
    let denial = super::super::super::native_fork_denial::admission_denial(denial);
    ForkFailure { denial }
}

fn signal_fork_failure(
    denial: &worth_signal::facade::branch::SignalBranchForkOperationDenial,
) -> ForkFailure {
    let denial = match super::super::super::super::execution_service::map_fork_no_effect(denial) {
        crate::publication::NoEffectCause::CancelledBeforeEffect => {
            RuntimeWorldBranchAdmissionDenial::CancelledBeforeEffect
        }
        crate::publication::NoEffectCause::CapacityExhausted => {
            RuntimeWorldBranchAdmissionDenial::CapacityExhausted
        }
        _ => RuntimeWorldBranchAdmissionDenial::OwnerUnavailable,
    };
    ForkFailure { denial }
}

fn original_input_denial(
    stop: worth_relational::facade::durability::RelationalRecoveryStop,
) -> RuntimeWorldBranchAdmissionDenial {
    use worth_relational::facade::durability::RelationalRecoveryStop as Stop;
    match stop {
        Stop::Allocation(_) => RuntimeWorldBranchAdmissionDenial::CapacityExhausted,
        Stop::Durability(_) | Stop::AdmissionImage(_) => {
            RuntimeWorldBranchAdmissionDenial::OwnerUnavailable
        }
        Stop::Request(cause) => super::super::super::native_fork_denial::execution_denial(cause),
        Stop::Content(cause) => {
            super::super::super::native_fork_denial::execution_denial(cause.into_execution_cause())
        }
    }
}
