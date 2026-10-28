//! One issued physical/empty fact closure for every derived source shape.
use super::*;
use worth_relational::facade::{
    runtime::DerivedRelationReadShape as Shape, schema::NativeMemberKey,
};
impl<Schema: ApplicationSchema, Operation: ApplicationOperationMarkerIdentity<Schema>>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>
{
    pub(in crate::domain_computation::primary_graph::invariant_projection::operation_reader::derived_relation) fn read_derived<
        Relation: OperationReads<Operation>,
        From,
        To,
    >(
        &mut self,
        relation: ApplicationDerivedRelationRef<Schema, Relation, From, To>,
        root: &WorthQueryInvariantEntityIdentity<Schema, From>,
    ) -> Result<PinnedDerivedRelationRead, Denial> {
        let layout = self
            .reader
            .layout
            .derived_relation(relation.name())
            .ok_or_else(|| Denial::new(Kind::RelationNotInstalled, relation.name()))?;
        let kind = layout
            .handle
            .parts()
            .from
            .entity_kind()
            .ok_or_else(|| Denial::new(Kind::ForeignIdentity, relation.name()))?;
        if !self.reader.identity_is_local(root, relation.from()) || root.kind != kind {
            return Err(Denial::new(Kind::ForeignIdentity, relation.name()));
        }
        self.read_derived_key(
            relation,
            root.entity_id.into(),
            Shape::Adjacency { reverse: false },
        )
    }
    pub(in crate::domain_computation::primary_graph::invariant_projection::operation_reader::derived_relation) fn read_derived_key<
        Relation: OperationReads<Operation>,
        From,
        To,
    >(
        &mut self,
        relation: ApplicationDerivedRelationRef<Schema, Relation, From, To>,
        root: NativeMemberKey,
        shape: Shape,
    ) -> Result<PinnedDerivedRelationRead, Denial> {
        let denied = || Denial::new(Kind::UndeclaredDecisionTarget, relation.name());
        self.admission_identity.ok_or_else(denied)?;
        self.admitted_contracts.as_ref().ok_or_else(denied)?;
        self.selected_product.ok_or_else(denied)?;
        let layout = self
            .reader
            .layout
            .derived_relation(relation.name())
            .ok_or_else(|| Denial::new(Kind::RelationNotInstalled, relation.name()))?;
        if layout.from.borrowed() != relation.from_member()
            || layout.to.borrowed() != relation.to_member()
            || !layout
                .operations
                .iter()
                .any(|name| name.as_ref() == Operation::IDENTIFIER)
        {
            return Err(denied());
        }
        let definition = layout.handle.clone();
        let sources = Arc::clone(&layout.sources);
        let carriers = Arc::clone(&layout.carriers);
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
        for (carrier, _) in carriers.iter() {
            self.admit_decision_target(&Target::Entity {
                entity: carrier.to_string(),
            })
            .map_err(|_| denied())?;
        }
        let native={
            let pin=self.reader.runtime.read_truth()
                .pin_read_capability(self.reader.snapshot,&[],self.reader.execution)
                .map_err(|cause| match cause {
                    worth_relational::facade::runtime::RelationalPinnedReadAdmissionDenial::Request(stop)=>
                        self.reader.walk_stop(stop,relation.name()),
                    _=>Denial::new(Kind::EndpointUnavailable,relation.name()),
                })?;
            pin.derived_source(self.reader.execution,&definition,root, shape,
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

    pub(in crate::domain_computation::primary_graph::invariant_projection::operation_reader::derived_relation) fn fail_derived<
        Entity,
    >(
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
    pub(in crate::domain_computation::primary_graph::invariant_projection::operation_reader::derived_relation) fn derived_denial(
        &mut self,
        cause: NativeDenial,
        subject: &str,
    ) -> Denial {
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
