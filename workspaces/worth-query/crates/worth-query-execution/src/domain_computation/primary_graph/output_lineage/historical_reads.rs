//! Retain descriptive predecessor metadata from the exact existing published row.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::producer::predecessor_reads::WorthQueryHistoricalProducerRead;
impl WorthQueryApplicationOutputLineage {
    pub(in crate::domain_computation::primary_graph) fn retain_predecessor_reads(
        &self,
        identity: &Arc<RecordedSettlementIdentity>,
    ) -> Option<Arc<[WorthQueryHistoricalProducerRead]>> {
        let coordinate = identity.coordinate();
        let row = self
            .by_source
            .get(identity.source())?
            .get(&coordinate.occurrence)?
            .get(&coordinate.generation)?
            .get(identity.slot())?
            .get()?;
        if !Arc::ptr_eq(&row.settlement_identity, identity) {
            return None;
        }
        let origin = match &row.performed_origin {
            Some(origin) => origin.get()?,
            None => row,
        };
        origin
            .completed_handler_facts
            .as_ref()?
            .retain_predecessor_reads()
    }
}
