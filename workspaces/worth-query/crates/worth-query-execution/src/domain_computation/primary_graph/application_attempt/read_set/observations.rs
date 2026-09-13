use std::marker::PhantomData;

use worth_query_installation::facade::{
    ApplicationEntityRef, ApplicationFieldRef, ApplicationFieldUnit,
    ApplicationOperationDecisionReadTarget, ApplicationReadableScalarValueBinding,
    ApplicationRelationRef, DeclaredApplicationFieldValue, OperationReads, WritePosture,
};

use super::super::fact::{WorthQueryApplicationFactKey, WorthQueryApplicationObservedFact};
use super::super::observation::{exact_relations, observe_field_value};
use super::{denial, WorthQueryApplicationReadAttempt, WorthQueryObservedApplicationRelation};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationEntityIdentity,
};

impl<Schema, Operation, Input, Scope, Phase>
    WorthQueryApplicationReadAttempt<Schema, Operation, Input, Scope, Phase>
{
    pub fn observe_entity<Entity>(
        &mut self,
        entity: ApplicationEntityRef<Schema, Entity>,
        identity: &WorthQueryApplicationEntityIdentity<Schema, Entity>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Entity: OperationReads<Operation>,
    {
        let target = ApplicationOperationDecisionReadTarget::Entity {
            entity: entity.name().to_string(),
        };
        let read_scope = self.admit_target(&target)?;
        self.validate_identity_authority(entity.name(), identity)?;
        let key = WorthQueryApplicationFactKey::Entity {
            entity: entity.name().to_string(),
            entity_id: identity.entity_id(),
        };
        self.admit_fact_key(&key)?;
        self.validate_identity_freshness(entity.name(), identity)?;
        self.installed_read_scopes.insert(key.clone(), read_scope);
        self.facts.insert(
            key,
            WorthQueryApplicationObservedFact::Entity {
                entity_id: identity.entity_id(),
                kind: identity.entity_kind(),
            },
        );
        Ok(())
    }

    pub fn observe_field<Entity, Aspect, Field, Value, Write, Equality, Unit>(
        &mut self,
        identity: &WorthQueryApplicationEntityIdentity<Schema, Entity>,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
    ) -> Result<Value, WorthQueryApplicationAttemptDenial>
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
        let read_scope = self.admit_target(&target)?;
        self.validate_identity_authority(field.entity(), identity)?;
        let graph_layout = self.field_layout(field.entity(), field.aspect(), field.field())?;
        let key = WorthQueryApplicationFactKey::Field {
            entity: field.entity().to_string(),
            entity_id: identity.entity_id(),
            locator: graph_layout.clone(),
        };
        self.admit_fact_key(&key)?;
        self.validate_identity_freshness(field.entity(), identity)?;
        let value = self
            .lease
            .handle()
            .with_runtime(|runtime| {
                observe_field_value(
                    runtime,
                    self.lease.snapshot(),
                    identity.entity_id(),
                    identity.entity_kind(),
                    &graph_layout,
                )
            })
            .ok_or_else(|| {
                denial(
                    WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact,
                    field.field(),
                )
            })?;
        let typed = Field::Binding::decode(&value).map_err(|_| {
            denial(
                WorthQueryApplicationAttemptDenialKind::InvalidAuthoritativeValue,
                field.field(),
            )
        })?;
        self.installed_read_scopes.insert(key.clone(), read_scope);
        self.facts.insert(
            key,
            WorthQueryApplicationObservedFact::Field {
                entity_id: identity.entity_id(),
                kind: identity.entity_kind(),
                locator: graph_layout,
                value,
            },
        );
        Ok(typed)
    }

    pub fn observe_relation<Relation, From, To>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
        from: &WorthQueryApplicationEntityIdentity<Schema, From>,
        to: &WorthQueryApplicationEntityIdentity<Schema, To>,
    ) -> Result<
        WorthQueryObservedApplicationRelation<Schema, Relation, From, To>,
        WorthQueryApplicationAttemptDenial,
    >
    where
        Relation: OperationReads<Operation>,
    {
        let target = ApplicationOperationDecisionReadTarget::Relation {
            relation: relation.name().to_string(),
            from: relation.from().to_string(),
            to: relation.to().to_string(),
        };
        let read_scope = self.admit_target(&target)?;
        self.validate_identity_authority(relation.from(), from)?;
        self.validate_identity_authority(relation.to(), to)?;
        let layout = self
            .layout
            .relation(relation.name())
            .cloned()
            .ok_or_else(|| {
                denial(
                    WorthQueryApplicationAttemptDenialKind::UndeclaredDecisionRead,
                    relation.name(),
                )
            })?;
        let key = WorthQueryApplicationFactKey::Relation {
            relation: relation.name().to_string(),
            from: from.entity_id(),
            to: to.entity_id(),
        };
        self.admit_fact_key(&key)?;
        self.validate_identity_freshness(relation.from(), from)?;
        self.validate_identity_freshness(relation.to(), to)?;
        let matching_relations = self.lease.handle().with_runtime(|runtime| {
            exact_relations(
                runtime,
                self.lease.snapshot(),
                layout.kind,
                from.entity_id(),
                to.entity_id(),
            )
        })?;
        if matching_relations.len() > 1 {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::AmbiguousRelation,
                relation.name(),
            ));
        }
        let count = matching_relations.len();
        let retained_relations = matching_relations.clone();
        self.installed_read_scopes.insert(key.clone(), read_scope);
        self.facts.insert(
            key,
            WorthQueryApplicationObservedFact::Relation {
                relation_kind: layout.kind,
                from: from.entity_id(),
                to: to.entity_id(),
                matching_relations,
            },
        );
        Ok(WorthQueryObservedApplicationRelation {
            count,
            matching_relations: retained_relations,
            _marker: PhantomData,
        })
    }
}
