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
    WorthQueryApplicationEffectEntity, WorthQueryApplicationEffectProgramBuilder,
    WorthQueryApplicationOptionalFieldWrite, WorthQueryApplicationRealizedEffect,
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
        let write = WorthQueryApplicationOptionalFieldWrite { contract, value };
        promote_ordinary_writes(&self.layout, &mut self.effects, field.entity(), entity_id)?;
        record_write(&mut self.effects, field.entity(), entity_id, locator, write);
        Ok(())
    }
}

fn promote_ordinary_writes(
    layout: &super::super::super::schema_layout::WorthQueryPrimaryGraphLayout,
    effects: &mut [WorthQueryApplicationRealizedEffect],
    entity: &str,
    entity_id: EntityId,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    let Some(index) = effects.iter().position(|effect| {
        matches!(
            effect,
            WorthQueryApplicationRealizedEffect::UpdateEntity {
                entity: candidate_entity,
                entity_id: candidate,
                ..
            } if candidate_entity == entity && *candidate == entity_id
        )
    }) else {
        return Ok(());
    };
    let WorthQueryApplicationRealizedEffect::UpdateEntity { fields, .. } = &effects[index] else {
        unreachable!("the selected effect is an ordinary entity update");
    };
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
    effects[index] = WorthQueryApplicationRealizedEffect::PatchOptionalEntityFields {
        entity: entity.to_owned(),
        entity_id,
        fields,
    };
    Ok(())
}

fn record_write(
    effects: &mut Vec<WorthQueryApplicationRealizedEffect>,
    entity: &str,
    entity_id: EntityId,
    locator: worth_foundational::facade::AspectFieldLocator,
    write: WorthQueryApplicationOptionalFieldWrite,
) {
    match effects.iter_mut().find(|effect| {
        matches!(
            effect,
            WorthQueryApplicationRealizedEffect::PatchOptionalEntityFields {
                entity: candidate_entity,
                entity_id: candidate,
                ..
            } if candidate_entity == entity && *candidate == entity_id
        )
    }) {
        Some(WorthQueryApplicationRealizedEffect::PatchOptionalEntityFields { fields, .. }) => {
            fields.insert(locator, write);
        }
        _ => effects.push(
            WorthQueryApplicationRealizedEffect::PatchOptionalEntityFields {
                entity: entity.to_owned(),
                entity_id,
                fields: BTreeMap::from([(locator, write)]),
            },
        ),
    }
}

fn denial(
    kind: WorthQueryApplicationAttemptDenialKind,
    subject: impl Into<String>,
) -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(kind, subject)
}
