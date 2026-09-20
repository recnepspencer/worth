use super::{
    WorthQueryObservedSource, WorthQuerySourceExpectationDenial,
    WorthQuerySourceExpectationDenialKind,
};

impl<Query> WorthQueryObservedSource<Query> {
    pub(in crate::domain_computation::primary_graph) fn retained_checkpoint_facts(
        &self,
        layout: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    ) -> Result<
        std::sync::Arc<
            [crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact],
        >,
        WorthQuerySourceExpectationDenial,
    > {
        use crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact as Fact;
        use WorthQuerySourceExpectationDenialKind as Kind;

        self.validate_completeness(&self.query_identifier)?;
        let mut facts = Vec::with_capacity(
            self.footprint
                .entities
                .len()
                .saturating_add(self.footprint.aspects.len())
                .saturating_add(self.footprint.adjacencies.len()),
        );
        facts.extend(
            self.footprint
                .entities
                .iter()
                .copied()
                .map(|entity_id| Fact::SourceEntity { entity_id }),
        );
        for aspect in &self.footprint.aspects {
            layout
                .aspect_contract(&aspect.entity_name, &aspect.aspect)
                .filter(|contract| contract.revision() == aspect.contract_revision)
                .ok_or_else(|| {
                    WorthQuerySourceExpectationDenial::new(
                        Kind::SourceContractMismatch,
                        aspect.aspect.as_str(),
                    )
                })?;
            facts.push(Fact::SourceAspectRevision {
                entity_id: aspect.entity,
                aspect: aspect.aspect.clone(),
                native_revision: aspect.native_revision,
            });
        }
        facts.extend(self.footprint.adjacencies.iter().map(|adjacency| {
            Fact::SourceAdjacencyRevision {
                relation_kind: adjacency.relation_kind,
                anchor: adjacency.anchor,
                direction: adjacency.direction,
                native_revision: adjacency.native_revision,
                comparison_work_limit: adjacency.comparison_work_limit,
                endpoints: adjacency.endpoints.clone(),
            }
        }));
        Ok(facts.into())
    }
}
