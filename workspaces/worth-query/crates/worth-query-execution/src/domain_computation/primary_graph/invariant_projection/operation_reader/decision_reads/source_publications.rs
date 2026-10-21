//! Original issued fact VALUES travel with this projection; keys carry no proof.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::SourcePublicationRead;

use crate::domain_computation::primary_graph::output_lineage::invalidation::ObservedSourcePublication;
use std::sync::Arc;
use worth_relational::facade::mvcc::CompanionPreflightStop as Stop;
type Value = (SourcePublicationRead, Arc<ObservedSourcePublication>);
pub(super) struct IssuedSourcePublicationValues {
    values: Vec<Value>,
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
    ) -> Result<(), Stop> {
        let prior = self.source_publications.as_deref();
        let count = prior.map_or(0, |prior| prior.values.len());
        if prior.is_none_or(|prior| prior.get(&key).is_none()) {
            let len = count
                .checked_add(1)
                .ok_or(Stop::PreparationMemoryCounterOverflow)?;
            let mut values = Vec::with_capacity(len);
            if let Some(prior) = prior {
                values.extend(prior.values.iter().cloned());
            }
            values.push((key.clone(), value));
            self.source_publications = Some(Arc::new(IssuedSourcePublicationValues { values }));
        }
        self.record(
            WorthQueryApplicationFactKey::SourcePublication(key),
            DecisionReadOutcome::Observed,
        );
        Ok(())
    }
}
