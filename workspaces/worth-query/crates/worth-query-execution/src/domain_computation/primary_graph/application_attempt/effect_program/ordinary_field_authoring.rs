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
        if self.layout.field_is_optional(field.entity(), &locator) {
            return self.write_optional_value(
                &worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus::new(
                    field.entity(), field.aspect(), field.field(),
                ),
                entity_id, locator, Some(value),
            );
        }
        self.write_required_value(
            &worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus::new(
                field.entity(),
                field.aspect(),
                field.field(),
            ),
            entity_id,
            locator,
            std::borrow::Cow::Owned(value),
        )
    }

    pub(super) fn write_required_value(
        &mut self,
        field: &worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus,
        entity_id: worth_relational::facade::identity::EntityId,
        locator: worth_foundational::facade::AspectFieldLocator,
        value: std::borrow::Cow<'_, worth_foundational::facade::AspectValue>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        let position = self
            .field_write_positions
            .position(field.entity(), entity_id);
        let matching_effect = position.map(|position| &self.effects[position]);
        let (replaced_representation_bytes, retains_entry) = match matching_effect {
            Some(WorthQueryApplicationRealizedEffect::UpdateEntity { fields, .. }) => {
                fields.get(&locator).map_or((None, true), |value| {
                    (Some(super::retained_representation::value(value)), false)
                })
            }
            Some(WorthQueryApplicationRealizedEffect::PatchOptionalEntityFields {
                fields, ..
            }) => fields.get(&locator).map_or((None, true), |write| {
                (
                    write
                        .value
                        .as_ref()
                        .map(super::retained_representation::value),
                    false,
                )
            }),
            _ => (None, true),
        };
        // Required fields stored in an optional patch retain the same contract
        // as optional entries. Resolve it before changing reservation credit.
        let patch_contract = match matching_effect {
            Some(WorthQueryApplicationRealizedEffect::PatchOptionalEntityFields { .. }) => Some(
                self.layout
                    .aspect_contract(field.entity(), locator.aspect().aspect_key())
                    .map(worth_foundational::facade::PortableAspectContractBasis::from_contract)
                    .ok_or_else(|| {
                        denial(
                            WorthQueryApplicationAttemptDenialKind::UndeclaredEffect,
                            field.field(),
                        )
                    })?,
            ),
            _ => None,
        };
        let entry_bytes = match patch_contract.as_ref() {
            Some(contract) => super::retained_representation::optional_field_entry(
                &locator,
                contract,
                Some(value.as_ref()),
                retains_entry,
            ),
            None => {
                super::retained_representation::field_entry(&locator, value.as_ref(), retains_entry)
            }
        };
        let retained_representation_bytes = entry_bytes
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
        let value = value.into_owned();
        match position.map(|position| &mut self.effects[position]) {
            Some(WorthQueryApplicationRealizedEffect::PatchOptionalEntityFields {
                fields, ..
            }) => {
                let contract =
                    patch_contract.expect("existing patch contract prepared before debit");
                fields.insert(
                    locator,
                    WorthQueryApplicationOptionalFieldWrite {
                        contract,
                        value: Some(value),
                    },
                );
            }
            Some(WorthQueryApplicationRealizedEffect::UpdateEntity { fields, .. }) => {
                fields.insert(locator, value);
            }
            Some(_) => unreachable!("field-write index names only entity-field effects"),
            None => {
                let position = self.effects.len();
                self.effects
                    .push(WorthQueryApplicationRealizedEffect::UpdateEntity {
                        entity: field.entity().to_string(),
                        entity_id,
                        fields: BTreeMap::from([(locator, value)]),
                    });
                self.field_write_positions
                    .remember(field.entity(), entity_id, position);
            }
        }
        Ok(())
    }
}
