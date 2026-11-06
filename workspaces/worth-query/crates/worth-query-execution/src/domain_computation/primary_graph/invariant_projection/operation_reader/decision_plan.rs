mod denial;
pub use denial::{
    WorthQueryInvariantDecisionPlanDenial, WorthQueryInvariantDecisionPlanDenialKind,
};
use worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus;
mod orbit;
use worth_query_installation::facade::{
    ApplicationEntityRef, ApplicationOperationDecisionReadTarget,
    ApplicationReadableScalarValueBinding, ApplicationRelationRef, ApplicationSchema,
    OperationReads,
};

use worth_relational::facade::identity::EntityId;

use super::decision_reads::DecisionReadOutcome;
use super::WorthQueryApplicationOperationInvariantProjectionReader;
mod indexed_selection;
mod prepared_selection;
pub use prepared_selection::WorthQueryPreparedEntitySelection;
mod body_requirement;
mod body_retirement;
mod body_selection;
pub(crate) mod field_presence;
mod operation_body;
mod predecode_admission;
mod signed_evidence;
mod structural_walk;
use crate::domain_computation::application_contract_admission::graph_reads_admit_target;
use crate::domain_computation::primary_graph::{
    application_attempt::{WorthQueryApplicationAdjacencyDirection, WorthQueryApplicationFactKey},
    WorthQueryApplicationInvariantProjectionReader, WorthQueryInvariantEntityIdentity,
    WorthQueryInvariantProjectionTraversalDenial, WorthQueryInvariantRelation,
};

