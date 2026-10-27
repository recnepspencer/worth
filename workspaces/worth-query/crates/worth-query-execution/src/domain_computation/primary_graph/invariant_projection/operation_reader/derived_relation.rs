//! Derived reads issue the complete physical dependency closure, not cached authority.
use super::super::{
    WorthQueryInvariantProjectionTraversalDenial as Denial,
    WorthQueryInvariantProjectionTraversalDenialKind as Kind,
};
use super::decision_reads::DecisionReadOutcome;
use super::*;
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationAdjacencyDirection as Direction, WorthQueryApplicationFactKey as Fact,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationDerivedRelationRef, ApplicationOperationMarkerIdentity,
};
use worth_query_installation::facade::ApplicationOperationDecisionReadTarget as Target;
use worth_relational::facade::{
    runtime::{DerivedRelationReadDenial as NativeDenial, PinnedDerivedRelationRead},
    schema::DerivedRelationDenial as DefinitionDenial,
    transactions::RelationalExecutionDenialCause as Cause,
};

/// One canonical derived endpoint pair. It has no stored relation identity.
pub struct WorthQueryInvariantDerivedRelation<Schema, Relation, From, To> {
    from: WorthQueryInvariantEntityIdentity<Schema, From>,
    to: WorthQueryInvariantEntityIdentity<Schema, To>,
    marker: PhantomData<fn() -> Relation>,
}
impl<S, R, F, T> WorthQueryInvariantDerivedRelation<S, R, F, T> {
    pub fn from(&self) -> &WorthQueryInvariantEntityIdentity<S, F> {
        &self.from
    }
    pub fn to(&self) -> &WorthQueryInvariantEntityIdentity<S, T> {
        &self.to
    }
}

impl<Schema: ApplicationSchema, Operation: ApplicationOperationMarkerIdentity<Schema>>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>
{
    pub(in crate::domain_computation::primary_graph) fn decision_derived_count<
        Relation: OperationReads<Operation>,
        From,
        To,
    >(
        &mut self,
        relation: ApplicationDerivedRelationRef<Schema, Relation, From, To>,
        root: &WorthQueryInvariantEntityIdentity<Schema, From>,
        association:&worth_query_declaration::facade::application_schema::ApplicationRuleReadableCollectionAssociation,
    ) -> Result<u64, Denial> {
        let result = self.read_derived(relation, root).and_then(|native| {
            let count = u64::try_from(native.data().endpoints().len()).map_err(|_| {
                self.reader
                    .walk_stop(Cause::WorkCounterOverflow, relation.name())
            })?;
            let product = self
                .selected_product
                .ok_or_else(|| Denial::new(Kind::UndeclaredDecisionTarget, relation.name()))?;
            let definition = &self
                .reader
                .layout
                .derived_relation(relation.name())
                .expect("read admitted this installed definition")
                .handle;
            let observed = self.reader.managed_views.retain_native_count(
                self.runtime_authority.as_u64(),
                self.binding_identity,
                definition,
                association,
                root.entity_id,
                product,
                native.data(),
                self.reader.execution,
            );
            self.reader
                .execution
                .check_live()
                .map_err(|cause| self.reader.walk_stop(cause.into(), relation.name()))?;
            Ok(observed.unwrap_or(count))
        });
        if result.is_err() {
            self.fail_derived(root, relation.from());
        }
        result
    }

    /// Read a declared derived source and retain every physical and empty read
    /// as this operation's decision facts. Cost follows the explored recipe paths.
    pub fn decision_derived_relations_from<Relation: OperationReads<Operation>, From, To>(
        &mut self,
        relation: ApplicationDerivedRelationRef<Schema, Relation, From, To>,
        root: &WorthQueryInvariantEntityIdentity<Schema, From>,
    ) -> Result<Vec<WorthQueryInvariantDerivedRelation<Schema, Relation, From, To>>, Denial> {
        let result = self.read_derived(relation, root).and_then(|native| {
            let target_kind = self
                .reader
                .layout
                .entity_kind(relation.to())
                .ok_or_else(|| Denial::new(Kind::RelationNotInstalled, relation.name()))?;
            let mut pairs = Vec::new();
            pairs
                .try_reserve_exact(native.data().endpoints().len())
                .map_err(|_| {
                    self.reader
                        .walk_stop(Cause::AllocationFailed, relation.name())
                })?;
            for endpoint in native.data().endpoints() {
                self.reader
                    .execution
                    .check_live()
                    .map_err(|cause| self.reader.walk_stop(cause.into(), relation.name()))?;
                pairs.push(WorthQueryInvariantDerivedRelation {
                    from: WorthQueryInvariantEntityIdentity {
                        entity_id: root.entity_id,
                        kind: root.kind,
                        entity: Arc::clone(&root.entity),
                        authority_identity: root.authority_identity,
                        _marker: PhantomData,
                    },
                    to: WorthQueryInvariantEntityIdentity {
                        entity_id: *endpoint,
                        kind: target_kind,
                        entity: Arc::from(relation.to()),
                        authority_identity: self.reader.authority_identity,
                        _marker: PhantomData,
                    },
                    marker: PhantomData,
                });
            }
            self.reader
                .execution
                .check_live()
                .map_err(|cause| self.reader.walk_stop(cause.into(), relation.name()))?;
            Ok(pairs)
        });
        if result.is_err() {
            self.fail_derived(root, relation.from());
        }
        result
    }

