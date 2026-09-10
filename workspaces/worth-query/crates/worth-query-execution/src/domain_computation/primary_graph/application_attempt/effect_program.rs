mod conditional_definition;
mod emission;
mod model;
mod optional_field_authoring;
mod ordinary_field_authoring;
mod relation_effects;
mod target_admission;

#[cfg(test)]
mod external_payload_tests;
#[cfg(test)]
mod outbox_persistence_tests;

use std::collections::{BTreeMap, BTreeSet};
use std::marker::PhantomData;
use std::sync::Arc;

use worth_foundational::facade::AspectFieldLocator;
use worth_query_declaration::facade::domain_computation::WorthQueryResourceDimension;
use worth_query_installation::facade::{
    ApplicationEntityRef, ApplicationFieldRef, ApplicationFieldUnit,
    ApplicationOperationProgramTarget, OperationCreates, OperationDeletes, OperationWrites,
    TypedApplicationValue,
};
use worth_relational::facade::transactions::EntityReference;

pub(in crate::domain_computation::primary_graph) use model::{
    WorthQueryAdmittedApplicationEmissionBatch, WorthQueryApplicationEmission,
};
pub use model::{
    WorthQueryApplicationEffectEntity, WorthQueryApplicationEffectProgram,
    WorthQueryApplicationEffectProgramBuilder,
};
pub(super) use model::{
    WorthQueryApplicationOptionalFieldWrite, WorthQueryApplicationRealizedEffect,
};

use super::effect_validation::{canonical_key, denial};
use super::read_set::WorthQueryCompleteApplicationReadSet;
use super::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryProjectedApplicationMutation,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationEntityIdentity, WorthQueryApplicationEntityKey,
    WorthQueryInvariantMutationTarget,
};
use target_admission::installed_contract_admits_program_target;

