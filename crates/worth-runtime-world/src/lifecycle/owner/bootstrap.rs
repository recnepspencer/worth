use std::sync::Arc;
mod durable;
mod occurrence_readmission;
mod preparation;
mod readmission;
pub(crate) use durable::WorldDurableBootstrapCustody;

use crate::basis::{admit_current, CompositeBasisAdmissionDenial};
use crate::branch::{
    NoEffectRuntimeWorldBootstrap, PerformedRuntimeWorldBootstrap, ProductBranchHeadProtection,
    ProductBranchObservation, ProductBranchReferenceCell, ProductBranchReferenceSnapshot,
    RuntimeWorldBootstrapIntent, RuntimeWorldBootstrapNoEffectCause, RuntimeWorldBootstrapOutcome,
};
use crate::history::{CompositeCommitParent, CompositeRuntimeWorldCommit};
use crate::lifecycle::owner::{RuntimeWorldBootstrapState, RuntimeWorldOwnerRoot};
use crate::retention::RetentionObligationDenial;

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub fn bootstrap_root(
        &self,
        intent: RuntimeWorldBootstrapIntent,
    ) -> RuntimeWorldBootstrapOutcome {
        if self.state.durable.is_some() {
            return no_effect(RuntimeWorldBootstrapNoEffectCause::DurableBootstrapRequired);
        }
        if intent
            .cancellation()
            .is_some_and(|token| token.is_cancelled())
        {
            return no_effect(RuntimeWorldBootstrapNoEffectCause::Cancelled);
        }
        let mut state = self
            .state
            .bootstrap
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if matches!(*state, RuntimeWorldBootstrapState::Performed) {
            return no_effect(RuntimeWorldBootstrapNoEffectCause::AlreadyBootstrapped);
        }
        if !matches!(*state, RuntimeWorldBootstrapState::Unperformed) {
            return no_effect(RuntimeWorldBootstrapNoEffectCause::OwnerUnavailable);
        }
        let close = self
            .state
            .close
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if !self.owner_is_present()
            || close.state() != super::super::close::RuntimeWorldCloseState::Open
        {
            return no_effect(RuntimeWorldBootstrapNoEffectCause::OwnerUnavailable);
        }
        drop(close);
        *state = RuntimeWorldBootstrapState::InProgress;
        drop(state);

        let mut rollback = BootstrapRollback {
            owner: self,
            completed: false,
        };
        let result = self.perform_bootstrap(intent);
        if matches!(&result, RuntimeWorldBootstrapOutcome::Performed(_)) {
            rollback.completed = true;
        }
        result
    }

    fn perform_bootstrap(
        &self,
        intent: RuntimeWorldBootstrapIntent,
    ) -> RuntimeWorldBootstrapOutcome {
        let mut prepared = match self.prepare_memory_root(intent) {
            Ok(prepared) => prepared,
            Err(cause) => return no_effect(cause),
        };
        let mut bootstrap = self
            .state
            .bootstrap
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let close = self.state.close.lock().unwrap_or_else(|e| e.into_inner());
        if !self.owner_is_present()
            || close.state() != super::super::close::RuntimeWorldCloseState::Open
        {
            return no_effect(RuntimeWorldBootstrapNoEffectCause::OwnerUnavailable);
        }
        drop(close);
        let result = prepared.install();
        if result.is_ok() {
            *bootstrap = RuntimeWorldBootstrapState::Performed;
        }
        drop(bootstrap);
        match result {
            Ok(performed) => performed.into_outcome(),
            Err(cause) => no_effect(cause),
        }
    }
}

struct BootstrapRollback<'a, D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    owner: &'a RuntimeWorldOwnerRoot<D, I, E, Ctx, T>,
    completed: bool,
}

impl<D, I, E, Ctx, T> Drop for BootstrapRollback<'_, D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    fn drop(&mut self) {
        if !self.completed {
            let mut state = self
                .owner
                .state
                .bootstrap
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            if matches!(*state, RuntimeWorldBootstrapState::InProgress) {
                *state = RuntimeWorldBootstrapState::Unperformed;
            }
        }
    }
}

fn no_effect(cause: RuntimeWorldBootstrapNoEffectCause) -> RuntimeWorldBootstrapOutcome {
    NoEffectRuntimeWorldBootstrap::new(cause).into_outcome()
}

fn map_basis_denial(denial: CompositeBasisAdmissionDenial) -> RuntimeWorldBootstrapNoEffectCause {
    match denial {
        CompositeBasisAdmissionDenial::Correspondence(_) => {
            RuntimeWorldBootstrapNoEffectCause::IncompatibleCorrespondence
        }
        CompositeBasisAdmissionDenial::Relational(_) | CompositeBasisAdmissionDenial::Signal(_) => {
            RuntimeWorldBootstrapNoEffectCause::ForeignBasis
        }
    }
}

fn map_retention_denial(denial: RetentionObligationDenial) -> RuntimeWorldBootstrapNoEffectCause {
    match denial {
        RetentionObligationDenial::LeaseIdentityExhausted => {
            RuntimeWorldBootstrapNoEffectCause::IdentityExhausted
        }
        RetentionObligationDenial::ForeignOwner { .. }
        | RetentionObligationDenial::Relational(_)
        | RetentionObligationDenial::Signal(_)
        | RetentionObligationDenial::OwnerOperationPanicked => {
            RuntimeWorldBootstrapNoEffectCause::OwnerUnavailable
        }
        _ => RuntimeWorldBootstrapNoEffectCause::CapacityExhausted,
    }
}

trait BootstrapOutcomeExt {
    fn into_outcome(self) -> RuntimeWorldBootstrapOutcome;
}

impl BootstrapOutcomeExt for NoEffectRuntimeWorldBootstrap {
    fn into_outcome(self) -> RuntimeWorldBootstrapOutcome {
        RuntimeWorldBootstrapOutcome::NoEffect(self)
    }
}

impl BootstrapOutcomeExt for PerformedRuntimeWorldBootstrap {
    fn into_outcome(self) -> RuntimeWorldBootstrapOutcome {
        RuntimeWorldBootstrapOutcome::Performed(self)
    }
}

fn cell_branch(observation: &ProductBranchObservation) -> crate::identity::ProductBranchIdentity {
    observation.branch_identity().clone()
}

fn cell_lifecycle(
    observation: &ProductBranchObservation,
) -> crate::identity::ProductBranchIncarnation {
    observation.lifecycle_incarnation()
}
