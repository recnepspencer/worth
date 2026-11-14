//! A versioned sparse capture must match the installed original role and demand.
use super::*;
use crate::domain_computation::primary_graph::retained_correction::record::MaterialCapture;
use worth_query_installation::facade::InstalledCorrectionMechanism;
use worth_relational::facade::transactions::RecordRef;

pub(super) fn validate(
    capture: &MaterialCapture,
    aftermath: &WorthQueryInstalledAftermathContract,
    correspondence: &WorthQueryApplicationOutputCorrespondence,
) -> Result<(), Denial> {
    let demand = aftermath.mechanism().and_then(|mechanism| match mechanism {
        InstalledCorrectionMechanism::RecordedInverse(inverse) => Some(inverse.preimage_demand()),
        _ => None,
    });
    let expected = demand.and_then(|demand| demand.changed_output());
    let (Some(output), Some(changed)) = (expected, capture.changed_fields.as_ref()) else {
        return if expected.is_none() && capture.changed_fields.is_none() {
            Ok(())
        } else {
            Err(Denial::SemanticMaterialUnavailable)
        };
    };
    if changed.role != output.role() || changed.entity != output.entity() {
        return Err(Denial::UnsupportedSourceMeaning);
    }
    let mut roles = correspondence
        .native_witness_roles()
        .filter(|(name, posture, entity, _)| {
            *name == output.role() && *posture == Posture::Preserve && *entity == output.entity()
        });
    let Some((_, _, _, target)) = roles.next() else {
        return Err(Denial::SemanticMaterialUnavailable);
    };
    if roles.next().is_some() || target != changed.target {
        return Err(Denial::SemanticMaterialUnavailable);
    }
    let demand = demand.ok_or(Denial::UnsupportedSourceMeaning)?;
    let eligible = |entity: &str, aspect: &str, field: &str| {
        demand.loci().iter().any(|locus| {
            locus.entity() == entity && locus.aspect() == aspect && locus.field() == field
        })
    };
    let mut total = 17usize
        .checked_add(output.role().len())
        .and_then(|n| n.checked_add(output.entity().len()));
    let cost = |entity: &str, aspect: &str, field: &str| {
        21usize
            .checked_add(entity.len())
            .and_then(|n| n.checked_add(aspect.len()))
            .and_then(|n| n.checked_add(field.len()))
    };
    let fields = capture
        .preimage
        .as_ref()
        .ok_or(Denial::SemanticMaterialUnavailable)?;
    for field in fields {
        if field.target != RecordRef::Entity(target)
            || !eligible(&field.entity, &field.aspect, &field.field)
        {
            return Err(Denial::SemanticMaterialUnavailable);
        }
        total = total
            .and_then(|n| n.checked_add(cost(&field.entity, &field.aspect, &field.field)?))
            .and_then(|n| n.checked_add(field.encoded_bytes));
    }
    for field in &changed.absences {
        if field.target != RecordRef::Entity(target)
            || !eligible(&field.entity, &field.aspect, &field.field)
        {
            return Err(Denial::SemanticMaterialUnavailable);
        }
        total =
            total.and_then(|n| n.checked_add(cost(&field.entity, &field.aspect, &field.field)?));
    }
    if total.is_none_or(|total| total > demand.maximum_encoded_bytes()) {
        return Err(Denial::SemanticMaterialUnavailable);
    }
    Ok(())
}
