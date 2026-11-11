//! Resolve original endpoint names; never format an unresolved symbol.
use crate::symbols::data::{ClientKey, StringInterner};
use crate::transactions::data::{
    EntityReference, RelationalCommitSemanticResultRefusal as Refusal,
};
use crate::validation::data::{InvariantVerdict, InvariantViolationFields as Fields};

fn canonicalize_entity(
    reference: &mut EntityReference,
    symbols: &StringInterner,
) -> Result<(), Refusal> {
    if let EntityReference::Created(created) = reference {
        let text = created
            .client_key
            .resolve_with_interner(symbols)
            .ok_or(Refusal::MissingClientKeyText)?
            .to_owned();
        created.client_key = ClientKey::raw(text);
    }
    Ok(())
}

pub(in super::super) fn canonicalize(
    verdict: &mut InvariantVerdict,
    symbols: &StringInterner,
) -> Result<(), Refusal> {
    let violation = match verdict {
        InvariantVerdict::Pass | InvariantVerdict::NotApplicable => return Ok(()),
        InvariantVerdict::Advisory { violation, .. } | InvariantVerdict::Violation(violation) => {
            violation
        }
    };
    match &mut violation.fields {
        Fields::RelationEndpointKindMismatch { source, target, .. }
        | Fields::RelationEndpointKindSelfEdge { source, target, .. }
        | Fields::RelationCardinalityPair { source, target, .. }
        | Fields::RelationUniqueness { source, target, .. }
        | Fields::RelationSymmetry { source, target, .. }
        | Fields::PartitionIsolation { source, target, .. }
        | Fields::Acyclicity { source, target, .. } => {
            canonicalize_entity(source, symbols)?;
            canonicalize_entity(target, symbols)?;
        }
        Fields::RelationCardinalityEndpoint { entity_id, .. } => {
            canonicalize_entity(entity_id, symbols)?
        }
        Fields::ConnectivityMinimum { source, .. } => canonicalize_entity(source, symbols)?,
        Fields::UniqueEntityField { value, .. } => {
            if let worth_foundational::facade::AspectValue::String(
                worth_foundational::facade::InternedString::Symbol(symbol),
            ) = value
            {
                let text = symbols
                    .resolve(crate::symbols::data::Symbol(symbol.0))
                    .ok_or(Refusal::MissingClientKeyText)?
                    .to_owned();
                *value = worth_foundational::facade::AspectValue::String(
                    worth_foundational::facade::InternedString::Raw(text),
                );
            }
        }
        Fields::None
        | Fields::MergedIntentLimit { .. }
        | Fields::SnapshotEntityLimit { .. }
        | Fields::SidecarConsistency { .. }
        | Fields::RelationEndpointKindCrossContext { .. }
        | Fields::RelationEndpointDeletionIntegrity { .. }
        | Fields::StorageInconsistency { .. }
        | Fields::CustomInvariantFailure { .. }
        | Fields::CustomInvariantViolation { .. }
        | Fields::DerivedRelationGraph { .. } => {}
    }
    Ok(())
}

/// Recorded validation never needs a process-local dictionary to interpret a
/// create name or an observed string value.
pub(in super::super) fn is_canonical(verdict: &InvariantVerdict) -> bool {
    let violation = match verdict {
        InvariantVerdict::Pass | InvariantVerdict::NotApplicable => return true,
        InvariantVerdict::Advisory { violation, .. } | InvariantVerdict::Violation(violation) => {
            violation
        }
    };
    let entity = |reference: &EntityReference| match reference {
        EntityReference::Existing(_) => true,
        EntityReference::Created(created) => created.client_key.as_raw_str().is_some(),
    };
    match &violation.fields {
        Fields::RelationEndpointKindMismatch { source, target, .. }
        | Fields::RelationEndpointKindSelfEdge { source, target, .. }
        | Fields::RelationCardinalityPair { source, target, .. }
        | Fields::RelationUniqueness { source, target, .. }
        | Fields::RelationSymmetry { source, target, .. }
        | Fields::PartitionIsolation { source, target, .. }
        | Fields::Acyclicity { source, target, .. } => entity(source) && entity(target),
        Fields::RelationCardinalityEndpoint { entity_id, .. } => entity(entity_id),
        Fields::ConnectivityMinimum { source, .. } => entity(source),
        Fields::UniqueEntityField { value, .. } => !matches!(
            value,
            worth_foundational::facade::AspectValue::String(
                worth_foundational::facade::InternedString::Symbol(_)
            )
        ),
        Fields::None
        | Fields::MergedIntentLimit { .. }
        | Fields::SnapshotEntityLimit { .. }
        | Fields::SidecarConsistency { .. }
        | Fields::RelationEndpointKindCrossContext { .. }
        | Fields::RelationEndpointDeletionIntegrity { .. }
        | Fields::StorageInconsistency { .. }
        | Fields::CustomInvariantFailure { .. }
        | Fields::CustomInvariantViolation { .. }
        | Fields::DerivedRelationGraph { .. } => true,
    }
}
