//! Derived selections retain the genuine Member43 scope and Member46 layout.
use super::*;
use worth_query_installation::facade::{
    WorthQueryOperationDerivedSourceReadScope as Scope, WorthQueryOperationGraphReadScope,
};
use worth_relational::facade::{
    schema::NativeMemberKey as Key, transactions::RelationalExecutionDenialCause as Cause,
};
impl ComponentSelection {
    pub(in crate::domain_computation::primary_graph) fn from_derived(
        scope: Scope,
        layout: ComponentSetLayout,
        root: EntityId,
        member: Key,
        source: SnapshotHandle,
        aggregates: Vec<Arc<AggregateObservation<Key>>>,
        facts: Vec<SourceFacts>,
        mut poll: impl FnMut() -> Result<(), Cause>,
    ) -> Result<Option<Self>, Cause> {
        let Some(handle) = layout.source.as_ref() else {
            return Ok(None);
        };
        poll()?;
        crate::domain_computation::primary_graph::aggregate_observation::member_work::checkpoint_names(&member,&mut poll)?;
        for body in &aggregates {
            crate::domain_computation::primary_graph::aggregate_observation::member_work::checkpoint_names(body.key().member(),&mut poll)?;
        }
        if handle.parts().identity.name != scope.relation()
            || aggregates.len() != 1
            || aggregates.iter().any(|body| {
                body.source() != &source
                    || body.native().definition() != layout.definition
                    || body.key().member() != &member
                    || body.key().entity_kind() != layout.member_kind
            })
        {
            return Ok(None);
        }
        let key = ComponentSelectionKey::for_derived(&scope, layout.definition, root, member);
        let commitment = commitment::derive_members(&key, &aggregates, &facts, poll)?;
        Ok(Some(Self {
            key,
            origin: ComponentSelectionOrigin::Derived(scope),
            layout,
            source,
            aggregates: ComponentAggregateBodies::Derived(aggregates),
            facts,
            commitment,
        }))
    }
    pub(in crate::domain_computation::primary_graph) fn installed_scope_matches(
        &self,
        scope: &WorthQueryOperationGraphReadScope,
    ) -> bool {
        match (&self.origin, scope) {
            (
                ComponentSelectionOrigin::Operation(a),
                WorthQueryOperationGraphReadScope::Components(b),
            ) => a == b,
            (
                ComponentSelectionOrigin::Derived(a),
                WorthQueryOperationGraphReadScope::DerivedSource(b),
            ) => a == b,
            _ => false,
        }
    }
    pub(in crate::domain_computation::primary_graph) fn operation_binding(
        &self,
    ) -> Option<&worth_query_installation::facade::ApplicationSchemaBindingIdentity> {
        match &self.origin {
            ComponentSelectionOrigin::Operation(scope) => Some(scope.schema()),
            ComponentSelectionOrigin::Derived(scope) => Some(scope.schema()),
            _ => None,
        }
    }
}
