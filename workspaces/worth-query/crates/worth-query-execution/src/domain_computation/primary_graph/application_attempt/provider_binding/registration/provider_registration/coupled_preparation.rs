//! Exact registered preparation transfers once into its publication owner.
use super::*;
use crate::domain_computation::primary_graph::provider::CoupledAttemptPreparation;
impl WorthQueryPrimaryGraphApplicationAttempt {
    pub(in crate::domain_computation::primary_graph) fn coupled_after_own(
        &self,
    ) -> Option<
        &worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
    > {
        match self.coupled_preparation.as_ref()? {
            CoupledAttemptPreparation::Producer(preparation) => preparation.after_own.as_ref(),
            CoupledAttemptPreparation::AuthoredStage(pin) => Some(pin),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn retain_authored_stage_after_own(
        &mut self,
        pin: worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
    ) {
        assert!(
            self.coupled_preparation
                .replace(CoupledAttemptPreparation::AuthoredStage(pin))
                .is_none(),
            "a stage retains its own completed candidate exactly once"
        );
    }
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
        match preparation {
            CoupledAttemptPreparation::Producer(preparation) => preparation.retain_after_own(pin),
            CoupledAttemptPreparation::AuthoredStage(_) => {
                unreachable!("producer dispatch owns its producer preparation")
            }
        }
    }
    pub(in crate::domain_computation::primary_graph) fn retain_coupled_preparation(
        &mut self,
        preparation: crate::domain_computation::primary_graph::provider::CoupledProducerPreparation,
    ) {
        assert!(
            self.coupled_preparation
                .replace(CoupledAttemptPreparation::Producer(preparation))
                .is_none(),
            "each actual writer transfers its preparation once"
        );
    }
    pub(in crate::domain_computation::primary_graph) fn coupled_preparation(
        &self,
    ) -> Option<&crate::domain_computation::primary_graph::provider::CoupledProducerPreparation>
    {
        match self.coupled_preparation.as_ref()? {
            CoupledAttemptPreparation::Producer(preparation) => Some(preparation),
            CoupledAttemptPreparation::AuthoredStage(_) => None,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn take_coupled_preparation(
        &mut self,
    ) -> Option<crate::domain_computation::primary_graph::provider::CoupledProducerPreparation>
    {
        match self.coupled_preparation.take()? {
            CoupledAttemptPreparation::Producer(preparation) => Some(preparation),
            CoupledAttemptPreparation::AuthoredStage(_) => None,
        }
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
