use crate::contracts::authority::commit::preparation::diagnostics::failures::PreparationFailureClass;
use crate::contracts::diagnostics::data::fields::{
    RelationalDiagnosticFields, RelationalDiagnosticValue,
};
use crate::contracts::transactions::data::preparation_strategy_posture::SerialPreparationReason;
use crate::contracts::validation::data::execution::{
    InvariantExecutionPoint, InvariantWitnessBasis, InvariantWitnessKey,
};
use crate::validation::engine::{
    CustomInvariantTraceArtifact, InvariantFailureArtifact, InvariantProofBoundaryArtifact,
};

use super::custom_invariant_provenance_diagnostic_projection::custom_invariant_provenance_diagnostic_value;
use super::invariant_violation_diagnostic_projection::InvariantViolationDiagnosticProjection;

pub fn proof_boundary_trace_diagnostic_fields(
    execution_point: InvariantExecutionPoint,
    proof_boundary: &InvariantProofBoundaryArtifact,
) -> RelationalDiagnosticFields {
    RelationalDiagnosticFields::from_diagnostic_value(RelationalDiagnosticValue::object([
        (
            "execution_point",
            RelationalDiagnosticValue::string(execution_point.diagnostic_label()),
        ),
        (
            "proof_boundary",
            proof_boundary_diagnostic_value(proof_boundary),
        ),
    ]))
}

pub fn failure_diagnostic_fields(
    artifact: &InvariantFailureArtifact,
) -> RelationalDiagnosticFields {
    RelationalDiagnosticFields::from_diagnostic_value(RelationalDiagnosticValue::object([
        (
            "execution_point",
            RelationalDiagnosticValue::string(artifact.execution_point().diagnostic_label()),
        ),
        (
            "failure_effect",
            RelationalDiagnosticValue::string(artifact.failure_effect().diagnostic_label()),
        ),
        ("witness", witness_diagnostic_value(artifact.witness())),
        (
            "proof_boundary",
            RelationalDiagnosticValue::optional(
                artifact
                    .proof_boundary()
                    .map(proof_boundary_diagnostic_value),
            ),
        ),
        (
            "violation",
            InvariantViolationDiagnosticProjection::from_fields(artifact.violation())
                .to_diagnostic_value(),
        ),
        (
            "custom_provenance",
            artifact
                .custom_provenance()
                .map(custom_invariant_provenance_diagnostic_value)
                .unwrap_or(RelationalDiagnosticValue::Null),
        ),
    ]))
}

fn witness_diagnostic_value(witness: &InvariantWitnessKey) -> RelationalDiagnosticValue {
    let basis = match witness.basis() {
        InvariantWitnessBasis::StringOnly => RelationalDiagnosticValue::object([(
            "basis_kind",
            RelationalDiagnosticValue::string("string_only"),
        )]),
        InvariantWitnessBasis::Pass => RelationalDiagnosticValue::object([(
            "basis_kind",
            RelationalDiagnosticValue::string("pass"),
        )]),
        InvariantWitnessBasis::UniqueEntityAspectField {
            field_locator,
            value,
            field_locator_canonical_bytes,
            value_canonical_bytes,
        } => RelationalDiagnosticValue::object([
            (
                "basis_kind",
                RelationalDiagnosticValue::string("unique_entity_aspect_field"),
            ),
            (
                "field_locator",
                RelationalDiagnosticValue::AspectFieldLocator(field_locator.clone()),
            ),
            (
                "value",
                RelationalDiagnosticValue::AspectValue(value.clone()),
            ),
            (
                "field_locator_canonical_bytes",
                RelationalDiagnosticValue::CanonicalBytes(field_locator_canonical_bytes.clone()),
            ),
            (
                "value_canonical_bytes",
                RelationalDiagnosticValue::CanonicalBytes(value_canonical_bytes.clone()),
            ),
        ]),
    };
    RelationalDiagnosticValue::object([
        (
            "presentation_key",
            RelationalDiagnosticValue::string(witness.as_str()),
        ),
        ("basis", basis),
    ])
}

pub fn custom_trace_diagnostic_fields(
    artifact: &CustomInvariantTraceArtifact,
) -> RelationalDiagnosticFields {
    RelationalDiagnosticFields::from_diagnostic_value(RelationalDiagnosticValue::object([
        (
            "rule_id",
            RelationalDiagnosticValue::string(artifact.rule_id()),
        ),
        (
            "semantic_version_major",
            RelationalDiagnosticValue::Unsigned(u64::from(artifact.semantic_version_major())),
        ),
        (
            "semantic_version_minor",
            RelationalDiagnosticValue::Unsigned(u64::from(artifact.semantic_version_minor())),
        ),
        (
            "execution_point",
            RelationalDiagnosticValue::string(artifact.execution_point().diagnostic_label()),
        ),
        (
            "verdict",
            RelationalDiagnosticValue::string(artifact.verdict()),
        ),
        (
            "provenance",
            custom_invariant_provenance_diagnostic_value(artifact.provenance()),
        ),
    ]))
}

pub fn serial_preparation_diagnostic_fields(
    execution_point: InvariantExecutionPoint,
    reason: SerialPreparationReason,
) -> RelationalDiagnosticFields {
    RelationalDiagnosticFields::from_diagnostic_value(RelationalDiagnosticValue::object([
        (
            "execution_point",
            RelationalDiagnosticValue::string(execution_point.diagnostic_label()),
        ),
        (
            "reason",
            RelationalDiagnosticValue::string(reason.diagnostic_label()),
        ),
    ]))
}

pub fn preparation_failure_diagnostic_fields(
    execution_point: InvariantExecutionPoint,
    failure: PreparationFailureClass,
) -> RelationalDiagnosticFields {
    RelationalDiagnosticFields::from_diagnostic_value(RelationalDiagnosticValue::object([
        (
            "execution_point",
            RelationalDiagnosticValue::string(execution_point.diagnostic_label()),
        ),
        (
            "failure_class",
            RelationalDiagnosticValue::string(failure.diagnostic_label()),
        ),
    ]))
}

fn proof_boundary_diagnostic_value(
    artifact: &InvariantProofBoundaryArtifact,
) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::object([
        (
            "scope_class",
            RelationalDiagnosticValue::string(artifact.scope_class().diagnostic_label()),
        ),
        (
            "widened_causes",
            RelationalDiagnosticValue::array(
                artifact
                    .widened_causes()
                    .iter()
                    .map(|cause| RelationalDiagnosticValue::string(cause.diagnostic_label())),
            ),
        ),
        (
            "packet_count",
            RelationalDiagnosticValue::unsigned(artifact.packet_count()),
        ),
        (
            "touched_partition_count",
            RelationalDiagnosticValue::unsigned(artifact.touched_partition_count()),
        ),
    ])
}
