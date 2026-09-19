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

use worth_relational::facade::branch::RelationalBranchReferenceObservation;
use worth_relational::facade::branch::{
    AdmittedRelationalBranchBasis, AdmittedRelationalForkSourceBasis, RelationalBranchVersion,
    RelationalForkDenial, RelationalForkSourceDescriptor,
};
use worth_relational::facade::history::BranchId;

#[cfg(test)]
#[path = "forks/source_axes_tests.rs"]
mod source_axes_tests;

/// One owner fork was refused. A creation fork is the first effect its owner
/// performs, so the denial always names why no branch exists.
pub(super) struct ForkFailure {
    pub(super) denial: RuntimeWorldBranchAdmissionDenial,
}

/// Ask the Relational owner for the exact destination this plan named, from a
/// source Runtime World observes for itself and proves is still the admitted
/// one. Custody is recorded only after the owner reports the branch exists.
pub(super) fn fork_relational<D, I, E, Ctx, T>(
    owner: &RuntimeWorldOwnerRoot<D, I, E, Ctx, T>,
    attempt: &mut ReservedBranchCreationAttempt,
    destination: &CreationDestination,
) -> Result<RelationalAttemptProgress, ForkFailure>
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
    let source = observe_exact_fork_source(owner, attempt)?;
    let custody = attempt.take_relational_custody().ok_or(ForkFailure {
        denial: RuntimeWorldBranchAdmissionDenial::CustodyCapacityExhausted,
    })?;
    let fork_port = owner.state.relational.fork_port();
    let reservation = fork_port
        .reserve_fork_target(target.clone())
        .map_err(|denial| relational_fork_failure(&denial))?;
    let (fork, target_basis) = fork_port
        .fork_reserved_with_basis(reservation, source)
        .map_err(|denial| relational_fork_failure(&denial))?;
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

/// Observe the fork source the admitted basis names and prove it is still that
/// exact source. Runtime World issues the fork token itself: no caller-held
/// source token exists, and a source that moved between admission and this call
/// denies here, before the destination is reserved or any branch is created.
fn observe_exact_fork_source<D, I, E, Ctx, T>(
    owner: &RuntimeWorldOwnerRoot<D, I, E, Ctx, T>,
    attempt: &ReservedBranchCreationAttempt,
) -> Result<AdmittedRelationalForkSourceBasis, ForkFailure>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    let admitted = attempt.source().basis().relational_basis();
    let (descriptor, source) = owner
        .state
        .relational
        .fork_port()
        .observe_fork_source(admitted.descriptor().branch_id())
        .map_err(|denial| relational_fork_failure(&denial))?;
    if source_is_the_admitted_basis(&observed_axes(&descriptor), &admitted_axes(admitted)) {
        Ok(source)
    } else {
        Err(ForkFailure {
            denial: RuntimeWorldBranchAdmissionDenial::ForkSourceChanged,
        })
    }
}

/// The axes a fresh fork-source observation shares with the admitted source
/// basis, lifted off both owner types so the comparison is total and can be
/// proved one axis at a time without an owner.
///
/// Two of the four axes are live and two are structural. `observation` and
/// `truth_version` are live: the Relational owner moves them whenever the
/// source branch moves, and that drift is exactly what this comparison
/// exists to refuse. `source_branch` and `runtime_instance_id` are
/// structural: the fresh observation is requested *for* the admitted basis's
/// own branch id, on this process's own owner, so they can differ only if
/// the port answers about a branch it was never asked about. They are
/// compared anyway, and proved case by case, so a later port, transport, or
/// cross-instance change cannot make that mismatch silent.
struct ForkSourceAxes<'a> {
    runtime_instance_id: u64,
    source_branch: &'a BranchId,
    observation: &'a RelationalBranchReferenceObservation,
    truth_version: RelationalBranchVersion,
}

/// A single differing axis means the branch the owner would fork is not the
/// branch this creation was admitted against.
fn source_is_the_admitted_basis(
    observed: &ForkSourceAxes<'_>,
    admitted: &ForkSourceAxes<'_>,
) -> bool {
    observed.runtime_instance_id == admitted.runtime_instance_id
        && observed.source_branch == admitted.source_branch
        && observed.observation == admitted.observation
        && observed.truth_version == admitted.truth_version
}

/// Axes of the source observation the Relational owner just issued.
fn observed_axes(descriptor: &RelationalForkSourceDescriptor) -> ForkSourceAxes<'_> {
    ForkSourceAxes {
        runtime_instance_id: descriptor.runtime_instance_id(),
        source_branch: descriptor.source_branch(),
        observation: descriptor.observation(),
        truth_version: descriptor.truth_version(),
    }
}

/// Axes of the source basis this creation was admitted against.
fn admitted_axes(admitted: &AdmittedRelationalBranchBasis) -> ForkSourceAxes<'_> {
    let basis = admitted.descriptor();
    ForkSourceAxes {
        runtime_instance_id: basis.runtime_instance_id(),
        source_branch: basis.branch_id(),
        observation: basis.reference(),
        truth_version: basis.truth_version(),
    }
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
    let mutation_port = owner.state.signal.mutation_port();
    let reservation = mutation_port
        .reserve_fork_exact(target.clone(), attempt.source().basis().signal_basis())
        .map_err(|denial| signal_fork_failure(&denial))?;
    #[cfg(test)]
    super::boundary_control::pause_at_signal_fork_cutoff(owner.owner_identity());
    let fork = mutation_port
        .fork_reserved_exact(reservation, cancellation.signal_token())
        .map_err(|denial| signal_fork_failure(&denial))?;
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

fn relational_fork_failure(denial: &RelationalForkDenial) -> ForkFailure {
    let denial = match denial {
        RelationalForkDenial::RetentionCapacityExhausted
        | RelationalForkDenial::RetentionIdentityExhausted => {
            RuntimeWorldBranchAdmissionDenial::CapacityExhausted
        }
        _ => RuntimeWorldBranchAdmissionDenial::OwnerUnavailable,
    };
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
