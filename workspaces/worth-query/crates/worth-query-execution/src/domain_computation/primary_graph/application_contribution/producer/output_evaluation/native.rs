//! Exact completed Native field reads, never DTO equality or guessed revisions.
use super::*;
use crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact as Fact;
use worth_foundational::facade::{
    AspectValue, CanonicalFieldPath, ContractValidatedAspectValueView, FieldKey,
};
use worth_relational::facade::runtime::RelationalFieldPresence;
use WorthQueryProducerEvaluationIneligibility as Refusal;

pub(super) struct ObservedOutput {
    pub(super) role: usize,
    pub(super) entity: EntityId,
    pub(super) kind: KindId,
    pub(super) before: u64,
    pub(super) after: u64,
    pub(super) revision: RelationalFieldRevision,
}
pub(super) fn prepare(
    execution: &ExecutionRequest<'_, '_>,
    field: &InstalledProducerOutputEquivalence,
    role: &str,
    basis: &NativeOutputBasis<'_>,
    same_publication: bool,
) -> Result<Result<ObservedOutput, Refusal>, WorthQueryOutputDemandDenial> {
    let NativeOutputBasis {
        receipt,
        readiness,
        lowering,
        ordinal,
        layout,
    } = *basis;
    consult(execution)?;
    if !same_publication {
        return Ok(Err(Refusal::ReadinessPublicationMismatch));
    }
    let locus = &field.field.binding.locus;
    let mut matches = receipt.output_correspondence().native_witness_roles().enumerate()
        .filter(|(_, (name,posture,entity_name,_))|
            *name == role && *entity_name == locus.entity()
            && *posture == crate::domain_computation::primary_graph::WorthQueryApplicationOutputPosture::Preserve);
    let Some((role, (_, _, _, entity))) = matches.next() else {
        return Ok(Err(Refusal::OutputRoleUnavailable));
    };
    if matches.next().is_some() {
        return Ok(Err(Refusal::OutputRoleUnavailable));
    }
    let kind = layout
        .entity_kind(locus.entity())
        .expect("installed output field kind");
    let locator = layout
        .field_locator(locus.entity(), locus.aspect(), locus.field())
        .expect("installed output field");
    let expected_record = worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts::entity(
        entity.partition_id.0,
        entity.local_slot.0,
        entity.generation.0,
    );
    let Some(dependency) = lowering
        .semantic_dependencies()
        .find(|dependency| dependency.dependency_ordinal() == ordinal)
    else {
        return Ok(Err(Refusal::ReadinessFieldMismatch));
    };
    if !dependency_matches_output(dependency, field, expected_record)
        || lowering
            .validate_signal_decision_contract(readiness.signal())
            .is_err()
    {
        return Ok(Err(Refusal::ReadinessFieldMismatch));
    }
    let same_field = |actual: &worth_foundational::facade::AspectFieldLocator| {
        actual.aspect().aspect_key() == locator.aspect().aspect_key()
            && actual.field_path() == locator.field_path()
    };
    let Some(inputs) = receipt.performed_input_facts() else {
        return Ok(Err(Refusal::BeforeFieldUnavailable));
    };
    let mut before = None;
    for fact in inputs.iter() {
        consult(execution)?;
        if let Fact::Field {
            entity_id,
            kind: actual_kind,
            locator,
            value: AspectValue::UInt64(value),
        } = fact
            && *entity_id == entity
            && *actual_kind == kind
            && same_field(locator)
        {
            if before.is_some_and(|previous| previous != *value) {
                return Ok(Err(Refusal::ConflictingBeforeField));
            }
            before = Some(*value);
        }
    }
    let Some(before) = before else {
        return Ok(Err(Refusal::BeforeFieldUnavailable));
    };
    let Some(Ok(after_facts)) = receipt.performed_output_fields() else {
        return Ok(Err(Refusal::NativeAfterUnavailable));
    };
    let mut revision = None;
    for fact in after_facts.iter() {
        consult(execution)?;
        if let Fact::SourceFieldRevision {
            entity_id,
            locator,
            native_revision: Some(actual),
        } = fact
            && *entity_id == entity
            && same_field(locator)
        {
            if revision.is_some_and(|previous| previous != *actual) {
                return Ok(Err(Refusal::ConflictingNativeAfter));
            }
            revision = Some(*actual);
        }
    }
    let Some(revision) = revision else {
        return Ok(Err(Refusal::NativeAfterUnavailable));
    };
    if revision.presence() != RelationalFieldPresence::Present {
        return Ok(Err(Refusal::NativeAfterAbsent));
    }
    let Some(observation) = readiness
        .semantic_observations()
        .iter()
        .find(|value| value.dependency_ordinal() == ordinal)
    else {
        return Ok(Err(Refusal::ReadinessFieldMismatch));
    };
    let Some(artifact) = observation.current() else {
        return Ok(Err(Refusal::ReadinessFieldMismatch));
    };
    let field_key = FieldKey::new(locus.field()).expect("declared output field key");
    let mask = observation.projection_mask();
    if artifact.payload().contract() != &field.field.aspect
        || (!mask.is_whole_aspect()
            && !mask
                .paths()
                .iter()
                .any(|path| path == &CanonicalFieldPath::single(field_key.clone())))
    {
        return Ok(Err(Refusal::ReadinessFieldMismatch));
    }
    let value = match artifact.payload().view() {
        ContractValidatedAspectValueView::Scalar(value) => Some(value),
        ContractValidatedAspectValueView::Struct(value) => value.get(&field_key),
    };
    let Some(AspectValue::UInt64(after)) = value else {
        return Ok(Err(Refusal::ReadinessFieldMismatch));
    };
    Ok(Ok(ObservedOutput {
        role,
        entity,
        kind,
        before,
        after: *after,
        revision,
    }))
}

/// Association only: the caller separately validates and consumes real readiness evidence.
pub(in super::super) fn dependency_matches_output(
    dependency: &worth_runtime_bridge::facade::BridgeSemanticDependencyCandidate,
    field: &InstalledProducerOutputEquivalence,
    record: worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts,
) -> bool {
    let exact_record = match dependency.observation_record_identity() {
        Some(fixed) => fixed == record,
        None => matches!(
            dependency.locality(),
            worth_runtime_bridge::facade::BridgeSemanticLocality::ManagedSourceRecord
        ),
    };
    exact_record
        && dependency.contract() == &field.field.aspect
        && dependency.binding() == &field.field.native_binding
}
