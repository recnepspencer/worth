//! A decision retains source support and the actual persistent Native cohort.
use super::*;
use crate::domain_computation::primary_graph::{
    aggregate_identity::WorthQueryAggregateIdentity as Identity,
    aggregate_observation::AggregateObservation,
    component_selection::ComponentSelection,
    source_facts::{SourceFactReadDenial, SourceFactsBuilder},
};
use worth_query_declaration::facade::application_schema::ApplicationComponentSetRef;
use worth_query_installation::facade::WorthQueryOperationGraphReadScope;
use worth_relational::facade::runtime::RelationalAdjacencyDirection;

/// Read-only logical cohort; it never supplies a writable Entity target.
pub struct WorthQueryInvariantGraphComponent<S, Set, M> {
    members: Vec<WorthQueryInvariantGraphMember<S, M>>,
    identity: Identity<S, Set>,
    _aggregate: Arc<AggregateObservation<Key>>,
}
impl<S, Set, M> WorthQueryInvariantGraphComponent<S, Set, M> {
    pub fn members(&self) -> &[WorthQueryInvariantGraphMember<S, M>] {
        &self.members
    }
    pub fn identity(&self) -> &Identity<S, Set> {
        &self.identity
    }
    pub fn len(&self) -> usize {
        self.members.len()
    }
    pub fn is_empty(&self) -> bool {
        false
    }
}
impl<S: ApplicationSchema, O: ApplicationOperationMarkerIdentity<S>>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, S, O>
{
    /// Member46 association and this operation's genuine Member43 source scope
    /// are both required; atomic carrier reads alone cannot issue this answer.
    pub fn decision_graph_component_of<Set, R: OperationReads<O>, M>(
        &mut self,
        set: ApplicationComponentSetRef<S, Set, M>,
        relation: ApplicationDerivedRelationRef<S, R, M, M>,
        member: &WorthQueryInvariantGraphMember<S, M>,
    ) -> Result<WorthQueryInvariantGraphComponent<S, Set, M>, Denial> {
        let result = (|| {
            let denied = || Denial::new(Kind::UndeclaredDecisionTarget, set.name());
            let (from, to) = self.derived_member_domains(relation)?;
            if from != to {
                return Err(denied());
            }
            self.check_derived_member(member, &from)?;
            let layout = self
                .reader
                .layout
                .component_set(set.name())
                .ok_or_else(denied)?
                .clone();
            let source = self
                .reader
                .layout
                .derived_relation(relation.name())
                .ok_or_else(denied)?
                .handle
                .clone();
            if layout.source.as_ref() != Some(&source)
                || layout.member.as_ref() != set.member_selector().name()
                || self
                    .reader
                    .layout
                    .member_binding(set.member_selector())
                    .as_ref()
                    != Some(&from)
            {
                return Err(denied());
            }
            let admission = self.admission_identity.ok_or_else(denied)?;
            let contracts = self.admitted_contracts.clone().ok_or_else(denied)?;
            let root = self.admitted_root.as_ref().ok_or_else(denied)?.entity;
            let scope = contracts
                .graph_reads()
                .roles()
                .iter()
                .flat_map(|role| role.read_scopes())
                .find_map(|scope| match scope {
                    WorthQueryOperationGraphReadScope::DerivedSource(scope)
                        if scope.schema() == self.binding_identity
                            && scope.relation() == relation.name() =>
                    {
                        Some(scope.clone())
                    }
                    _ => None,
                })
                .ok_or_else(denied)?;
            // The complete source proof/facts precede any aggregate acquisition.
            let native =
                Arc::new(self.read_derived_key(relation, member.key.clone(), Shape::Component)?);
            let pin=self.reader.runtime.read_truth().pin_read_capability(self.reader.snapshot,&[],self.reader.execution).map_err(|cause|match cause {
                worth_relational::facade::runtime::RelationalPinnedReadAdmissionDenial::Request(stop)=>self.reader.walk_stop(stop.into(),set.name()),
                _=>Denial::new(Kind::EndpointUnavailable,set.name()),
            })?;
            let execution = *self.reader.execution;
            let acquired=execution.run_native_with_checkpoint(|context|{
                if !layout.matches_native_definition(&pin,||AggregateObservation::checkpoint(context)).map_err(|cause|self.reader.walk_stop(cause,set.name()))? {return Err(denied());}
                let aggregate=self.reader.read_member_aggregate_identity(&pin,layout.member_kind,layout.definition,member.key.clone(),set.name())?;
                if aggregate.native().len()!=native.data().endpoints().len() {return Err(Denial::new(Kind::InvalidComponent,set.name()));}
                for (actual,expected) in aggregate.native().members().zip(native.data().endpoints()) {
                    let mut poll=||AggregateObservation::checkpoint(context).map_err(|cause|self.reader.walk_stop(cause,set.name()));
                    poll()?;
                    crate::domain_computation::primary_graph::aggregate_observation::member_work::checkpoint_names(actual,&mut poll)?;
                    crate::domain_computation::primary_graph::aggregate_observation::member_work::checkpoint_names(expected,&mut poll)?;
                    if actual!=expected {return Err(Denial::new(Kind::InvalidComponent,set.name()));}
                }
                let projection=pin.project_snapshot(pin.snapshot()).ok_or_else(denied)?;
                let mut facts=SourceFactsBuilder::default();
                for (entity,_) in native.data().entity_dependencies(){
                    AggregateObservation::checkpoint(context).map_err(|cause|self.reader.walk_stop(cause,set.name()))?;
                    facts.observe_entity(&execution,*entity).map_err(|cause|self.reader.walk_stop(cause,set.name()))?;
                }
                for probe in native.data().adjacency_dependencies(){
                    AggregateObservation::checkpoint(context).map_err(|cause|self.reader.walk_stop(cause,set.name()))?;
                    facts.observe_adjacency(&execution,&projection,probe.anchor,probe.relation,if probe.outgoing {RelationalAdjacencyDirection::Outgoing}else{RelationalAdjacencyDirection::Incoming}).map_err(|error|match error {
                        SourceFactReadDenial::Execution(cause)=>self.reader.walk_stop(cause,set.name()),
                        SourceFactReadDenial::WorkLimitExceeded=>Denial::new(Kind::WorkBudgetExceeded,set.name()),
                        SourceFactReadDenial::Unavailable=>Denial::new(Kind::EndpointUnavailable,set.name()),
                    })?;
                }
                let facts=facts.finish(&execution).map_err(|cause|self.reader.walk_stop(cause,set.name()))?;
                let selection=Arc::new(ComponentSelection::from_derived(scope,layout.clone(),root,member.key.clone(),self.reader.snapshot.clone(),vec![aggregate.clone()],vec![facts],||AggregateObservation::checkpoint(context)).map_err(|cause|self.reader.walk_stop(cause,set.name()))?.ok_or_else(denied)?);
                self.decision_facts.record_component(&selection,admission,contracts,self.reader.authority_identity,context).map_err(|error|match error {
                    crate::domain_computation::primary_graph::invariant_projection::operation_reader::decision_reads::structural_walks::WalkRecordDenial::Execution(cause)=>self.reader.walk_stop(cause,set.name()),
                    _=>Denial::new(Kind::ForeignIdentity,set.name()),
                })?;
                Ok(aggregate)
            });
            let aggregate = match acquired {
                Ok((answer, _)) => answer?,
                Err(stop) => return Err(self.reader.walk_stop(stop.into(), set.name())),
            };
            let mut members = Vec::new();
            members
                .try_reserve_exact(aggregate.native().len())
                .map_err(|_| self.reader.walk_stop(Cause::AllocationFailed, set.name()))?;
            for key in aggregate.native().members() {
                self.reader
                    .execution
                    .check_live()
                    .map_err(|stop| self.reader.walk_stop(stop.into(), set.name()))?;
                members.push(self.issued_derived_member(key, &from, &native)?);
            }
            self.reader
                .execution
                .check_live()
                .map_err(|stop| self.reader.walk_stop(stop.into(), set.name()))?;
            Ok(WorthQueryInvariantGraphComponent {
                identity: Identity::from_component_read(
                    aggregate.native(),
                    self.binding_identity.clone(),
                ),
                members,
                _aggregate: aggregate,
            })
        })();
        if result.is_err() {
            self.decision_facts.fail_component();
            self.fail_member(member);
        }
        result
    }
}