    fn read_derived<Relation: OperationReads<Operation>, From, To>(
        &mut self,
        relation: ApplicationDerivedRelationRef<Schema, Relation, From, To>,
        root: &WorthQueryInvariantEntityIdentity<Schema, From>,
    ) -> Result<PinnedDerivedRelationRead, Denial> {
        let denied = || Denial::new(Kind::UndeclaredDecisionTarget, relation.name());
        self.admission_identity.ok_or_else(denied)?;
        self.admitted_contracts.as_ref().ok_or_else(denied)?;
        let layout = self
            .reader
            .layout
            .derived_relation(relation.name())
            .ok_or_else(|| Denial::new(Kind::RelationNotInstalled, relation.name()))?;
        if layout.from.as_ref() != relation.from()
            || layout.to.as_ref() != relation.to()
            || !layout
                .operations
                .iter()
                .any(|name| name.as_ref() == Operation::IDENTIFIER)
        {
            return Err(denied());
        }
        if !self.reader.identity_is_local(root, relation.from())
            || root.kind != layout.handle.parts().from
        {
            return Err(Denial::new(Kind::ForeignIdentity, relation.name()));
        }
        let definition = layout.handle.clone();
        let sources = Arc::clone(&layout.sources);
        for (name, _, from, to) in sources.iter() {
            self.reader
                .execution
                .check_live()
                .map_err(|cause| self.reader.walk_stop(cause.into(), relation.name()))?;
            for kind in [from, to] {
                let entity = self.reader.layout.entity_name(*kind).ok_or_else(denied)?;
                self.admit_decision_target(&Target::Entity {
                    entity: entity.to_owned(),
                })
                .map_err(|_| denied())?;
            }
            self.admit_decision_target(&Target::Relation {
                relation: name.to_string(),
                from: self
                    .reader
                    .layout
                    .entity_name(*from)
                    .ok_or_else(denied)?
                    .to_owned(),
                to: self
                    .reader
                    .layout
                    .entity_name(*to)
                    .ok_or_else(denied)?
                    .to_owned(),
            })
            .map_err(|_| denied())?;
        }
        let native={
            let pin=self.reader.runtime.read_truth()
                .pin_read_capability(self.reader.snapshot,&[],self.reader.execution)
                .map_err(|cause| match cause {
                    worth_relational::facade::runtime::RelationalPinnedReadAdmissionDenial::Request(stop)=>
                        self.reader.walk_stop(worth_execution::WorkCeilingDenial::Stopped(stop).into(),relation.name()),
                    _=>Denial::new(Kind::EndpointUnavailable,relation.name()),
                })?;
            pin.derived_relations_from(self.reader.execution,&definition,root.entity_id,
                self.reader.work_budget.remaining())
        }.map_err(|cause|self.derived_denial(cause,relation.name()))?;
        let reads = native.data().reads();
        self.reader
            .work
            .record_adjacencies(reads.adjacency_lists, reads.relation_records);
        self.reader.work.record_endpoints(reads.entity_records);
        if !self.reader.work_budget.consume(native.work_units()) {
            return Err(Denial::new(Kind::WorkBudgetExceeded, relation.name()));
        }
        // This finite replay allowance covers each actual physical adjacency's
        // existing frontier metric plus its empty-read admission. It is not a
        // claim that the extra admission fits the caller's original work limit.
        let maximum_work_units = native.work_units().checked_add(1).ok_or_else(|| {
            self.reader
                .walk_stop(Cause::WorkCounterOverflow, relation.name())
        })?;
        for (entity, kind) in native.data().entity_dependencies() {
            self.reader
                .execution
                .check_live()
                .map_err(|cause| self.reader.walk_stop(cause.into(), relation.name()))?;
            let name = self.reader.layout.entity_name(*kind).ok_or_else(denied)?;
            self.reader.realized_scope.record(*entity);
            self.decision_facts.record(
                Fact::Entity {
                    entity: name.to_owned(),
                    entity_id: *entity,
                },
                DecisionReadOutcome::Observed,
            );
        }
        for probe in native.data().adjacency_dependencies() {
            self.reader
                .execution
                .check_live()
                .map_err(|cause| self.reader.walk_stop(cause.into(), relation.name()))?;
            let name = sources
                .iter()
                .find(|(_, kind, _, _)| *kind == probe.relation)
                .map(|(name, _, _, _)| name)
                .ok_or_else(denied)?;
            self.decision_facts.record(
                Fact::Adjacency {
                    relation: name.to_string(),
                    anchor: probe.anchor,
                    direction: if probe.outgoing {
                        Direction::Outgoing
                    } else {
                        Direction::Incoming
                    },
                    maximum_work_units,
                },
                DecisionReadOutcome::Observed,
            );
        }
        self.reader
            .execution
            .check_live()
            .map_err(|cause| self.reader.walk_stop(cause.into(), relation.name()))?;
        Ok(native)
    }

    fn fail_derived<Entity>(
        &mut self,
        root: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        entity: &str,
    ) {
        self.decision_facts.record(
            Fact::Entity {
                entity: entity.to_owned(),
                entity_id: root.entity_id,
            },
            DecisionReadOutcome::Failed,
        );
    }
    fn derived_denial(&mut self, cause: NativeDenial, subject: &str) -> Denial {
        match cause {
            NativeDenial::Request(cause) => self.reader.walk_stop(cause, subject),
            NativeDenial::Definition(DefinitionDenial::WorkBudgetExceeded) => {
                self.reader.work_budget.mark_exceeded();
                Denial::new(Kind::WorkBudgetExceeded, subject)
            }
            NativeDenial::Definition(
                DefinitionDenial::ForeignDefinition | DefinitionDenial::ForeignObservation,
            ) => Denial::new(Kind::ForeignIdentity, subject),
            NativeDenial::Definition(DefinitionDenial::OutsideDeclaredAccess) => {
                Denial::new(Kind::UndeclaredDecisionTarget, subject)
            }
            NativeDenial::Definition(_) | NativeDenial::Structural(_) => {
                Denial::new(Kind::EndpointUnavailable, subject)
            }
        }
    }
}
