use std::marker::PhantomData;
use std::sync::Arc;

use worth_query_installation::facade::{
    ApplicationFieldRef, ApplicationFieldUnit, ApplicationScalarValueBinding,
    DeclaredApplicationFieldValue, EqualityPredicate, OperationReads, WritePosture,
};
use worth_relational::facade::transactions::EntityReference;

use super::{denial, WorthQueryApplicationEffectEntity, WorthQueryApplicationEffectProgramBuilder};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationEntityIdentity, WorthQueryInvariantMutationTarget,
};

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgramBuilder<Schema, Operation, Input, Scope>
{
    pub(in crate::domain_computation::primary_graph) fn resolve_observed_entity<
        Entity,
        Aspect,
        Field,
        Value,
        Write,
        Unit,
    >(
        &self,
        field: ApplicationFieldRef<
            Schema,
            Entity,
            Aspect,
            Field,
            Value,
            Write,
            EqualityPredicate,
            Unit,
        >,
        value: Value,
    ) -> Result<WorthQueryApplicationEffectEntity<Schema, Entity>, WorthQueryApplicationAttemptDenial>
    where
        Field: OperationReads<Operation> + DeclaredApplicationFieldValue<Value = Value>,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
    {
        let locator = self.field_locator(field.entity(), field.aspect(), field.field())?;
        let encoded = Field::Binding::encode(&value).map_err(|_| {
            denial(
                WorthQueryApplicationAttemptDenialKind::InvalidEffectValue,
                field.field(),
            )
        })?;
        let expected_kind = self.layout.entity_kind(field.entity()).ok_or_else(|| {
            denial(
                WorthQueryApplicationAttemptDenialKind::UndeclaredEffect,
                field.entity(),
            )
        })?;
        let mut matches = self.read_set.facts.iter().filter_map(|fact| match fact {
            super::super::WorthQueryApplicationObservedFact::Field {
                entity_id,
                kind,
                locator: observed_locator,
                value: observed_value,
            } if *kind == expected_kind
                && *observed_locator == locator
                && *observed_value == encoded =>
            {
                Some(*entity_id)
            }
            _ => None,
        });
        let Some(entity_id) = matches.next() else {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget,
                field.entity(),
            ));
        };
        if matches.next().is_some() {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget,
                field.entity(),
            ));
        }
        Ok(WorthQueryApplicationEffectEntity {
            reference: EntityReference::Existing(entity_id),
            entity: field.entity().to_owned(),
            created_effect: None,
            program: Arc::clone(&self.program),
            _marker: PhantomData,
        })
    }

    pub fn projected_entity<Entity>(
        &self,
        target: &WorthQueryInvariantMutationTarget<Schema, Entity>,
    ) -> Result<WorthQueryApplicationEffectEntity<Schema, Entity>, WorthQueryApplicationAttemptDenial>
    {
        let observed = self
            .read_set
            .facts
            .iter()
            .any(|fact| fact.touches_entity(target.entity_id));
        if !observed {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget,
                target.entity.as_ref(),
            ));
        }
        Ok(WorthQueryApplicationEffectEntity {
            reference: EntityReference::Existing(target.entity_id),
            entity: target.entity.to_string(),
            created_effect: None,
            program: Arc::clone(&self.program),
            _marker: PhantomData,
        })
    }

    pub fn existing_entity<Entity>(
        &self,
        identity: &WorthQueryApplicationEntityIdentity<Schema, Entity>,
    ) -> Result<WorthQueryApplicationEffectEntity<Schema, Entity>, WorthQueryApplicationAttemptDenial>
    {
        let observed = self
            .read_set
            .facts
            .iter()
            .any(|fact| fact.touches_entity(identity.entity_id()));
        if identity.runtime_authority() != self.read_set.admission.runtime_authority()
            || identity.binding_identity() != self.read_set.admission.binding_identity()
            || !observed
        {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget,
                identity.entity_name(),
            ));
        }
        Ok(WorthQueryApplicationEffectEntity {
            reference: EntityReference::Existing(identity.entity_id()),
            entity: identity.entity_name().to_string(),
            created_effect: None,
            program: Arc::clone(&self.program),
            _marker: PhantomData,
        })
    }
}
