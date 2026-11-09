//! Prepared creation resources cross selection without new owner calls.
use super::*;
impl ActiveAttemptCustody {
    pub(crate) fn install_durable_creation_cell(
        &mut self,
        reservation: &mut crate::branch::registry::ProductBranchRegistryReservation,
        permit: &mut crate::branch::WorldProductCreationPermit,
        source: &ProductBranchObservation,
        branch: crate::identity::ProductBranchIdentity,
        incarnation: crate::identity::ProductBranchIncarnation,
    ) {
        let mut lease = self.lease_resources();
        let creation = lease.resources_mut().creation.as_mut().expect("creation");
        let retired = reservation.install_durably_created(
            source,
            branch,
            incarnation,
            &mut creation.cell,
            permit,
        );
        drop(lease);
        drop(retired);
    }
    pub(crate) fn adopt_reused_creation_destination(
        &mut self,
        commit: Arc<crate::history::CompositeRuntimeWorldCommit>,
        cell: crate::branch::ProductBranchReferenceCell,
    ) {
        let mut lease = self.lease_resources();
        let r = lease.resources_mut();
        r.history_custody = ActiveHistoryCustody::Released;
        // Exact reuse keeps the original unused pin reservation until terminal.
        // The actual new head was issued independently for the shared commit.
        r.commit = Some(commit);
        let c = r.creation.as_mut().expect("creation");
        assert!(c.observation.is_some(), "the original response is prepared");
        c.cell = Some(cell);
    }
    pub(crate) fn durable_creation_commit(
        &mut self,
    ) -> Option<Arc<crate::history::CompositeRuntimeWorldCommit>> {
        self.lease_resources().resources_mut().commit.clone()
    }
    pub(crate) fn creation_checkpoint_observation(&mut self) -> ProductBranchObservation {
        self.lease_resources()
            .resources_mut()
            .creation
            .as_ref()
            .expect("creation")
            .observation
            .as_ref()
            .expect("prepared observation")
            .clone()
    }
    pub(crate) fn creation_prepared(&mut self) -> bool {
        self.lease_resources()
            .resources_mut()
            .creation
            .as_ref()
            .is_some_and(|c| c.cell.is_some())
    }
}