impl<Schema, Operation, Input, Scope>
    WorthQueryCompleteApplicationReadSet<
        Schema,
        Operation,
        Input,
        Scope,
        WorthQueryProjectedApplicationMutation,
    >
{
    /// Root-scoped ordinary reads cannot advance to mutation authoring:
    ///
    /// ```compile_fail
    /// use worth_query_execution::facade::primary_graph::{
    ///     WorthQueryCompleteApplicationReadSet, WorthQueryOrdinaryApplicationRead,
    /// };
    ///
    /// fn ordinary_read_cannot_author_effects<Schema, Operation, Input, Scope>(
    ///     reads: WorthQueryCompleteApplicationReadSet<
    ///         Schema, Operation, Input, Scope, WorthQueryOrdinaryApplicationRead,
    ///     >,
    /// ) {
    ///     let _ = reads.begin_effect_program();
    /// }
    /// ```
    pub fn begin_effect_program(
        self,
    ) -> WorthQueryApplicationEffectProgramBuilder<Schema, Operation, Input, Scope> {
        let layout = Arc::clone(&self.lease.layout);
        let emission_retained_bytes_ceiling = self
            .admission
            .allowed_graph_contract()
            .execution_strategy()
            .expect("installed application operation has exactly one execution strategy")
            .envelope()
            .resource_ceiling(WorthQueryResourceDimension::RetainedBytes);
        WorthQueryApplicationEffectProgramBuilder {
            read_set: self,
            layout,
            program: Arc::new(()),
            effects: Vec::new(),
            keys: BTreeSet::new(),
            emission_retained_bytes: 0,
            emission_retained_bytes_ceiling,
            conditional_definition: None,
        }
    }
}

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgramBuilder<Schema, Operation, Input, Scope>
{
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

    pub fn create_entity<Entity>(
        &mut self,
        entity: ApplicationEntityRef<Schema, Entity>,
        key: WorthQueryApplicationEntityKey<Schema, Entity>,
    ) -> Result<WorthQueryApplicationEffectEntity<Schema, Entity>, WorthQueryApplicationAttemptDenial>
    where
        Entity: OperationCreates<Operation>,
    {
        let target = ApplicationOperationProgramTarget::Create {
            entity: entity.name().to_string(),
        };
        self.admit_program_target(&target)?;
        let kind = self.layout.entity_kind(entity.name()).ok_or_else(|| {
            denial(
                WorthQueryApplicationAttemptDenialKind::UndeclaredEffect,
                entity.name(),
            )
        })?;
        let key = key.into_string();
        if !self.keys.insert((kind, key.clone())) {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::DuplicateEffectKey,
                entity.name(),
            ));
        }
        let reference =
            EntityReference::Created(worth_relational::facade::transactions::CreatedEntityRef {
                partition_id: worth_relational::facade::identity::PartitionId::main(),
                kind_id: kind,
                client_key: worth_relational::facade::symbols::ClientKey::raw(key.clone()),
            });
        let created_effect = self.effects.len();
        self.effects
            .push(WorthQueryApplicationRealizedEffect::CreateEntity {
                kind,
                key,
                fields: BTreeMap::new(),
            });
        Ok(WorthQueryApplicationEffectEntity {
            reference,
            entity: entity.name().to_string(),
            created_effect: Some(created_effect),
            program: Arc::clone(&self.program),
            _marker: PhantomData,
        })
    }

    pub fn initialize_field<Entity, Aspect, Field, Value, Write, Equality, Unit>(
        &mut self,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
        value: Value,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Entity: OperationCreates<Operation>,
        Field: OperationWrites<Operation>,
        Value: TypedApplicationValue,
        Unit: ApplicationFieldUnit,
    {
        self.validate_target(target, field.entity())?;
        self.admit_program_target(&ApplicationOperationProgramTarget::Write {
            entity: field.entity().to_string(),
            aspect: field.aspect().to_string(),
            field: field.field().to_string(),
        })?;
        let locator = self.field_locator(field.entity(), field.aspect(), field.field())?;
        let Some(WorthQueryApplicationRealizedEffect::CreateEntity { fields, .. }) = target
            .created_effect
            .and_then(|ordinal| self.effects.get_mut(ordinal))
        else {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget,
                field.entity(),
            ));
        };
        fields.insert(locator, value.into_foundational_value());
        Ok(())
    }

    pub fn delete_entity<Entity>(
        &mut self,
        entity: ApplicationEntityRef<Schema, Entity>,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Entity: OperationDeletes<Operation>,
    {
        self.validate_target(target, entity.name())?;
        self.admit_program_target(&ApplicationOperationProgramTarget::Delete {
            entity: entity.name().to_string(),
        })?;
        let EntityReference::Existing(entity_id) = target.reference else {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget,
                entity.name(),
            ));
        };
        self.effects
            .push(WorthQueryApplicationRealizedEffect::DeleteEntity { entity_id });
        Ok(())
    }

    pub fn finish(
        self,
    ) -> Result<
        WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    > {
        self.read_set
            .admission
            .validate_current_authority()
            .map_err(|_| {
                denial(
                    WorthQueryApplicationAttemptDenialKind::CurrentAuthorityDenied,
                    self.read_set.admission.operation(),
                )
            })?;
        Ok(WorthQueryApplicationEffectProgram {
            read_set: self.read_set,
            effects: self.effects,
            emission_retained_bytes: self.emission_retained_bytes,
            emission_retained_bytes_ceiling: self.emission_retained_bytes_ceiling,
            conditional_definition: self.conditional_definition,
        })
    }

    fn validate_target<Entity>(
        &self,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        entity: &str,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        if Arc::ptr_eq(&target.program, &self.program) && target.entity == entity {
            Ok(())
        } else {
            Err(denial(
                WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget,
                entity,
            ))
        }
    }

    fn admit_program_target(
        &self,
        target: &ApplicationOperationProgramTarget,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        let contracts = self.read_set.admission.allowed_graph_contract();
        if installed_contract_admits_program_target(contracts, target) {
            Ok(())
        } else {
            Err(denial(
                WorthQueryApplicationAttemptDenialKind::UndeclaredEffect,
                self.read_set.admission.operation(),
            ))
        }
    }

    fn field_locator(
        &self,
        entity: &str,
        aspect: &str,
        field: &str,
    ) -> Result<AspectFieldLocator, WorthQueryApplicationAttemptDenial> {
        self.layout
            .field_locator(entity, aspect, field)
            .cloned()
            .ok_or_else(|| {
                denial(
                    WorthQueryApplicationAttemptDenialKind::UndeclaredEffect,
                    field,
                )
            })
    }
}
