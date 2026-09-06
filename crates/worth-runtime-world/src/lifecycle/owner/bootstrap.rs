use std::sync::Arc;

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
        if *state == RuntimeWorldBootstrapState::Performed {
            return no_effect(RuntimeWorldBootstrapNoEffectCause::AlreadyBootstrapped);
        }
        if *state == RuntimeWorldBootstrapState::InProgress {
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
        let cancellation = intent.cancellation().cloned();
        let (creation, relational, signal, correspondence, generation) = intent.into_parts();
        let branch_reservation = match self.state.branches.reserve_root(self.owner_identity()) {
            Ok(reservation) => reservation,
            Err(_) => return no_effect(RuntimeWorldBootstrapNoEffectCause::CapacityExhausted),
        };

        let root_name = creation.name().clone();
        let (bootstrap_attempt, branch, lifecycle, commit_identity) = {
            let mut identities = self
                .state
                .identities
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            let bootstrap_attempt = match identities.bootstrap_attempt() {
                Ok(identity) => identity,
                Err(_) => return no_effect(RuntimeWorldBootstrapNoEffectCause::IdentityExhausted),
            };
            let branch =
                crate::identity::ProductBranchIdentity::issued(self.owner_identity(), root_name);
            let lifecycle = match identities.branch_incarnation() {
                Ok(identity) => identity,
                Err(_) => return no_effect(RuntimeWorldBootstrapNoEffectCause::IdentityExhausted),
            };
            let commit = match identities.composite_commit() {
                Ok(identity) => identity,
                Err(_) => return no_effect(RuntimeWorldBootstrapNoEffectCause::IdentityExhausted),
            };
            (bootstrap_attempt, branch, lifecycle, commit)
        };

        let basis = match admit_current(
            &self
                .state
                .identities
                .lock()
                .unwrap_or_else(|error| error.into_inner()),
            &self.state.relational.basis_port(),
            &self.state.signal.basis_port(),
            &self.state.bridge,
            relational,
            signal,
            correspondence,
        ) {
            Ok(basis) => basis,
            Err(denial) => return no_effect(map_basis_denial(denial)),
        };
        if basis.owner_identity() != self.owner_identity() {
            return no_effect(RuntimeWorldBootstrapNoEffectCause::ForeignBasis);
        }

        let root = match CompositeRuntimeWorldCommit::from_root_bootstrap(
            commit_identity.clone(),
            basis.clone(),
            bootstrap_attempt.clone(),
            None,
        ) {
            Ok(root) => Arc::new(root),
            Err(_) => return no_effect(RuntimeWorldBootstrapNoEffectCause::ForeignBasis),
        };
        let history_capacity = match self
            .state
            .history
            .reserve_commit_capacity(commit_identity, CompositeCommitParent::Root)
        {
            Ok(capacity) => capacity,
            Err(_) => return no_effect(RuntimeWorldBootstrapNoEffectCause::CapacityExhausted),
        };

        let mut root_rollback;
        // These are the unique component pins. They are acquired before any
        // retained-history protection is issued.
        let product_head = match self.state.retention.issue_product_head(&basis) {
            Ok(obligation) => obligation,
            Err(denial) => return no_effect(map_retention_denial(denial)),
        };
        let observation_components = match self.state.retention.issue_observation(root.as_ref()) {
            Ok(obligation) => obligation,
            Err(denial) => return no_effect(map_retention_denial(denial)),
        };

        let history_pins = match product_head.fork_history(&basis) {
            Ok(pins) => pins,
            Err(_) => return no_effect(RuntimeWorldBootstrapNoEffectCause::CapacityExhausted),
        };
        // No component-owner calls occur under this final admission guard.
        // Every consuming-call failure remains anchored by another acquired
        // exact pair: head + observation during history installation, observation
        // during head/cell construction, cell during observation construction,
        // and observation + installed history during root-cell installation.
        // Therefore no inner error drop can release the final owner lease.
        // This guard drops before outer rollback and acquired-pair custody.
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
        if cancellation
            .as_ref()
            .is_some_and(|token| token.is_cancelled())
        {
            return no_effect(RuntimeWorldBootstrapNoEffectCause::Cancelled);
        }
        let root_entry = match history_capacity.install(Arc::clone(&root), history_pins) {
            Ok(entry) => entry,
            Err(_) => return no_effect(RuntimeWorldBootstrapNoEffectCause::CapacityExhausted),
        };
        root_rollback = Some(
            self.state
                .history
                .arm_installed_root_rollback(root_entry.identity()),
        );
        let product_history = match self.state.history.protect_product_head(root.as_ref()) {
            Ok(protection) => protection,
            Err(_) => return no_effect(RuntimeWorldBootstrapNoEffectCause::CapacityExhausted),
        };
        let observation_history = match self.state.history.protect_explicit_commit(root.as_ref()) {
            Ok(protection) => protection,
            Err(_) => return no_effect(RuntimeWorldBootstrapNoEffectCause::CapacityExhausted),
        };
        let snapshot = match ProductBranchReferenceSnapshot::owner_issued(
            self.owner_identity(),
            branch,
            lifecycle,
            generation,
            Arc::clone(&root),
        ) {
            Ok(snapshot) => snapshot,
            Err(_) => return no_effect(RuntimeWorldBootstrapNoEffectCause::ForeignBasis),
        };
        let protection = match ProductBranchHeadProtection::bootstrap_issued(
            snapshot.clone(),
            product_head,
            product_history,
        ) {
            Ok(protection) => protection,
            Err(_) => return no_effect(RuntimeWorldBootstrapNoEffectCause::ForeignBasis),
        };
        let cell = match ProductBranchReferenceCell::new(protection) {
            Ok(cell) => cell,
            Err(_) => return no_effect(RuntimeWorldBootstrapNoEffectCause::ForeignBasis),
        };
        let observation = match ProductBranchObservation::owner_issued(
            snapshot,
            observation_components,
            observation_history,
        ) {
            Ok(observation) => observation,
            Err(_) => return no_effect(RuntimeWorldBootstrapNoEffectCause::ForeignBasis),
        };

        if branch_reservation
            .install_root(
                creation.name().clone(),
                cell_branch(&observation),
                cell_lifecycle(&observation),
                cell,
            )
            .is_err()
        {
            return no_effect(RuntimeWorldBootstrapNoEffectCause::CapacityExhausted);
        }
        root_rollback
            .take()
            .expect("successful root installation retains rollback custody")
            .commit();
        *bootstrap = RuntimeWorldBootstrapState::Performed;
        drop(bootstrap);
        PerformedRuntimeWorldBootstrap::new(bootstrap_attempt, basis, observation).into_outcome()
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
            *self
                .owner
                .state
                .bootstrap
                .lock()
                .unwrap_or_else(|error| error.into_inner()) =
                RuntimeWorldBootstrapState::Unperformed;
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
