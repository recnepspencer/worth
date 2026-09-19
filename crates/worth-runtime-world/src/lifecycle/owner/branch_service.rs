use std::sync::Arc;

use crate::branch::{
    ProductBranchHeadProtection, ProductBranchName, ProductBranchObservation,
    ProductBranchReferenceCell, ProductBranchReferenceSnapshot, ProductBranchRetirementReport,
    RuntimeWorldBranchAdmissionDenial, RuntimeWorldBranchRetirementDenial,
};
use crate::identity::{
    ProductBranchIdentity, ProductBranchIncarnation, ProductBranchReferenceGeneration,
};

use super::super::ports::{RuntimeWorldBranchCreationOutcome, RuntimeWorldBranchCreationRequest};
use super::RuntimeWorldOwnerRoot;

mod creation;
mod history;
#[cfg(test)]
mod install_control;
mod observation;
mod retirement;

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    fn branch_service_is_available(&self) -> bool {
        if !self.owner_is_present() {
            return false;
        }
        let bootstrap = self
            .state
            .bootstrap
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if *bootstrap != super::RuntimeWorldBootstrapState::Performed {
            return false;
        }
        self.state
            .close
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .state()
            == super::super::close::RuntimeWorldCloseState::Open
    }

    /// The branch identity is the owner plus the normalized name, so retiring
    /// and recreating one name yields the same identity with a new
    /// incarnation. Only the incarnation is drawn from the issuer.
    pub(super) fn issue_branch_identities(
        &self,
        name: ProductBranchName,
    ) -> Result<(ProductBranchIdentity, ProductBranchIncarnation), ()> {
        let mut identities = self
            .state
            .identities
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let incarnation = identities.branch_incarnation().map_err(|_| ())?;
        Ok((
            ProductBranchIdentity::issued(self.owner_identity(), name),
            incarnation,
        ))
    }

    /// Name a source head's admission before any charge. A source whose
    /// branch holds no occurrence is `RetiredBranch`, the answer
    /// `observe_product_branch` gives for the same branch; a source whose
    /// branch carries another head, or another occurrence of the same name,
    /// is `StaleSourceHead`. Every creation route names a source this way.
    pub(super) fn admit_source_head(
        &self,
        expected: &ProductBranchObservation,
    ) -> Result<(), RuntimeWorldBranchAdmissionDenial> {
        let cell = self
            .state
            .branches
            .branch_cell(expected.branch_identity())
            .ok_or(RuntimeWorldBranchAdmissionDenial::RetiredBranch)?;
        if expected
            .mismatch_against_snapshot(&cell.atomic_snapshot())
            .is_some()
        {
            return Err(RuntimeWorldBranchAdmissionDenial::StaleSourceHead);
        }
        Ok(())
    }

    fn create_reused_branch(
        &self,
        source: ProductBranchObservation,
        name: ProductBranchName,
        cancellation: &crate::publication::RuntimeWorldCancellationToken,
    ) -> Result<ProductBranchObservation, RuntimeWorldBranchAdmissionDenial> {
        // The reference cell is the only authority for a head. A source the
        // cell has moved past, or no longer holds, is refused here before any
        // capacity is charged, and refused again under the cell's own guard
        // at installation, so a publication between the two cannot install a
        // child from a head that is no longer current. A current source is
        // reused as the exact commit the caller observed, which that
        // observation keeps alive in history.
        self.admit_source_head(&source)?;
        let observation_capacity = self
            .state
            .retention
            .reserve_observation()
            .map_err(map_retention_denial)?;
        let commit = source.snapshot().shared_commit();
        // Reuse moves no owner, but it does install a product reference, and
        // close drains those. Holding an operation reservation across the
        // installation is what makes close wait for it instead of draining
        // underneath it.
        let operation = self
            .reserve_creation_operation()
            .map_err(|()| RuntimeWorldBranchAdmissionDenial::OwnerUnavailable)?;

        let reservation = self
            .state
            .branches
            .reserve_branch(self.owner_identity(), name.clone())
            .map_err(map_registry_denial)?;
        let (branch, lifecycle) = self
            .issue_branch_identities(name)
            .map_err(|_| RuntimeWorldBranchAdmissionDenial::IdentityExhausted)?;
        let (cell, snapshot) =
            self.issue_reused_head(branch.clone(), lifecycle, Arc::clone(&commit))?;
        let observation = self.issue_reused_observation(snapshot, commit, observation_capacity)?;

        #[cfg(test)]
        install_control::pause_before_source_guarded_install(self.owner_identity());
        reservation
            .install_from_source(&source, branch, lifecycle, &mut Some(cell), cancellation)
            .map_err(|failure| map_source_install_denial(failure.denial))?;
        drop(operation);
        Ok(observation)
    }

    fn issue_reused_head(
        &self,
        branch: ProductBranchIdentity,
        lifecycle: ProductBranchIncarnation,
        commit: Arc<crate::history::CompositeRuntimeWorldCommit>,
    ) -> Result<
        (ProductBranchReferenceCell, ProductBranchReferenceSnapshot),
        RuntimeWorldBranchAdmissionDenial,
    > {
        let product_head = self
            .state
            .retention
            .issue_product_head(commit.basis())
            .map_err(map_retention_denial)?;
        let product_history = self
            .state
            .history
            .protect_product_head(commit.as_ref())
            .map_err(|_| RuntimeWorldBranchAdmissionDenial::CapacityExhausted)?;
        let snapshot = ProductBranchReferenceSnapshot::owner_issued(
            self.owner_identity(),
            branch.clone(),
            lifecycle,
            ProductBranchReferenceGeneration::initial(),
            Arc::clone(&commit),
        )
        .map_err(|_| RuntimeWorldBranchAdmissionDenial::OwnerUnavailable)?;
        let protection = ProductBranchHeadProtection::bootstrap_issued(
            snapshot.clone(),
            product_head,
            product_history,
        )
        .map_err(|_| RuntimeWorldBranchAdmissionDenial::OwnerUnavailable)?;
        let cell = ProductBranchReferenceCell::new(protection)
            .map_err(|_| RuntimeWorldBranchAdmissionDenial::OwnerUnavailable)?;
        Ok((cell, snapshot))
    }

    fn issue_reused_observation(
        &self,
        snapshot: ProductBranchReferenceSnapshot,
        commit: Arc<crate::history::CompositeRuntimeWorldCommit>,
        observation_capacity: crate::retention::ReservedObservationCapacity,
    ) -> Result<ProductBranchObservation, RuntimeWorldBranchAdmissionDenial> {
        let observation_components = self
            .state
            .retention
            .issue_reserved_observation(commit.as_ref(), observation_capacity)
            .map_err(map_retention_denial)?;
        let observation_history = self
            .state
            .history
            .protect_explicit_commit(commit.as_ref())
            .map_err(|_| RuntimeWorldBranchAdmissionDenial::CapacityExhausted)?;
        let observation = ProductBranchObservation::owner_issued(
            snapshot,
            observation_components,
            observation_history,
        )
        .map_err(|_| RuntimeWorldBranchAdmissionDenial::OwnerUnavailable)?;
        Ok(observation)
    }
}

