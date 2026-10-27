//! Actual admitted operation scope is the only ordinary component issuance entrance.
use super::super::{
    WorthQueryInvariantProjectionComponent, WorthQueryInvariantProjectionTraversalDenial as Denial,
    WorthQueryInvariantProjectionTraversalDenialKind as Kind,
};
use super::*;
use crate::domain_computation::primary_graph::{
    aggregate_observation::{AggregateObservation, AggregateSelection},
    component_selection::{ComponentSelection, ComponentSelectionSubject},
    schema_layout::ComponentSetLayout,
    source_facts::SourceFacts,
};
use std::collections::BTreeSet;
use worth_query_declaration::facade::application_schema::{
    ApplicationComponentSetRef, ApplicationOperationMarkerIdentity,
};
use worth_query_installation::facade::{
    WorthQueryOperationComponentReadScope, WorthQueryOperationGraphReadScope,
};
use worth_relational::facade::{identity::EntityId, runtime::RelationalPinnedRead};
mod paths;
mod public_reads;
mod selection;

impl<Schema: ApplicationSchema, Operation: ApplicationOperationMarkerIdentity<Schema>>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>
{
    fn component_scope<Set, Entity>(
        &self,
        set: ApplicationComponentSetRef<Schema, Set, Entity>,
    ) -> Result<
        (
            WorthQueryOperationComponentReadScope,
            ComponentSetLayout,
            EntityId,
        ),
        Denial,
    > {
        let denied = || Denial::new(Kind::UndeclaredDecisionTarget, set.name());
        self.admission_identity.ok_or_else(denied)?;
        let root = self.admitted_root.as_ref().ok_or_else(denied)?;
        let contracts = self.admitted_contracts.as_ref().ok_or_else(denied)?;
        let layout = self.reader.component_layout(set)?;
        let scope = contracts
            .graph_reads()
            .roles()
            .iter()
            .flat_map(|role| role.read_scopes())
            .find_map(|scope| match scope {
                WorthQueryOperationGraphReadScope::Components(scope)
                    if scope.schema() == self.binding_identity
                        && scope.meaning().operation() == Operation::IDENTIFIER
                        && scope.meaning().set() == set.name()
                        && scope.meaning().member() == set.member()
                        && scope.meaning().root() == root.name.as_ref()
                        && self.reader.layout.entity_kind(&root.name) == Some(root.kind) =>
                {
                    Some(scope.clone())
                }
                _ => None,
            })
            .ok_or_else(denied)?;
        Ok((scope, layout, root.entity))
    }
    fn select_component_answer<Set: OperationReads<Operation>, Entity>(
        &mut self,
        set: ApplicationComponentSetRef<Schema, Set, Entity>,
        member: Option<&WorthQueryInvariantEntityIdentity<Schema, Entity>>,
    ) -> Result<
        (
            Vec<WorthQueryInvariantProjectionComponent<Schema, Set, Entity>>,
            Arc<ComponentSelection>,
        ),
        Denial,
    > {
        let (scope, layout, root) = self.component_scope(set)?;
        if member.is_some_and(|member| {
            !self.reader.identity_is_local(member, set.member())
                || member.kind != layout.member_kind
        }) {
            return Err(Denial::new(Kind::ForeignIdentity, set.name()));
        }
        let execution = *self.reader.execution;
        let result=execution.run_native_with_checkpoint(|context| {
            let pin=self.component_pin(set.name())?;
            if !layout.matches_native_definition(&pin,||AggregateObservation::checkpoint(context)).map_err(|cause|self.reader.walk_stop(cause,set.name()))? { return Err(Denial::new(Kind::AggregateDefinitionUnavailable,set.name())); }
            let (aggregates,facts)=self.collect_component_selection::<Entity>(&scope,&layout,root,member.map(|m|m.entity_id),&pin,context,set.name())?;
            let mut answer=Vec::new();
            answer.try_reserve_exact(aggregates.len()).map_err(|_|self.reader.walk_stop(worth_relational::facade::transactions::RelationalExecutionDenialCause::AllocationFailed,set.name()))?;
            for aggregate in &aggregates { answer.push(WorthQueryInvariantProjectionComponent::new(Arc::clone(aggregate),&layout,self.reader.binding_identity.clone(),self.reader.authority_identity)); }
            let body=ComponentSelection::new(scope,layout,root,member.map_or(ComponentSelectionSubject::Census,|m|ComponentSelectionSubject::Of(m.entity_id)),self.reader.snapshot.clone(),aggregates,facts,||AggregateObservation::checkpoint(context)).map_err(|cause|self.reader.walk_stop(cause,set.name()))?.ok_or_else(||Denial::new(Kind::ForeignIdentity,set.name()))?;
            Ok((answer,Arc::new(body)))
        });
        match result {
            Ok((answer, _)) => answer,
            Err(stop) => Err(self.reader.walk_stop(stop.into(), set.name())),
        }
    }
    fn component_pin(&mut self, subject: &str) -> Result<RelationalPinnedRead, Denial> {
        self.reader
            .runtime
            .read_truth()
            .pin_read_capability(self.reader.snapshot, &[], self.reader.execution)
            .map_err(|stop| match stop {
                worth_relational::facade::runtime::RelationalPinnedReadAdmissionDenial::Request(
                    stop,
                ) => self.reader.walk_stop(
                    worth_execution::WorkCeilingDenial::Stopped(stop).into(),
                    subject,
                ),
                _ => Denial::new(Kind::EndpointUnavailable, subject),
            })
    }
}
