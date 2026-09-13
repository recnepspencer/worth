use super::{ActiveAttemptCustody, ActiveHistoryCustody};
use crate::branch::registry::{
    ProductBranchInstallationWitness, ProductBranchRegistryReservation,
    ProductBranchSourceInstallFailure,
};
use crate::branch::{
    ProductBranchObservation, ProductBranchReferenceCell, ProductBranchReferenceSnapshot,
};
use crate::history::CompositeRuntimeWorldCommit;
use crate::retention::ReservedObservationCapacity;
use std::sync::Arc;

/// Creation-only custody carried in the same owner record as its fork evidence.
#[derive(Debug)]
pub(super) struct ActiveCreationResources {
    pub(super) observation_capacity: Option<ReservedObservationCapacity>,
    pub(super) observation: Option<ProductBranchObservation>,
    pub(super) cell: Option<ProductBranchReferenceCell>,
}

impl ActiveAttemptCustody {
    pub(crate) fn configure_creation(&mut self, observation_capacity: ReservedObservationCapacity) {
        let mut lease = self.lease_resources();
        let resources = lease.resources_mut();
        assert!(resources.creation.is_none());
        resources.creation = Some(ActiveCreationResources {
            observation_capacity: Some(observation_capacity),
            observation: None,
            cell: None,
        });
    }

    pub(crate) fn bind_creation_destination(
        &mut self,
        witness: Arc<ProductBranchInstallationWitness>,
    ) {
        let mut state = self.record.state();
        assert_eq!(
            witness.destination().0.owner_identity(),
            self.record.identity().owner_identity()
        );
        assert_eq!(state.progress.owner_effect_count(), 0);
        assert!(state.destination.is_none());
        state.destination = Some(witness);
    }

    pub(crate) fn install_creation_history(&mut self, commit: &Arc<CompositeRuntimeWorldCommit>) {
        let mut lease = self.lease_resources();
        let resources = lease.resources_mut();
        let ActiveHistoryCustody::Reserved(capacity) = &mut resources.history_custody else {
            panic!("creation holds reserved history")
        };
        let protection = capacity
            .try_install_product_head(
                Arc::clone(commit),
                resources
                    .history_pins
                    .take()
                    .expect("creation bound exact history pins"),
            )
            .expect("the admitted creation installs with immediate protection");
        resources.history_custody = ActiveHistoryCustody::Installed(protection);
    }

    pub(crate) fn prepare_creation_cell(&mut self, snapshot: ProductBranchReferenceSnapshot) {
        let mut lease = self.lease_resources();
        lease.assemble_head(snapshot);
        let resources = lease.resources_mut();
        let head = resources
            .product_head
            .take()
            .expect("head assembly completed");
        match ProductBranchReferenceCell::new(head) {
            Ok(cell) => resources.creation.as_mut().expect("creation custody").cell = Some(cell),
            Err(failure) => {
                resources.product_head = Some(failure.into_protection());
                panic!("the admitted creation head binds its cell");
            }
        }
    }

    /// Component calls run with an exclusive resource lease and no World lock.
    pub(crate) fn issue_creation_observation<D, I, T>(
        &mut self,
        retention: &crate::retention::RuntimeWorldRetentionOwner<D, I, T>,
        history: &crate::history::CompositeHistoryCatalog,
        snapshot: ProductBranchReferenceSnapshot,
    ) -> Result<(), crate::recovery::ProductUnpublishedCause>
    where
        D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
        I: Copy + Ord + Send + Sync + 'static,
        T: Copy + Ord + Send + Sync + 'static,
    {
        #[cfg(test)]
        let recovery_identity = self.record.identity().clone();
        let mut lease = self.lease_resources();
        let creation = lease
            .resources_mut()
            .creation
            .as_mut()
            .expect("creation custody");
        let capacity = creation
            .observation_capacity
            .take()
            .expect("creation reserved its observation before forks");
        let components = retention
            .issue_reserved_observation(snapshot.commit(), capacity)
            .map_err(|denial| {
                crate::recovery::ProductUnpublishedCause::from_retention_denial(&denial)
            })?;
        let history = history
            .protect_explicit_commit(snapshot.commit())
            .map_err(|_| crate::recovery::ProductUnpublishedCause::DestinationAdmissionDenied)?;
        let observation = ProductBranchObservation::owner_issued(snapshot, components, history)
            .map_err(|_| crate::recovery::ProductUnpublishedCause::DestinationAdmissionDenied)?;
        #[cfg(test)]
        let observation =
            super::creation_rehearsal::withhold_observation_authority_under_rehearsal(
                &recovery_identity,
                Ok(observation),
            )?;
        creation.observation = Some(observation);
        Ok(())
    }

    pub(crate) fn install_creation_cell(
        &mut self,
        reservation: ProductBranchRegistryReservation,
        cancellation: &crate::publication::RuntimeWorldCancellationToken,
    ) -> Result<ProductBranchObservation, ProductBranchSourceInstallFailure> {
        let source = self.record.expected.clone();
        let witness = self
            .record
            .state()
            .destination
            .as_ref()
            .cloned()
            .expect("creation bound its destination before forks");
        let (branch, incarnation) = witness.destination();
        let mut lease = self.lease_resources();
        let creation = lease
            .resources_mut()
            .creation
            .as_mut()
            .expect("creation custody");
        reservation.install_from_source(
            &source,
            branch.clone(),
            incarnation,
            &mut creation.cell,
            cancellation,
        )?;
        assert!(
            witness.installed_commit().is_some(),
            "the registry records success before releasing its guard"
        );
        Ok(creation
            .observation
            .take()
            .expect("the installed creation returns its admitted observation"))
    }

    /// Explicit retention releases prospective observation authority and puts
    /// any refused cell's original proof back into the common custody lane.
    pub(super) fn retain_creation_resources(&mut self) {
        let mut lease = self.lease_resources();
        let resources = lease.resources_mut();
        let Some(creation) = resources.creation.as_mut() else {
            return;
        };
        drop(creation.observation.take());
        drop(creation.observation_capacity.take());
        if let Some(cell) = creation.cell.take() {
            resources.product_head = Some(
                cell.into_protection()
                    .expect("a refused destination has no other cell holder"),
            );
        }
    }
}
