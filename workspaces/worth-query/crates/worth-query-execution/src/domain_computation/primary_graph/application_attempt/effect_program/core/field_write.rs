//! Existing-field writes retain exact observation and parent-conflict requirements.
use super::field_patch::{promote_ordinary_writes, record_write};
use super::*;
use std::collections::BTreeMap;
use worth_foundational::facade::PortableAspectContractBasis;
impl EffectCore<'_> {
    pub(in crate::domain_computation::primary_graph::application_attempt::effect_program) fn write_field(
        &mut self,
        target: Target<'_>,
        field: ColumnIdentity,
        encoder: &dyn FieldEncoder,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        let column = self.layout.record_column(field)?;
        if !column.writable() {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::UndeclaredEffect,
                column.path(),
            ));
        }
        let value = encoder.encode().map_err(|_| {
            denial(
                WorthQueryApplicationAttemptDenialKind::InvalidEffectValue,
                column.path(),
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
                *entity_id, locator, Some(value),
            );
        }
        self.write_required_value(
            &worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus::new(
                field.entity(),
                field.aspect(),
                field.field(),
            ),
            *entity_id,
            locator,
            std::borrow::Cow::Owned(value),
        )
    }
    pub(in crate::domain_computation::primary_graph::application_attempt::effect_program) fn write_optional_field(
        &mut self,
        target: Target<'_>,
        field: ColumnIdentity,
        encoder: Option<&dyn FieldEncoder>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        let column = self.layout.record_column(field)?;
        if !column.writable()
            || column.presence()
                != worth_query_installation::facade::ApplicationFieldPresence::Optional
        {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::UndeclaredEffect,
                column.path(),
            ));
        }
        let value = encoder
            .map(|encoder| encoder.encode())
            .transpose()
            .map_err(|_| {
                denial(
                    WorthQueryApplicationAttemptDenialKind::InvalidEffectValue,
                    column.path(),
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
        self.write_optional_value(
            &ApplicationFieldBindingLocus::new(field.entity(), field.aspect(), field.field()),
            *entity_id,
            locator,
            value,
        )
    }
    pub(in crate::domain_computation::primary_graph::application_attempt::effect_program) fn write_required_value(
        &mut self,
        field: &ApplicationFieldBindingLocus,
        entity_id: EntityId,
        locator: AspectFieldLocator,
        value: std::borrow::Cow<'_, AspectValue>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        self.require_parent_not_removed(field.entity(), entity_id, &locator)?;
        self.require_field_observation(entity_id, &locator)?;
        let position = self
            .field_write_positions
            .position(field.entity(), entity_id);
        let matching_effect = position.map(|position| &self.effects[position]);
        // Required fields stored in an optional patch retain the same contract
        // as optional entries. Resolve it before retaining the value.
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
        self.reserve_effects(usize::from(position.is_none()))?;
        let value = value.into_owned();
        match position.map(|position| &mut self.effects[position]) {
            Some(WorthQueryApplicationRealizedEffect::PatchOptionalEntityFields {
                fields, ..
            }) => {
                let contract = patch_contract.expect("existing patch contract prepared");
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
    pub(in crate::domain_computation::primary_graph::application_attempt::effect_program) fn write_optional_value(
        &mut self,
        field: &ApplicationFieldBindingLocus,
        entity_id: EntityId,
        locator: AspectFieldLocator,
        value: Option<AspectValue>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        let present = value.is_some();
        self.require_parent_not_removed(field.entity(), entity_id, &locator)?;
        self.require_field_observation(entity_id, &locator)?;
        let contract = self
            .layout
            .aspect_contract(field.entity(), locator.aspect().aspect_key())
            .map(PortableAspectContractBasis::from_contract)
            .ok_or_else(|| {
                denial(
                    WorthQueryApplicationAttemptDenialKind::UndeclaredEffect,
                    field.field(),
                )
            })?;
        let position = self
            .field_write_positions
            .position(field.entity(), entity_id);
        self.reserve_effects(usize::from(position.is_none()))?;
        let write = WorthQueryApplicationOptionalFieldWrite { contract, value };
        if let Some(position) = position {
            promote_ordinary_writes(self.layout, &mut self.effects[position], field.entity())?;
        }
        let first_write = position.is_none();
        let position = record_write(
            self.effects,
            position,
            field.entity(),
            entity_id,
            locator,
            write,
        );
        if first_write {
            self.field_write_positions
                .remember(field.entity(), entity_id, position);
        }
        if present {
            self.retain_record_presence(field, position)?;
        }
        Ok(())
    }
}

impl EffectCore<'_> {
    pub(in crate::domain_computation::primary_graph::application_attempt::effect_program) fn retain_record_presence(
        &mut self,
        field: &ApplicationFieldBindingLocus,
        position: usize,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        let Some(locator) = self.layout.record_presence_locator(field).cloned() else {
            return Ok(());
        };
        let contract = self
            .layout
            .aspect_contract(field.entity(), locator.aspect().aspect_key())
            .map(PortableAspectContractBasis::from_contract)
            .ok_or_else(|| {
                WorthQueryApplicationAttemptDenial::new(
                    WorthQueryApplicationAttemptDenialKind::UndeclaredEffect,
                    field.field(),
                )
            })?;
        let Some(WorthQueryApplicationRealizedEffect::PatchOptionalEntityFields { fields, .. }) =
            self.effects.get_mut(position)
        else {
            return Err(WorthQueryApplicationAttemptDenial::new(
                WorthQueryApplicationAttemptDenialKind::ConflictingEffectStep,
                field.field(),
            ));
        };
        fields.insert(
            locator,
            WorthQueryApplicationOptionalFieldWrite {
                contract,
                value: Some(AspectValue::Bool(true)),
            },
        );
        Ok(())
    }
}
