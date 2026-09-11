use worth_query_installation::facade::{
    ApplicationEntityRef, ApplicationFieldRef, ApplicationFieldUnit,
    ApplicationOperationDecisionReadTarget, ApplicationReadableScalarValueBinding,
    ApplicationRelationRef, ApplicationSchema, DeclaredApplicationFieldValue, OperationReads,
    WritePosture,
};

use super::WorthQueryApplicationOperationInvariantProjectionReader;
use crate::domain_computation::application_contract_admission::graph_reads_admit_target;
use crate::domain_computation::primary_graph::{
    application_attempt::{WorthQueryApplicationAdjacencyDirection, WorthQueryApplicationFactKey},
    WorthQueryInvariantEntityIdentity, WorthQueryInvariantProjectionTraversalDenial,
    WorthQueryInvariantRelation,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryInvariantDecisionPlanDenialKind {
    UndeclaredDecisionTarget,
    ForeignIdentity,
    FieldNotInstalled,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInvariantDecisionPlanDenial {
    kind: WorthQueryInvariantDecisionPlanDenialKind,
    subject: String,
}

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
        let target = ApplicationOperationDecisionReadTarget::Entity {
            entity: entity.name().to_string(),
        };
        self.admit_decision_target(&target)?;
        if !self.reader.identity_is_local(identity, entity.name()) {
            return Err(WorthQueryInvariantDecisionPlanDenial::new(
                WorthQueryInvariantDecisionPlanDenialKind::ForeignIdentity,
                entity.name(),
            ));
        }
        self.decision_facts
            .insert(WorthQueryApplicationFactKey::Entity {
                entity: entity.name().to_string(),
                entity_id: identity.entity_id,
            });
        Ok(())
    }

    pub fn decision_field<Entity, Aspect, Field, Value, Write, Equality, Unit>(
        &mut self,
        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
    ) -> Result<Option<Value>, WorthQueryInvariantDecisionPlanDenial>
    where
        Field: OperationReads<Operation> + DeclaredApplicationFieldValue<Value = Value>,
        Field::Binding: ApplicationReadableScalarValueBinding,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
    {
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
        let target = relation_target(&relation);
        self.admit_decision_target(&target)
            .map_err(|denial| decision_traversal_denial(&denial))?;
        let before = self.reader.work;
        let relations = self.reader.relations_from(relation, from)?;
        let maximum_work_units = adjacency_recomparison_limit(before, self.reader.work);
        self.decision_facts
            .insert(WorthQueryApplicationFactKey::Adjacency {
                relation: target_relation_name(&target),
                anchor: from.entity_id,
                direction: WorthQueryApplicationAdjacencyDirection::Outgoing,
                maximum_work_units,
            });
        Ok(relations)
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
        let target = relation_target(&relation);
        self.admit_decision_target(&target)
            .map_err(|denial| decision_traversal_denial(&denial))?;
        let before = self.reader.work;
        let relations = self.reader.relations_to(relation, to)?;
        let maximum_work_units = adjacency_recomparison_limit(before, self.reader.work);
        self.decision_facts
            .insert(WorthQueryApplicationFactKey::Adjacency {
                relation: target_relation_name(&target),
                anchor: to.entity_id,
                direction: WorthQueryApplicationAdjacencyDirection::Incoming,
                maximum_work_units,
            });
        Ok(relations)
    }

    pub fn require_decision_field<Entity, Aspect, Field, Value, Write, Equality, Unit>(
        &mut self,
        identity: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
    ) -> Result<(), WorthQueryInvariantDecisionPlanDenial>
    where
        Field: OperationReads<Operation> + DeclaredApplicationFieldValue<Value = Value>,
        Field::Binding: ApplicationReadableScalarValueBinding,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
    {
        let target = ApplicationOperationDecisionReadTarget::Field {
            entity: field.entity().to_string(),
            aspect: field.aspect().to_string(),
            field: field.field().to_string(),
        };
        self.admit_decision_target(&target)?;
        if !self.reader.identity_is_local(identity, field.entity()) {
            return Err(WorthQueryInvariantDecisionPlanDenial::new(
                WorthQueryInvariantDecisionPlanDenialKind::ForeignIdentity,
                field.entity(),
            ));
        }
        let locator = self
            .reader
            .layout
            .field_locator(field.entity(), field.aspect(), field.field())
            .cloned()
            .ok_or_else(|| {
                WorthQueryInvariantDecisionPlanDenial::new(
                    WorthQueryInvariantDecisionPlanDenialKind::FieldNotInstalled,
                    field.field(),
                )
            })?;
        self.decision_facts
            .insert(WorthQueryApplicationFactKey::Field {
                entity: field.entity().to_string(),
                entity_id: identity.entity_id,
                locator,
            });
        Ok(())
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
        self.decision_facts
            .insert(WorthQueryApplicationFactKey::Relation {
                relation: relation.name().to_string(),
                from: from.entity_id,
                to: to.entity_id,
            });
        Ok(())
    }

    pub(super) fn admit_decision_target(
        &self,
        target: &ApplicationOperationDecisionReadTarget,
    ) -> Result<(), WorthQueryInvariantDecisionPlanDenial> {
        if self
            .admitted_graph_reads
            .is_none_or(|reads| graph_reads_admit_target(reads, target))
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

fn adjacency_recomparison_limit(
    before: super::super::WorthQueryInvariantProjectionWork,
    after: super::super::WorthQueryInvariantProjectionWork,
) -> usize {
    let examined = after
        .adjacency_edges_inspected()
        .saturating_sub(before.adjacency_edges_inspected());
    let endpoints = after
        .endpoint_records_read()
        .saturating_sub(before.endpoint_records_read());
    examined.saturating_add(endpoints).saturating_add(2)
}

fn decision_traversal_denial(
    denial: &WorthQueryInvariantDecisionPlanDenial,
) -> WorthQueryInvariantProjectionTraversalDenial {
    WorthQueryInvariantProjectionTraversalDenial::new(
        super::super::WorthQueryInvariantProjectionTraversalDenialKind::UndeclaredDecisionTarget,
        denial.subject(),
    )
}

impl WorthQueryInvariantDecisionPlanDenial {
    pub const fn kind(&self) -> WorthQueryInvariantDecisionPlanDenialKind {
        self.kind
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }

    fn new(kind: WorthQueryInvariantDecisionPlanDenialKind, subject: impl Into<String>) -> Self {
        Self {
            kind,
            subject: subject.into(),
        }
    }
}

impl std::fmt::Display for WorthQueryInvariantDecisionPlanDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "invariant decision plan denied: {:?} ({})",
            self.kind, self.subject
        )
    }
}

impl std::error::Error for WorthQueryInvariantDecisionPlanDenial {}
