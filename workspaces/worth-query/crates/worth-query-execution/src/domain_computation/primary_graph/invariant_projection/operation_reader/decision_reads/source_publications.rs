//! Original issued fact VALUES travel with this projection; keys carry no proof.
use super::*;
use crate::domain_computation::primary_graph::{
    application_attempt::SourcePublicationRead,
    output_lineage::invalidation::{
        arc_bytes, InvalidationEditAdmission, ObservedSourcePublication, SourceInvalidationOwner,
    },
};
use std::sync::Arc;
use worth_relational::facade::mvcc::CompanionPreflightStop as Stop;
type Value = (SourcePublicationRead, Arc<ObservedSourcePublication>);
pub(super) struct IssuedSourcePublicationValues {
 values:Vec<Value>,
 _capacity:Arc<crate::domain_computation::execution_runtime::source_invalidation::RetainedInvalidationCapacity>,
}
impl IssuedSourcePublicationValues {
    pub(super) fn get(
        &self,
        key: &SourcePublicationRead,
    ) -> Option<&Arc<ObservedSourcePublication>> {
        self.values
            .iter()
            .find(|(issued, _)| issued == key)
            .map(|(_, value)| value)
    }
}
impl DecisionReads {
    pub(in crate::domain_computation::primary_graph::invariant_projection::operation_reader) fn record_source_publication(
        &mut self,
        key: SourcePublicationRead,
        value: Arc<ObservedSourcePublication>,
        owner: &SourceInvalidationOwner,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), Stop> {
        let prior = self.source_publications.as_deref();
        let count = prior.map_or(0, |prior| prior.values.len());
        // Fund duplicate lookup/copy (2*n) and both eventual original-value
        // lookups (projection and closure). Across inserted prefixes the latter
        // sum(4*n+2) covers 2*N*N comparisons in the final value packet.
        // This is full work, never sparse or carried-read authority.
        let visits = u64::try_from(count)
            .ok()
            .and_then(|n| n.checked_mul(6))
            .and_then(|n| n.checked_add(2))
            .and_then(|n| n.checked_mul(8 + 4 * 32))
            .ok_or(Stop::WorkCounterOverflow)?;
        admission.charge_external_work(visits)?;
        if prior.is_none_or(|prior| prior.get(&key).is_none()) {
            let len = count
                .checked_add(1)
                .ok_or(Stop::PreparationMemoryCounterOverflow)?;
            let bytes = len
                .checked_mul(std::mem::size_of::<Value>())
                .and_then(|n| u64::try_from(n).ok())
                .and_then(|n| n.checked_add(arc_bytes::<IssuedSourcePublicationValues>()?))
                .ok_or(Stop::PreparationMemoryCounterOverflow)?;
            admission.admit_read_scratch(bytes)?;
            let capacity = owner.retain_consumed_output_backing(bytes, admission)?;
            let mut values = Vec::with_capacity(len);
            if let Some(prior) = prior {
                values.extend(prior.values.iter().cloned());
            }
            values.push((key.clone(), value));
            self.source_publications = Some(Arc::new(IssuedSourcePublicationValues {
                values,
                _capacity: capacity,
            }));
        }
        self.record(
            WorthQueryApplicationFactKey::SourcePublication(key),
            DecisionReadOutcome::Observed,
        );
        Ok(())
    }
}
