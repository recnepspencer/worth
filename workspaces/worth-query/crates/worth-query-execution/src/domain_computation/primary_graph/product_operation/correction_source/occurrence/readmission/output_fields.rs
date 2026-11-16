//! Retained output fields match the installed mode, exact role and vocabulary.
use super::*;
use crate::domain_computation::primary_graph::retained_correction::record::{
    MaterialCapture, RecordedFieldCapture,
};
use worth_query_declaration::facade::application_aftermath::PreImageOutputFieldCapture;
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
    let expected = demand.and_then(|demand| demand.output_fields());
    let (Some(output), Some(recorded)) = (expected, capture.output_fields.as_ref()) else {
        return if expected.is_none() && capture.output_fields.is_none() {
            Ok(())
        } else {
            Err(Denial::SemanticMaterialUnavailable)
        };
    };
    if recorded.role != output.role()
        || recorded.entity != output.entity()
        || recorded.capture != RecordedFieldCapture::from(output.capture())
    {
        return Err(Denial::UnsupportedSourceMeaning);
    }
    let mut roles = correspondence
        .native_witness_roles()
        .filter(|(name, posture, entity, _)| {
            *name == output.role()
                && *posture == output.capture().posture()
                && *entity == output.entity()
        });
    let Some((_, _, _, target)) = roles.next() else {
        return Err(Denial::SemanticMaterialUnavailable);
    };
    if roles.next().is_some() || target != recorded.target {
        return Err(Denial::SemanticMaterialUnavailable);
    }
    let demand = demand.ok_or(Denial::UnsupportedSourceMeaning)?;
    let eligible = |entity: &str, aspect: &str, field: &str| {
        demand.loci().iter().any(|locus| {
            locus.entity() == entity && locus.aspect() == aspect && locus.field() == field
        })
    };
    let mut total = 18usize
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
    let mut captured = std::collections::BTreeSet::new();
    for field in fields {
        if field.target != RecordRef::Entity(target)
            || !eligible(&field.entity, &field.aspect, &field.field)
            || !captured.insert((
                field.entity.as_str(),
                field.aspect.as_str(),
                field.field.as_str(),
            ))
        {
            return Err(Denial::SemanticMaterialUnavailable);
        }
        total = total
            .and_then(|n| n.checked_add(cost(&field.entity, &field.aspect, &field.field)?))
            .and_then(|n| n.checked_add(field.encoded_bytes));
    }
    for field in &recorded.absences {
        if field.target != RecordRef::Entity(target)
            || !eligible(&field.entity, &field.aspect, &field.field)
            || !captured.insert((
                field.entity.as_str(),
                field.aspect.as_str(),
                field.field.as_str(),
            ))
        {
            return Err(Denial::SemanticMaterialUnavailable);
        }
        total =
            total.and_then(|n| n.checked_add(cost(&field.entity, &field.aspect, &field.field)?));
    }
    if output.capture() == PreImageOutputFieldCapture::CompleteRetired
        && demand
            .loci()
            .iter()
            .any(|locus| !captured.contains(&(locus.entity(), locus.aspect(), locus.field())))
    {
        return Err(Denial::SemanticMaterialUnavailable);
    }
    if total.is_none_or(|total| total > demand.maximum_encoded_bytes()) {
        return Err(Denial::SemanticMaterialUnavailable);
    }
    Ok(())
}
