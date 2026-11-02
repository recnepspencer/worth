//! Check all producers ordinary selection can reach, before a group's first effect.
use super::*;

impl<S: ApplicationSchema> WorthQueryInstalledApplicationProducerRegistry<S> {
    pub(in crate::domain_computation::primary_graph) fn supports_history_group<Family>(
        &self,
    ) -> bool
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
            .all(|entry| entry.executor.supports_history_group())
    }
}
