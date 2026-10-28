//! The actual installed origin describes an observation; admission remains at each caller.
use super::*;
use worth_query_installation::facade::{
    ApplicationSchemaBindingIdentity, WorthQueryInstalledGraphReadContract,
};
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum ComponentSelectionOrigin {
    Operation(WorthQueryOperationComponentReadScope),
    Derived(worth_query_installation::facade::WorthQueryOperationDerivedSourceReadScope),
    Query {
        binding: ApplicationSchemaBindingIdentity,
        graph: worth_foundational::facade::CanonicalDigestId,
    },
}
impl ComponentSelection {
    pub(in crate::domain_computation::primary_graph) fn from_query(
        installed: &WorthQueryInstalledGraphReadContract,
        binding: &ApplicationSchemaBindingIdentity,
        graph: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
        root: EntityId,
        source: SnapshotHandle,
        subject: ComponentSelectionSubject,
        aggregates: Vec<Arc<AggregateObservation>>,
        facts: Vec<SourceFacts>,
        allocation: Arc<super::super::source_facts::SourceFactsAllocation>,
        poll: impl FnMut() -> Result<
            (),
            worth_relational::facade::transactions::RelationalExecutionDenialCause,
        >,
    ) -> Result<Option<Self>, worth_relational::facade::transactions::RelationalExecutionDenialCause>
    {
        let Some(meaning) = installed.component_selection() else {
            return Ok(None);
        };
        let Some(layout) = graph.component_set(meaning.set()) else {
            return Ok(None);
        };
        if meaning.member() != layout.member.as_ref() {
            return Ok(None);
        }
        let origin = ComponentSelectionOrigin::Query {
            binding: binding.clone(),
            graph: *installed.digest(),
        };
        let key = ComponentSelectionKey::for_query(installed, layout.definition, root, subject);
        Self::from_parts(
            origin,
            key,
            layout.clone(),
            source,
            aggregates,
            facts,
            allocation,
            poll,
        )
    }
    pub(in crate::domain_computation::primary_graph) fn has_query_origin(
        &self,
        binding: &ApplicationSchemaBindingIdentity,
    ) -> bool {
        matches!(&self.origin,ComponentSelectionOrigin::Query{binding:installed,..} if installed==binding)
    }
}
