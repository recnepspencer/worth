//! The registry reserves a real settlement vacancy, never a byte quote.
use super::*;
impl super::super::WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph::application_output_demand::registry) fn reserve_settlement_vacancy(
        &self,
        identity: &Arc<RecordedSettlementIdentity>,
    ) -> Result<(Posting, Box<PendingVacancyCleanup>), WorthQueryOutputDemandDenial> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.reserve_settlement_vacancy(identity)
    }
}
