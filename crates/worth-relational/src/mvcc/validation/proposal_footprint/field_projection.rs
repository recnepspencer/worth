//! Demand for field projection is selected on a live validated proposal.
use super::ValidatedMutationFootprintProjection;
use crate::mvcc::validation::validated_proposal::ValidatedRelationalProposal;

impl ValidatedRelationalProposal {
    /// Projects field mutation truth only when the consumer supplies demand.
    pub fn mutation_footprint<Demand: ?Sized>(
        &self,
        demand: Option<&Demand>,
    ) -> ValidatedMutationFootprintProjection {
        ValidatedMutationFootprintProjection::from_plan(
            demand.map(|_| self.state.prepared.plan.view()),
        )
    }
}
