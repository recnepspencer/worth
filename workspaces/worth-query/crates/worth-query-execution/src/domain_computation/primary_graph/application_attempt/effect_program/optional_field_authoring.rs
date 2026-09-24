use std::collections::BTreeMap;

use worth_foundational::facade::PortableAspectContractBasis;
use worth_query_installation::facade::{
    ApplicationFieldRef, ApplicationFieldUnit, ApplicationOperationProgramTarget,
    ApplicationScalarValueBinding, OperationWrites, OptionalApplicationFieldValue,
    WritableCapability,
};
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::transactions::EntityReference;

use super::{
    CandidateItemKind, WorthQueryApplicationEffectEntity,
    WorthQueryApplicationEffectProgramBuilder, WorthQueryApplicationOptionalFieldWrite,
    WorthQueryApplicationRealizedEffect,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
};

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgramBuilder<Schema, Operation, Input, Scope>
{
    /// Authors an exact presence change for an installed optional field.
    /// `Some(value)` preserves even empty or zero values; `None` clears the
    /// field. The field must remain an admitted decision read so absence or
    /// value drift participates in compare-and-commit currentness.
    pub fn write_optional_field<Entity, Aspect, Field, Value, Write, Equality, Unit>(
        &mut self,
        target: &WorthQueryApplicationEffectEntity<Schema, Entity>,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
        value: Option<Value>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial>
    where
        Field: OperationWrites<Operation> + OptionalApplicationFieldValue<Value = Value>,
        Write: WritableCapability,
        Unit: ApplicationFieldUnit,
    {
        let value = value
            .map(|value| Field::Binding::encode(&value))
            .transpose()
            .map_err(|_| {
                super::denial(
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
        let matching_effect = position.map(|position| &self.effects[position]);
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
        let promotion_contract_bytes = ordinary_write_contract_representation_bytes(
            &self.layout,
            matching_effect,
            field.entity(),
        )?;
        let retains_locator_and_contract = replaced_representation_bytes.is_none();
        let retained_representation_bytes = super::retained_representation::optional_field_entry(
            &locator,
            &contract,
            value.as_ref(),
            retains_locator_and_contract,
        )
        .and_then(|bytes| bytes.checked_add(promotion_contract_bytes))
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
        let write = WorthQueryApplicationOptionalFieldWrite { contract, value };
        if let Some(position) = position {
            promote_ordinary_writes(&self.layout, &mut self.effects[position], field.entity())?;
        }
        let first_write = position.is_none();
        let position = record_write(
            &mut self.effects,
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
        Ok(())
    }
}

fn ordinary_write_contract_representation_bytes(
    layout: &super::super::super::schema_layout::WorthQueryPrimaryGraphLayout,
    effect: Option<&WorthQueryApplicationRealizedEffect>,
    entity: &str,
) -> Result<usize, WorthQueryApplicationAttemptDenial> {
    let Some(WorthQueryApplicationRealizedEffect::UpdateEntity { fields, .. }) = effect else {
        return Ok(0);
    };
    fields.keys().try_fold(0_usize, |total, locator| {
        let contract = layout
            .aspect_contract(entity, locator.aspect().aspect_key())
            .map(PortableAspectContractBasis::from_contract)
            .ok_or_else(|| {
                denial(
                    WorthQueryApplicationAttemptDenialKind::UndeclaredEffect,
                    format!("{:?}", locator.field_path()),
                )
            })?;
        total
            .checked_add(super::retained_representation::contract_width(&contract))
            .ok_or_else(|| {
                denial(
                    WorthQueryApplicationAttemptDenialKind::CandidateReservationExceeded,
                    entity,
                )
            })
    })
}

fn promote_ordinary_writes(
    layout: &super::super::super::schema_layout::WorthQueryPrimaryGraphLayout,
    effect: &mut WorthQueryApplicationRealizedEffect,
    entity: &str,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    let WorthQueryApplicationRealizedEffect::UpdateEntity {
        entity_id, fields, ..
    } = effect
    else {
        return Ok(());
    };
    let entity_id = *entity_id;
    let fields = fields
        .iter()
        .map(|(locator, value)| {
            let contract = layout
                .aspect_contract(entity, locator.aspect().aspect_key())
                .map(PortableAspectContractBasis::from_contract)
                .ok_or_else(|| {
                    denial(
                        WorthQueryApplicationAttemptDenialKind::UndeclaredEffect,
                        format!("{:?}", locator.field_path()),
                    )
                })?;
            Ok((
                locator.clone(),
                WorthQueryApplicationOptionalFieldWrite {
                    contract,
                    value: Some(value.clone()),
                },
            ))
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    *effect = WorthQueryApplicationRealizedEffect::PatchOptionalEntityFields {
        entity: entity.to_owned(),
        entity_id,
        fields,
    };
    Ok(())
}

fn record_write(
    effects: &mut Vec<WorthQueryApplicationRealizedEffect>,
    position: Option<usize>,
    entity: &str,
    entity_id: EntityId,
    locator: worth_foundational::facade::AspectFieldLocator,
    write: WorthQueryApplicationOptionalFieldWrite,
) -> usize {
    match position.map(|position| &mut effects[position]) {
        Some(WorthQueryApplicationRealizedEffect::PatchOptionalEntityFields { fields, .. }) => {
            fields.insert(locator, write);
            position.expect("existing field effect has a position")
        }
        Some(_) => unreachable!("optional write promotes an ordinary field effect first"),
        None => {
            let position = effects.len();
            effects.push(
                WorthQueryApplicationRealizedEffect::PatchOptionalEntityFields {
                    entity: entity.to_owned(),
                    entity_id,
                    fields: BTreeMap::from([(locator, write)]),
                },
            );
            position
        }
    }
}

fn denial(
    kind: WorthQueryApplicationAttemptDenialKind,
    subject: impl Into<String>,
) -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(kind, subject)
}
