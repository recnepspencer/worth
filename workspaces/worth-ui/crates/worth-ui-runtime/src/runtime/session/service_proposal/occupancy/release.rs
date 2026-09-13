use super::super::UiServiceProposalIdentity;
use super::{
    UiServiceProposalOccupancyDenial, UiServiceProposalOccupancyLease,
    UiServiceProposalOccupancyTable,
};

impl UiServiceProposalOccupancyTable {
    pub(in crate::runtime::session::service_proposal) fn can_release(
        &self,
        proposal: UiServiceProposalIdentity,
        leases: &[UiServiceProposalOccupancyLease],
    ) -> Result<(), UiServiceProposalOccupancyDenial> {
        let Some(first) = leases.first() else {
            return Err(UiServiceProposalOccupancyDenial::AmbiguousConflict);
        };
        if leases.iter().any(|lease| {
            lease.key.application != first.key.application
                || lease.key.semantic_surface != first.key.semantic_surface
        }) {
            return Err(UiServiceProposalOccupancyDenial::AmbiguousConflict);
        }
        let neighborhood = self
            .neighborhoods
            .find(&first.key.application, first.key.semantic_surface);
        if leases.iter().any(|lease| {
            lease.proposal != proposal
                || !neighborhood
                    .into_iter()
                    .flat_map(|neighborhood| &neighborhood.records)
                    .any(|record| {
                        record.proposal == proposal
                            && record.key == lease.key
                            && record.slot_generation == lease.slot_generation
                    })
        }) {
            return Err(UiServiceProposalOccupancyDenial::AmbiguousConflict);
        }
        Ok(())
    }

    pub(in crate::runtime::session::service_proposal) fn close_before_effect_window(
        &mut self,
        proposal: UiServiceProposalIdentity,
        leases: &[UiServiceProposalOccupancyLease],
    ) -> Result<(), UiServiceProposalOccupancyDenial> {
        self.can_release(proposal, leases)?;
        let first = leases
            .first()
            .expect("validated service proposal owns at least one lease");
        for record in self
            .neighborhoods
            .find_mut(&first.key.application, first.key.semantic_surface)
            .expect("validated service proposal neighborhood remains live")
            .records
            .iter_mut()
            .filter(|record| record.proposal == proposal)
        {
            record.before_effect_open = false;
        }
        Ok(())
    }

    pub(in crate::runtime::session::service_proposal) fn release(
        &mut self,
        proposal: UiServiceProposalIdentity,
        leases: &[UiServiceProposalOccupancyLease],
    ) -> u16 {
        let first = leases
            .first()
            .expect("a reserved service proposal owns at least one lease");
        self.neighborhoods
            .release(&first.key.application, first.key.semantic_surface, proposal)
    }
}