impl<D, I, E, Ctx, T> super::super::ports::RuntimeWorldBranchService
    for RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    fn create_product_branch(
        &self,
        request: RuntimeWorldBranchCreationRequest<'_>,
    ) -> Result<RuntimeWorldBranchCreationOutcome, RuntimeWorldBranchAdmissionDenial> {
        let (source, intent, cancellation) = request.into_parts();
        if source.owner_identity() != self.owner_identity()
            || source.branch_identity().owner_identity() != self.owner_identity()
        {
            return Err(RuntimeWorldBranchAdmissionDenial::ForeignOwner);
        }
        if !self.branch_service_is_available() {
            return Err(RuntimeWorldBranchAdmissionDenial::OwnerUnavailable);
        }
        let plans = intent
            .plans()
            .ok_or(RuntimeWorldBranchAdmissionDenial::PlansOmitted)?;
        if cancellation.is_cancelled() {
            return Err(RuntimeWorldBranchAdmissionDenial::CancelledBeforeEffect);
        }
        if plans.is_exact_reuse() {
            return self
                .create_reused_branch(source, intent.name().clone(), cancellation)
                .map(RuntimeWorldBranchCreationOutcome::Performed);
        }
        creation::create_forked_branch(self, source, intent, cancellation)
    }

    fn retire_product_branch(
        &self,
        observed: &ProductBranchObservation,
    ) -> Result<ProductBranchRetirementReport, RuntimeWorldBranchRetirementDenial> {
        self.retire_observed_branch(observed)
    }
}

