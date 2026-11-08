//! Exact registered preparation transfers once into its publication owner.
use super::*;
impl WorthQueryPrimaryGraphApplicationAttempt {
    pub(in crate::domain_computation::primary_graph) fn retain_coupled_after_own(
        &mut self,
        pin: worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
    ) {
        let preparation = self
            .coupled_preparation
            .as_mut()
            .expect("actual writer preparation");
        preparation.retain_after_own(pin);
    }
    pub(in crate::domain_computation::primary_graph) fn retain_coupled_preparation(
        &mut self,
        preparation: crate::domain_computation::primary_graph::provider::CoupledProducerPreparation,
    ) {
        assert!(
            self.coupled_preparation.replace(preparation).is_none(),
            "each actual writer transfers its preparation once"
        );
    }
    pub(in crate::domain_computation::primary_graph) fn coupled_preparation(
        &self,
    ) -> Option<&crate::domain_computation::primary_graph::provider::CoupledProducerPreparation>
    {
        self.coupled_preparation.as_ref()
    }
    pub(in crate::domain_computation::primary_graph) fn take_coupled_preparation(
        &mut self,
    ) -> Option<crate::domain_computation::primary_graph::provider::CoupledProducerPreparation>
    {
        self.coupled_preparation.take()
    }
}

impl WorthQueryPrimaryGraphApplicationAttempt {
    pub(in crate::domain_computation::primary_graph) fn admitted_emissions(
        &self,
    ) -> &super::super::super::WorthQueryAdmittedApplicationEmissionBatch {
        self.effects.emissions()
    }
    pub(in crate::domain_computation::primary_graph) fn reserve_coupled_dispatch(
        &mut self,
        provider: &crate::domain_computation::primary_graph::provider::WorthQueryPrimaryGraphProvider,
    ) -> Result<
        (),
        crate::domain_computation::primary_graph::live_delivery::WorthQueryLiveAdmissionDenial,
    > {
        assert!(self.outstanding_dispatch_reservation.is_none());
        self.outstanding_dispatch_reservation = provider.reserve_outstanding_dispatch(
            self.dispatch_outbox
                .as_ref()
                .map(|pending| pending.record()),
            self.affinity.product_publication().observation(),
        )?;
        Ok(())
    }
}
