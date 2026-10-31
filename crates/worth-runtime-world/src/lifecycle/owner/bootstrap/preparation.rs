//! Build the one root destination and acquire all existing custody before publication.
use super::*;
use crate::basis::AdmittedCompositeRuntimeWorldBasis;
use crate::branch::{registry::ProductBranchRegistryReservation, ProductBranchName};
use crate::history::InstalledRootCommitRollback;
use crate::identity::RuntimeWorldBootstrapAttemptIdentity;

pub(super) struct PreparedRoot {
    pub(super) attempt: RuntimeWorldBootstrapAttemptIdentity,
    pub(super) basis: AdmittedCompositeRuntimeWorldBasis,
    pub(super) observation: ProductBranchObservation,
    reservation: ProductBranchRegistryReservation,
    name: ProductBranchName,
    cell: Option<ProductBranchReferenceCell>,
    rollback: Option<InstalledRootCommitRollback>,
    recovered: bool,
}
impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(super) fn prepare_root(
        &self,
        intent: RuntimeWorldBootstrapIntent,
    ) -> Result<PreparedRoot, RuntimeWorldBootstrapNoEffectCause> {
        let cancellation = intent.cancellation().cloned();
        let (creation, relational, signal, correspondence, generation, posture) =
            intent.into_parts();
        let branch_reservation = match self.state.branches.reserve_root(self.owner_identity()) {
            Ok(reservation) => reservation,
            Err(_) => return Err(RuntimeWorldBootstrapNoEffectCause::CapacityExhausted),
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
                Err(_) => return Err(RuntimeWorldBootstrapNoEffectCause::IdentityExhausted),
            };
            let branch =
                crate::identity::ProductBranchIdentity::issued(self.owner_identity(), root_name);
            let lifecycle = match identities.branch_incarnation() {
                Ok(identity) => identity,
                Err(_) => return Err(RuntimeWorldBootstrapNoEffectCause::IdentityExhausted),
            };
            let commit = match identities.composite_commit() {
                Ok(identity) => identity,
                Err(_) => return Err(RuntimeWorldBootstrapNoEffectCause::IdentityExhausted),
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
            Err(denial) => return Err(map_basis_denial(denial)),
        };
        if basis.owner_identity() != self.owner_identity() {
            return Err(RuntimeWorldBootstrapNoEffectCause::ForeignBasis);
        }

        let root = match CompositeRuntimeWorldCommit::from_root_bootstrap(
            commit_identity.clone(),
            basis.clone(),
            bootstrap_attempt.clone(),
            None,
        ) {
            Ok(root) => Arc::new(root),
            Err(_) => return Err(RuntimeWorldBootstrapNoEffectCause::ForeignBasis),
        };
        let history_capacity = match self
            .state
            .history
            .reserve_commit_capacity(commit_identity, CompositeCommitParent::Root)
        {
            Ok(capacity) => capacity,
            Err(_) => return Err(RuntimeWorldBootstrapNoEffectCause::CapacityExhausted),
        };

        // These are the unique component pins. They are acquired before any
        // retained-history protection is issued.
        let product_head = match self.state.retention.issue_product_head(&basis) {
            Ok(obligation) => obligation,
            Err(denial) => return Err(map_retention_denial(denial)),
        };
        let observation_components = match self.state.retention.issue_observation(root.as_ref()) {
            Ok(obligation) => obligation,
            Err(denial) => return Err(map_retention_denial(denial)),
        };

        let history_pins = match product_head.fork_history(&basis) {
            Ok(pins) => pins,
            Err(_) => return Err(RuntimeWorldBootstrapNoEffectCause::CapacityExhausted),
        };
        if !self.owner_is_present() {
            return Err(RuntimeWorldBootstrapNoEffectCause::OwnerUnavailable);
        }
        if cancellation
            .as_ref()
            .is_some_and(|token| token.is_cancelled())
        {
            return Err(RuntimeWorldBootstrapNoEffectCause::Cancelled);
        }
        let root_entry = match history_capacity.install(Arc::clone(&root), history_pins) {
            Ok(entry) => entry,
            Err(_) => return Err(RuntimeWorldBootstrapNoEffectCause::CapacityExhausted),
        };
        let root_rollback = Some(
            self.state
                .history
                .arm_installed_root_rollback(root_entry.identity()),
        );
        let product_history = match self.state.history.protect_product_head(root.as_ref()) {
            Ok(protection) => protection,
            Err(_) => return Err(RuntimeWorldBootstrapNoEffectCause::CapacityExhausted),
        };
        let observation_history = match self.state.history.protect_explicit_commit(root.as_ref()) {
            Ok(protection) => protection,
            Err(_) => return Err(RuntimeWorldBootstrapNoEffectCause::CapacityExhausted),
        };
        let snapshot = match ProductBranchReferenceSnapshot::owner_issued(
            self.owner_identity(),
            branch,
            lifecycle,
            generation,
            Arc::clone(&root),
        ) {
            Ok(snapshot) => snapshot,
            Err(_) => return Err(RuntimeWorldBootstrapNoEffectCause::ForeignBasis),
        };
        let protection = match ProductBranchHeadProtection::bootstrap_issued(
            snapshot.clone(),
            product_head,
            product_history,
        ) {
            Ok(protection) => protection,
            Err(_) => return Err(RuntimeWorldBootstrapNoEffectCause::ForeignBasis),
        };
        let cell = match ProductBranchReferenceCell::new(protection) {
            Ok(cell) => cell,
            Err(_) => return Err(RuntimeWorldBootstrapNoEffectCause::ForeignBasis),
        };
        let observation = match ProductBranchObservation::owner_issued(
            snapshot,
            observation_components,
            observation_history,
        ) {
            Ok(observation) => observation,
            Err(_) => return Err(RuntimeWorldBootstrapNoEffectCause::ForeignBasis),
        };

        Ok(PreparedRoot {
            attempt: bootstrap_attempt,
            basis,
            observation,
            reservation: branch_reservation,
            name: creation.name().clone(),
            cell: Some(cell),
            rollback: root_rollback,
            recovered: posture,
        })
    }
}
impl PreparedRoot {
    /// A durable selection may never roll its original staged history back.
    pub(super) fn retain_selected_history(&mut self) {
        if let Some(rollback) = self.rollback.take() {
            rollback.commit();
        }
    }
    pub(super) fn install(
        &mut self,
    ) -> Result<PerformedRuntimeWorldBootstrap, RuntimeWorldBootstrapNoEffectCause> {
        if self.cell.is_some() {
            self.reservation
                .install_root_borrowed(
                    self.name.clone(),
                    cell_branch(&self.observation),
                    cell_lifecycle(&self.observation),
                    &mut self.cell,
                )
                .map_err(|_| RuntimeWorldBootstrapNoEffectCause::CapacityExhausted)?;
        }
        self.retain_selected_history();
        Ok(PerformedRuntimeWorldBootstrap::new(
            self.attempt.clone(),
            self.basis.clone(),
            self.observation.clone(),
            self.recovered,
        ))
    }
}