impl<'reader, 'runtime, Schema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'reader, 'runtime, Schema, Operation>
where
    Schema: ApplicationSchema,
{
    pub fn require_decision_entity<Entity>(
        &mut self,
        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        entity: ApplicationEntityRef<Schema, Entity>,
    ) -> Result<(), WorthQueryInvariantDecisionPlanDenial>
    where
        Entity: OperationReads<Operation>,
    {
        self.require_named_decision_entity(identity, entity.name())
    }

    fn require_named_decision_entity<Entity>(
        &mut self,
        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        entity: &str,
    ) -> Result<(), WorthQueryInvariantDecisionPlanDenial> {
        let target = ApplicationOperationDecisionReadTarget::Entity {
            entity: entity.to_string(),
        };
        self.admit_decision_target(&target)?;
        if !self.reader.identity_is_local(identity, entity) {
            return Err(WorthQueryInvariantDecisionPlanDenial::new(
                WorthQueryInvariantDecisionPlanDenialKind::ForeignIdentity,
                entity,
            ));
        }
        self.decision_facts.record(
            WorthQueryApplicationFactKey::Entity {
                entity: entity.to_string(),
                entity_id: identity.entity_id,
            },
            DecisionReadOutcome::Observed,
        );
        Ok(())
    }

    pub fn decision_field<Entity, Aspect, Field, Value>(
        &mut self,

        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        field: Field,
    ) -> Result<Option<Value>, WorthQueryInvariantDecisionPlanDenial>
    where
        Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = Entity,
            Aspect = Aspect,
            Value = Value,
        >,
        Field::Binding: ApplicationReadableScalarValueBinding,
    {
        let _execution = self.reader.execution;
        self.require_decision_field(identity, field)?;
        Ok(self.reader.field(identity, field))
    }

    pub fn decision_relations_from<Relation, From, To>(
        &mut self,

        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        from: &WorthQueryInvariantEntityIdentity<Schema, From>,
    ) -> Result<
        Vec<WorthQueryInvariantRelation<Schema, Relation, From, To>>,
        WorthQueryInvariantProjectionTraversalDenial,
    >
    where
        Relation: OperationReads<Operation>,
    {
        let _execution = self.reader.execution;
        let target = relation_target(&relation);
        self.admit_decision_target(&target)
            .map_err(|denial| decision_traversal_denial(&denial))?;
        self.recorded_adjacency(
            &target,
            from.entity_id,
            WorthQueryApplicationAdjacencyDirection::Outgoing,
            |reader| reader.relations_from(relation, from),
        )
    }

    pub fn decision_relations_to<Relation, From, To>(
        &mut self,

        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        to: &WorthQueryInvariantEntityIdentity<Schema, To>,
    ) -> Result<
        Vec<WorthQueryInvariantRelation<Schema, Relation, From, To>>,
        WorthQueryInvariantProjectionTraversalDenial,
    >
    where
        Relation: OperationReads<Operation>,
    {
        let _execution = self.reader.execution;
        let target = relation_target(&relation);
        self.admit_decision_target(&target)
            .map_err(|denial| decision_traversal_denial(&denial))?;
        self.recorded_adjacency(
            &target,
            to.entity_id,
            WorthQueryApplicationAdjacencyDirection::Incoming,
            |reader| reader.relations_to(relation, to),
        )
    }

    /// Runs one adjacency read and records its fact whatever the read
    /// returned, so no early return skips the recording. A read that failed
    /// is recorded as failed: whoever swallows the failure has an outcome its
    /// fact does not hold, and the projection seals nothing.
    fn recorded_adjacency<Relations, Cause>(
        &mut self,
        target: &ApplicationOperationDecisionReadTarget,
        anchor: EntityId,
        direction: WorthQueryApplicationAdjacencyDirection,
        read: impl FnOnce(
            &mut WorthQueryApplicationInvariantProjectionReader<'runtime, Schema>,
        ) -> Result<Relations, Cause>,
    ) -> Result<Relations, Cause> {
        let before = self.reader.work_budget;
        let relations = read(self.reader);
        let maximum_work_units = self
            .reader
            .work_budget
            .adjacency_recomparison_limit_since(before);
        self.decision_facts.record(
            WorthQueryApplicationFactKey::Adjacency {
                relation: target_relation_name(target),
                anchor,
                direction,
                maximum_work_units,
            },
            if relations.is_ok() {
                DecisionReadOutcome::Observed
            } else {
                DecisionReadOutcome::Failed
            },
        );
        relations
    }

    pub fn require_decision_field<Entity, Aspect, Field, Value>(
        &mut self,
        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        field: Field,
    ) -> Result<(), WorthQueryInvariantDecisionPlanDenial>
    where
        Field: worth_query_declaration::facade::application_schema::fragment::RecordSelection<
            Entity = Entity,
            Aspect = Aspect,
            Value = Value,
        >,
        Field::Binding: ApplicationReadableScalarValueBinding,
    {
        field
            .installed(self.reader.value_bindings.fragments())
            .map_err(|_| {
                WorthQueryInvariantDecisionPlanDenial::new(
                    WorthQueryInvariantDecisionPlanDenialKind::FieldNotInstalled,
                    field.field(),
                )
            })?;
        self.require_named_field(identity, field.entity(), field.aspect(), field.field())
    }

    fn require_named_field<Entity>(
        &mut self,
        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        entity: &str,
        aspect: &str,
        field: &str,
    ) -> Result<(), WorthQueryInvariantDecisionPlanDenial> {
        let target = ApplicationOperationDecisionReadTarget::Field {
            entity: entity.to_string(),
            aspect: aspect.to_string(),
            field: field.to_string(),
        };
        self.admit_decision_target(&target)?;
        if !self.reader.identity_is_local(identity, entity) {
            return Err(WorthQueryInvariantDecisionPlanDenial::new(
                WorthQueryInvariantDecisionPlanDenialKind::ForeignIdentity,
                entity,
            ));
        }
        let locator = self
            .reader
            .layout
            .field_locator(entity, aspect, field)
            .cloned()
            .ok_or_else(|| {
                WorthQueryInvariantDecisionPlanDenial::new(
                    WorthQueryInvariantDecisionPlanDenialKind::FieldNotInstalled,
                    field,
                )
            })?;
        self.record_field_dependency(
            identity,
            &ApplicationFieldBindingLocus::new(entity, aspect, field),
            locator,
        );
        Ok(())
    }

    fn record_field_dependency<Entity>(
        &mut self,
        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        field: &ApplicationFieldBindingLocus,
        locator: worth_foundational::facade::AspectFieldLocator,
    ) {
        let entity = field.entity();
        if let Some(presence) = self.reader.layout.record_presence_locator(field).cloned()
            && presence != locator
            && self.admitted_contracts.as_deref().is_some_and(|contract| {
                field_presence::may_write(contract.touches(), entity, &locator)
            })
        {
            self.decision_facts.record(
                WorthQueryApplicationFactKey::Field {
                    entity: entity.to_owned(),
                    entity_id: identity.entity_id,
                    locator: presence,
                },
                DecisionReadOutcome::Observed,
            );
        }
        if self.reader.layout.field_is_optional(entity, &locator)
            && self.admitted_contracts.as_deref().is_some_and(|contract| {
                field_presence::may_write(contract.touches(), entity, &locator)
            })
        {
            self.decision_facts.record(
                WorthQueryApplicationFactKey::AspectPresence {
                    entity: entity.to_owned(),
                    entity_id: identity.entity_id,
                    locator: locator.clone(),
                },
                DecisionReadOutcome::Observed,
            );
        }
        self.decision_facts.record(
            WorthQueryApplicationFactKey::Field {
                entity: entity.to_owned(),
                entity_id: identity.entity_id,
                locator,
            },
            DecisionReadOutcome::Observed,
        );
    }

    pub fn require_decision_relation<Relation, From, To>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        from: &WorthQueryInvariantEntityIdentity<Schema, From>,
        to: &WorthQueryInvariantEntityIdentity<Schema, To>,
    ) -> Result<(), WorthQueryInvariantDecisionPlanDenial>
    where
        Relation: OperationReads<Operation>,
    {
        let target = ApplicationOperationDecisionReadTarget::Relation {
            relation: relation.name().to_string(),
            from: relation.from().to_string(),
            to: relation.to().to_string(),
        };
        self.admit_decision_target(&target)?;
        if !self.reader.identity_is_local(from, relation.from())
            || !self.reader.identity_is_local(to, relation.to())
        {
            return Err(WorthQueryInvariantDecisionPlanDenial::new(
                WorthQueryInvariantDecisionPlanDenialKind::ForeignIdentity,
                relation.name(),
            ));
        }
        self.decision_facts.record(
            WorthQueryApplicationFactKey::Relation {
                relation: relation.name().to_string(),
                from: from.entity_id,
                to: to.entity_id,
            },
            DecisionReadOutcome::Observed,
        );
        Ok(())
    }

    pub(super) fn admit_decision_target(
        &self,
        target: &ApplicationOperationDecisionReadTarget,
    ) -> Result<(), WorthQueryInvariantDecisionPlanDenial> {
        if self
            .admitted_contracts
            .as_deref()
            .is_none_or(|contract| match target {
                ApplicationOperationDecisionReadTarget::ManagedIndex { parent } => {
                    contract.managed_index_reads().contains(parent)
                }
                _ => graph_reads_admit_target(contract.graph_reads(), target),
            })
        {
            Ok(())
        } else {
            Err(WorthQueryInvariantDecisionPlanDenial::new(
                WorthQueryInvariantDecisionPlanDenialKind::UndeclaredDecisionTarget,
                format!("{target:?}"),
            ))
        }
    }
}

fn relation_target<Schema, Relation, From, To>(
    relation: &ApplicationRelationRef<Schema, Relation, From, To>,
) -> ApplicationOperationDecisionReadTarget {
    ApplicationOperationDecisionReadTarget::Relation {
        relation: relation.name().to_string(),
        from: relation.from().to_string(),
        to: relation.to().to_string(),
    }
}

fn target_relation_name(target: &ApplicationOperationDecisionReadTarget) -> String {
    match target {
        ApplicationOperationDecisionReadTarget::Relation { relation, .. } => relation.clone(),
        _ => unreachable!("relation_target always constructs a relation target"),
    }
}

fn decision_traversal_denial(
    denial: &WorthQueryInvariantDecisionPlanDenial,
) -> WorthQueryInvariantProjectionTraversalDenial {
    WorthQueryInvariantProjectionTraversalDenial::new(
        super::super::WorthQueryInvariantProjectionTraversalDenialKind::UndeclaredDecisionTarget,
        denial.subject(),
    )
}
