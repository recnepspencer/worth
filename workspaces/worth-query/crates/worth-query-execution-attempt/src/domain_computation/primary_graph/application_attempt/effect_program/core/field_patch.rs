//! Promote and retain optional patches without changing field contract meaning.
use super::*;
use std::collections::BTreeMap;
use worth_foundational::facade::PortableAspectContractBasis;
pub(super) fn promote_ordinary_writes(
    layout: &super::super::super::super::schema_layout::WorthQueryPrimaryGraphLayout,
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
        .collect::<Result<BTreeMap<_, _>, WorthQueryApplicationAttemptDenial>>()?;
    *effect = WorthQueryApplicationRealizedEffect::PatchOptionalEntityFields {
        entity: entity.to_owned(),
        entity_id,
        fields,
    };
    Ok(())
}

pub(super) fn record_write(
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
