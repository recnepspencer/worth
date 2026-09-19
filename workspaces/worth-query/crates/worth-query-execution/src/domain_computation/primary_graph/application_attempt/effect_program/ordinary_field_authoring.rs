use std::collections::BTreeMap;

use worth_query_installation::facade::{
    ApplicationFieldRef, ApplicationFieldUnit, ApplicationOperationProgramTarget,
    ApplicationScalarValueBinding, DeclaredApplicationFieldValue, OperationWrites,
    WritableCapability,
};
use worth_relational::facade::transactions::EntityReference;

use super::{
    denial, CandidateItemKind, WorthQueryApplicationEffectEntity,
    WorthQueryApplicationEffectProgramBuilder, WorthQueryApplicationOptionalFieldWrite,
    WorthQueryApplicationRealizedEffect,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
};

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgramBuilder<Schema, Operation, Input, Scope>
{
    pub fn write_field<Entity, Aspect, Field, Value, Write, Equality, Unit>(
        &mut self,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
        value: Value,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Field: OperationWrites<Operation> + DeclaredApplicationFieldValue<Value = Value>,
        Write: WritableCapability,
        Unit: ApplicationFieldUnit,
    {
        let value = Field::Binding::encode(&value).map_err(|_| {
            denial(
                WorthQueryApplicationAttemptDenialKind::InvalidEffectValue,
                field.field(),
            )
        })?;
        self.validate_target(target, field.entity())?;
        self.admit_program_target(&ApplicationOperationProgramTarget::Write {
            entity: field.entity().to_string(),
            aspect: field.aspect().to_string(),
            field: field.field().to_string(),
        })?;
        let EntityReference::Existing(entity_id) = target.reference else {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::ForeignEffectTarget,
                field.field(),
            ));
        };
        let locator = self.field_locator(field.entity(), field.aspect(), field.field())?;
        let matching_effect = self.effects.iter().find(|effect| {
            matches!(
                effect,
                WorthQueryApplicationRealizedEffect::UpdateEntity {
                    entity,
                    entity_id: candidate,
                    ..
                } | WorthQueryApplicationRealizedEffect::PatchOptionalEntityFields {
                    entity,
                    entity_id: candidate,
                    ..
                } if entity == field.entity() && *candidate == entity_id
            )
        });
        let replaced_representation_bytes = matching_effect.and_then(|effect| match effect {
            WorthQueryApplicationRealizedEffect::UpdateEntity { fields, .. } => fields
                .get(&locator)
                .map(super::retained_representation::value),
            WorthQueryApplicationRealizedEffect::PatchOptionalEntityFields { fields, .. } => fields
                .get(&locator)
                .and_then(|write| write.value.as_ref())
                .map(super::retained_representation::value),
            _ => None,
        });
        let retains_locator = replaced_representation_bytes.is_none();
        let retained_representation_bytes =
            super::retained_representation::field_entry(&locator, &value, retains_locator)
                .and_then(|bytes| {
                    bytes.checked_add(if matching_effect.is_none() {
                        field.entity().len()
                    } else {
                        0
                    })
                })
                .ok_or_else(|| {
                    denial(
                        WorthQueryApplicationAttemptDenialKind::CandidateReservationExceeded,
                        field.field(),
                    )
                })?;
        self.charge_candidate_representation(
            CandidateItemKind::Write,
            retained_representation_bytes,
            replaced_representation_bytes.unwrap_or(0),
        )?;
        if let Some(WorthQueryApplicationRealizedEffect::PatchOptionalEntityFields {
            fields, ..
        }) = self.effects.iter_mut().find(|effect| {
            matches!(
                effect,
                WorthQueryApplicationRealizedEffect::PatchOptionalEntityFields {
                    entity,
                    entity_id: candidate,
                    ..
                } if entity == field.entity() && *candidate == entity_id
            )
        }) {
            let contract = self
                .layout
                .aspect_contract(field.entity(), locator.aspect().aspect_key())
                .map(worth_foundational::facade::PortableAspectContractBasis::from_contract)
                .ok_or_else(|| {
                    denial(
                        WorthQueryApplicationAttemptDenialKind::UndeclaredEffect,
                        field.field(),
                    )
                })?;
            fields.insert(
                locator,
                WorthQueryApplicationOptionalFieldWrite {
                    contract,
                    value: Some(value),
                },
            );
            return Ok(());
        }
        match self.effects.iter_mut().find(|effect| {
            matches!(
                effect,
                WorthQueryApplicationRealizedEffect::UpdateEntity {
                    entity,
                    entity_id: candidate,
                    ..
                } if entity == field.entity() && *candidate == entity_id
            )
        }) {
            Some(WorthQueryApplicationRealizedEffect::UpdateEntity {
                entity,
                entity_id: candidate,
                fields,
            }) if entity == field.entity() && *candidate == entity_id => {
                fields.insert(locator, value);
            }
            _ => self
                .effects
                .push(WorthQueryApplicationRealizedEffect::UpdateEntity {
                    entity: field.entity().to_string(),
                    entity_id,
                    fields: BTreeMap::from([(locator, value)]),
                }),
        }
        Ok(())
    }
}
