//! Check all producers ordinary selection can reach, before a group's or a recorded stage's first effect.
use super::*;
use crate::domain_computation::primary_graph::WorthQueryDocumentHistoryDenial;

impl<S: ApplicationSchema> WorthQueryInstalledApplicationProducerRegistry<S> {
    pub fn supports_history_group<Family>(&self) -> bool
    where
        Family: WorthQueryProducerOutputFamily<S>,
    {
        self.supports_history_group_family(Family::IDENTITY, TypeId::of::<Family>())
    }

    fn supports_history_group_family(&self, identity: &str, family: TypeId) -> bool {
        self.entries
            .values()
            .filter(|entry| {
                entry.declaration.output_family == identity
                    && entry.declaration.output_family_type == family
            })
            .all(|entry| entry.executor.history_refusal().is_none())
    }

    /// The first installed producer in the families of `producers` that
    /// document history cannot record.
    pub(in crate::domain_computation::primary_graph::application_contribution) fn history_refusal_among(
        &self,
        producers: &[super::super::super::authored_edit::ProducerDependencyScopes],
    ) -> Option<WorthQueryDocumentHistoryDenial> {
        self.entries
            .values()
            .filter(|entry| {
                producers.iter().any(|producer| {
                    entry.declaration.output_family == producer.family
                        && entry.declaration.output_family_type == TypeId::from(producer.family_type)
                })
            })
            .find_map(|entry| entry.executor.history_refusal())
    }
}