fn map_registry_denial(
    denial: crate::branch::registry::ProductBranchRegistryDenial,
) -> RuntimeWorldBranchAdmissionDenial {
    use crate::branch::registry::ProductBranchRegistryDenial;

    match denial {
        ProductBranchRegistryDenial::ForeignOwner => {
            RuntimeWorldBranchAdmissionDenial::ForeignOwner
        }
        ProductBranchRegistryDenial::CapacityExhausted => {
            RuntimeWorldBranchAdmissionDenial::CapacityExhausted
        }
        ProductBranchRegistryDenial::AlreadyInstalled
        | ProductBranchRegistryDenial::AlreadyRetired
        | ProductBranchRegistryDenial::ReservationMissing
        | ProductBranchRegistryDenial::IdentityMismatch
        | ProductBranchRegistryDenial::BranchAlreadyInstalled
        | ProductBranchRegistryDenial::LifecycleAlreadyInstalled => {
            RuntimeWorldBranchAdmissionDenial::OwnerUnavailable
        }
        ProductBranchRegistryDenial::NameAlreadyReserved
        | ProductBranchRegistryDenial::NameAlreadyInstalled => {
            RuntimeWorldBranchAdmissionDenial::DuplicateName
        }
    }
}

fn map_source_install_denial(
    denial: crate::branch::registry::ProductBranchSourceInstallDenial,
) -> RuntimeWorldBranchAdmissionDenial {
    use crate::branch::registry::ProductBranchSourceInstallDenial;

    match denial {
        ProductBranchSourceInstallDenial::Cancelled => {
            RuntimeWorldBranchAdmissionDenial::CancelledBeforeEffect
        }
        ProductBranchSourceInstallDenial::Registry(denial) => map_registry_denial(denial),
        ProductBranchSourceInstallDenial::SourceRetired => {
            RuntimeWorldBranchAdmissionDenial::RetiredBranch
        }
        ProductBranchSourceInstallDenial::SourceDisplaced(_) => {
            RuntimeWorldBranchAdmissionDenial::StaleSourceHead
        }
    }
}

fn map_retention_denial(
    denial: crate::retention::RetentionObligationDenial,
) -> RuntimeWorldBranchAdmissionDenial {
    use crate::retention::RetentionObligationDenial;

    match denial {
        RetentionObligationDenial::LeaseIdentityExhausted => {
            RuntimeWorldBranchAdmissionDenial::IdentityExhausted
        }
        RetentionObligationDenial::ForeignOwner { .. }
        | RetentionObligationDenial::Relational(_)
        | RetentionObligationDenial::Signal(_)
        | RetentionObligationDenial::OwnerOperationPanicked => {
            RuntimeWorldBranchAdmissionDenial::OwnerUnavailable
        }
        RetentionObligationDenial::ObservationCapacityExhausted
        | RetentionObligationDenial::HistoryDependency(_)
        | RetentionObligationDenial::InvalidComponentPair
        | RetentionObligationDenial::UniquePinCapacityExhausted { .. }
        | RetentionObligationDenial::InFlightAcquisitionCapacityExhausted { .. }
        | RetentionObligationDenial::DependencyCountExhausted => {
            RuntimeWorldBranchAdmissionDenial::CapacityExhausted
        }
    }
}

fn map_observation_denial(
    denial: crate::branch::ProductBranchReferenceObservationFailure,
) -> RuntimeWorldBranchAdmissionDenial {
    use crate::branch::ProductBranchReferenceObservationFailure;

    match denial {
        ProductBranchReferenceObservationFailure::Retired => {
            RuntimeWorldBranchAdmissionDenial::RetiredBranch
        }
        ProductBranchReferenceObservationFailure::HistoryProtection(_)
        | ProductBranchReferenceObservationFailure::Retention(_)
        | ProductBranchReferenceObservationFailure::ObservationBinding(_) => {
            RuntimeWorldBranchAdmissionDenial::CapacityExhausted
        }
    }
}

#[cfg(test)]
#[path = "../../branch/retirement_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "branch_service_contract_tests.rs"]
mod contract_tests;
