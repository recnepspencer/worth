//! Borrow the exact successor and predecessor while caller custody is live.

use super::*;

impl<Schema> RequiredFreshProgress<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    pub(in crate::domain_computation::primary_graph) fn interest(
        &self,
    ) -> &WorthQueryOutputDemandInterest {
        self.successor.interest()
    }

    pub(in crate::domain_computation::primary_graph) fn producer_identity(&self) -> &str {
        self.successor.producer_identity()
    }

    pub(in crate::domain_computation::primary_graph) fn producer_contacts(&self) -> usize {
        self.successor.producer_contacts()
    }

    pub(in crate::domain_computation::primary_graph) fn advance_checkpoint(
        &mut self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        request: &WorthQueryRequestScope,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        self.successor
            .advance_checkpoint(runtime, request, admission)
    }

    pub(in crate::domain_computation::primary_graph) fn is_family<Family>(&self) -> bool
    where
        Family: WorthQueryProducerOutputFamily<Schema> + 'static,
    {
        self.successor.family_type() == TypeId::of::<Family>()
    }

    pub(in crate::domain_computation::primary_graph) fn predecessor(
        &self,
    ) -> &SelectedReadyReadmission {
        self.successor.predecessor()
    }

    pub(super) fn take_outcome(&mut self) -> RequiredFreshOutcome {
        self.outcome
            .take()
            .expect("required successor outcome is taken only after custody install")
    }
}
