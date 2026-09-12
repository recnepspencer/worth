use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, AspectValue, CanonicalFieldPath, FieldKey, InternedString,
    LocatorAuthority,
};

use crate::canonical_basis_ready_sequence::canonical_basis_ready_sequence;
use crate::config::data::RelationalExecutionModel;
use crate::diagnostics::data::{DiagnosticCode, RelationalDiagnosticValue};
use crate::identity::data::{EntityId, PartitionId, RelationId, VersionId};
use crate::validation::data::{
    CustomInvariantProvenance, CustomInvariantTraversalSummary, InvariantClass, InvariantCostClass,
    InvariantExecutionPoint, InvariantFailureEffect, InvariantGroupSet, InvariantReportedRule,
    InvariantRule, InvariantVerdict, InvariantViolation, InvariantViolationFields,
    StructuralCountView, TouchedStructuralSet,
};
use crate::validation::engine::{
    InvariantExecutionDisposition, InvariantExecutionMetadata, InvariantExecutionResult,
    InvariantObservationKind, InvariantPlanScopeClass, InvariantProofBoundarySummary,
    InvariantScopeWideningCause,
};

use super::{failure_diagnostic_fields, proof_boundary_trace_diagnostic_fields};

#[test]
fn proof_boundary_trace_projection_is_publication_only_diagnostic_fields() {
    let result = execution_result_with_proof_boundary(Vec::new());
    let proof_boundary = result
        .proof_boundary_artifact()
        .expect("proof boundary artifact");
    let document = proof_boundary_trace_diagnostic_fields(
        result.metadata().execution_point(),
        &proof_boundary,
    );

    assert_eq!(
        document.root(),
        &RelationalDiagnosticValue::object([
            (
                "execution_point",
                RelationalDiagnosticValue::string("commit_boundary"),
            ),
            ("proof_boundary", expected_proof_boundary_value()),
        ])
    );
}

#[test]
fn failure_projection_preserves_typed_invariant_artifact_fields() {
    let violation = none_violation();
    let result = execution_result_with_proof_boundary(vec![check_result_for_violation(
        violation.clone(),
        None,
    )]);
    let failure = invariant_failure_for(violation);
    let document = failure_diagnostic_fields(&result.failure_artifact(&failure));

    assert_eq!(
        document.root(),
        &RelationalDiagnosticValue::object([
            (
                "execution_point",
                RelationalDiagnosticValue::string("commit_boundary"),
            ),
            (
                "failure_effect",
                RelationalDiagnosticValue::string("block_commit"),
            ),
            ("witness", expected_none_violation_witness_value()),
            ("proof_boundary", expected_proof_boundary_value()),
            (
                "violation",
                RelationalDiagnosticValue::object([(
                    "violation_kind",
                    RelationalDiagnosticValue::string("none"),
                )]),
            ),
            ("custom_provenance", RelationalDiagnosticValue::Null),
        ])
    );
}

#[test]
fn failure_projection_emits_aspect_field_diagnostic_fields() {
    let aspect_key = AspectKey::new("profile.email").expect("valid aspect key");
    let field = FieldKey::new("email").expect("valid field key");
    let field_locator = AspectFieldLocator::new(
        LocatorAuthority::Planned,
        aspect_key.clone(),
        CanonicalFieldPath::single(field.clone()),
    );
    let violation = InvariantViolation {
        class: InvariantClass::CommitBoundary,
        code: DiagnosticCode::InvariantViolation,
        detail: "duplicate email".to_string(),
        fields: InvariantViolationFields::UniqueEntityField {
            field_locator: field_locator.clone(),
            value: AspectValue::String(InternedString::Raw("dupe@example.test".to_string())),
        },
    };
    let result = execution_result_with_proof_boundary(vec![check_result_for_violation(
        violation.clone(),
        None,
    )]);
    let failure = invariant_failure_for(violation);
    let document = failure_diagnostic_fields(&result.failure_artifact(&failure));
    let RelationalDiagnosticValue::Object(root) = document.root() else {
        panic!("expected typed diagnostic object root");
    };
    let Some(RelationalDiagnosticValue::Object(violation)) = root.get("violation") else {
        panic!("expected typed violation diagnostic object");
    };
    let Some(RelationalDiagnosticValue::Object(typed_aspect_field)) = violation.get("aspect_field")
    else {
        panic!("expected typed aspect-field diagnostic object");
    };
    let Some(RelationalDiagnosticValue::Object(witness)) = root.get("witness") else {
        panic!("expected typed witness diagnostic object");
    };
    let Some(RelationalDiagnosticValue::Object(witness_basis)) = witness.get("basis") else {
        panic!("expected typed witness basis diagnostic object");
    };
    assert!(matches!(
        typed_aspect_field.get("aspect_key"),
        Some(RelationalDiagnosticValue::AspectKey(key)) if key == &aspect_key
    ));
    assert!(matches!(
        typed_aspect_field.get("field_path"),
        Some(RelationalDiagnosticValue::FieldPath(path)) if path == &CanonicalFieldPath::single(field.clone())
    ));
    assert!(matches!(
        typed_aspect_field.get("diagnostic_mask"),
        Some(RelationalDiagnosticValue::DiagnosticMask(mask)) if !mask.is_whole_aspect()
    ));
    assert!(matches!(
        typed_aspect_field.get("diagnostic_mask_locator"),
        Some(RelationalDiagnosticValue::DiagnosticMaskLocator(locator))
            if locator.aspect_key() == &aspect_key && locator.paths().len() == 1
    ));
    assert!(matches!(
        typed_aspect_field.get("canonical_diagnostic_mask_basis"),
        Some(RelationalDiagnosticValue::CanonicalBasis(basis)) if {
            let ready_sequence = canonical_basis_ready_sequence(basis);
            ready_sequence.entries().len() == 1
                && ready_sequence.version().as_str()
                    == "worth.relational.invariant.diagnostic_mask.v1"
        }
    ));
    assert!(matches!(
        typed_aspect_field.get("value"),
        Some(RelationalDiagnosticValue::AspectValue(value)) if value == &AspectValue::String(InternedString::Raw("dupe@example.test".to_string()))
    ));
    assert_eq!(
        diagnostic_object_field(violation, "violation_kind"),
        &RelationalDiagnosticValue::string("unique_entity_field")
    );
    assert_eq!(
        diagnostic_object_field(witness_basis, "basis_kind"),
        &RelationalDiagnosticValue::string("unique_entity_aspect_field")
    );
    assert!(matches!(
        witness_basis.get("field_locator"),
        Some(RelationalDiagnosticValue::AspectFieldLocator(locator)) if locator == &field_locator
    ));
    assert!(matches!(
        witness_basis.get("value"),
        Some(RelationalDiagnosticValue::AspectValue(value)) if value == &AspectValue::String(InternedString::Raw("dupe@example.test".to_string()))
    ));
    assert!(matches!(
        witness_basis.get("field_locator_canonical_bytes"),
        Some(RelationalDiagnosticValue::CanonicalBytes(bytes)) if !bytes.is_empty()
    ));
}

#[test]
fn failure_projection_preserves_custom_provenance_as_typed_diagnostic_fields() {
    let visible_entity_id = EntityId::new(PartitionId::new(7), 11, 2);
    let visible_relation_id = RelationId::new(PartitionId::new(9), 13, 4);
    let provenance = custom_provenance_with_visible_records(visible_entity_id, visible_relation_id);
    let violation = none_violation();
    let result = execution_result_with_proof_boundary(vec![check_result_for_violation(
        violation.clone(),
        Some(provenance),
    )]);
    let failure = invariant_failure_for(violation);
    let document = failure_diagnostic_fields(&result.failure_artifact(&failure));

    let RelationalDiagnosticValue::Object(root) = document.root() else {
        panic!("expected typed diagnostic object root");
    };
    let Some(RelationalDiagnosticValue::Object(custom_provenance)) = root.get("custom_provenance")
    else {
        panic!("expected typed custom provenance diagnostic object");
    };
    assert!(matches!(
        custom_provenance.get("version_id"),
        Some(RelationalDiagnosticValue::VersionId(version_id)) if *version_id == VersionId::new(20)
    ));
    assert!(matches!(
        custom_provenance.get("current_version_id"),
        Some(RelationalDiagnosticValue::VersionId(version_id)) if *version_id == VersionId::new(21)
    ));
    let Some(RelationalDiagnosticValue::Object(touched)) = custom_provenance.get("touched") else {
        panic!("expected typed touched provenance object");
    };
    assert!(matches!(
        touched.get("visible_entity_ids"),
        Some(RelationalDiagnosticValue::Array(values))
            if values == &vec![RelationalDiagnosticValue::EntityId(visible_entity_id)]
    ));
    assert!(matches!(
        touched.get("visible_relation_ids"),
        Some(RelationalDiagnosticValue::Array(values))
            if values == &vec![RelationalDiagnosticValue::RelationId(visible_relation_id)]
    ));
    assert_eq!(
        diagnostic_object_field(custom_provenance, "observation_kind"),
        &RelationalDiagnosticValue::string("speculative")
    );
}

fn expected_proof_boundary_value() -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::object([
        (
            "scope_class",
            RelationalDiagnosticValue::string("partition_scope"),
        ),
        (
            "widened_causes",
            RelationalDiagnosticValue::array([RelationalDiagnosticValue::string(
                "all_observed_partition_scope",
            )]),
        ),
        ("packet_count", RelationalDiagnosticValue::Unsigned(1)),
        (
            "touched_partition_count",
            RelationalDiagnosticValue::Unsigned(1),
        ),
    ])
}

fn expected_none_violation_witness_value() -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::object([
        (
            "presentation_key",
            RelationalDiagnosticValue::string("none:InvariantViolation"),
        ),
        (
            "basis",
            RelationalDiagnosticValue::object([(
                "basis_kind",
                RelationalDiagnosticValue::string("string_only"),
            )]),
        ),
    ])
}

fn diagnostic_object_field<'a>(
    object: &'a std::collections::BTreeMap<String, RelationalDiagnosticValue>,
    field: &str,
) -> &'a RelationalDiagnosticValue {
    object
        .get(field)
        .unwrap_or_else(|| panic!("missing diagnostic field {field}"))
}

fn check_result_for_violation(
    violation: InvariantViolation,
    custom_provenance: Option<CustomInvariantProvenance>,
) -> crate::validation::data::InvariantCheckResult {
    let witness = violation.witness_key();
    crate::validation::data::InvariantCheckResult {
        execution_point: InvariantExecutionPoint::CommitBoundary,
        failure_effect: InvariantFailureEffect::BlockCommit,
        rule: InvariantReportedRule::Native(InvariantRule::MaxMergedIntents(1)),
        witness,
        groups: InvariantGroupSet::empty(),
        cost: InvariantCostClass::Touched,
        custom_provenance,
        verdict: InvariantVerdict::Violation(violation),
    }
}

fn none_violation() -> InvariantViolation {
    InvariantViolation {
        class: InvariantClass::CommitBoundary,
        code: DiagnosticCode::InvariantViolation,
        detail: "detail".to_string(),
        fields: InvariantViolationFields::None,
    }
}

fn invariant_failure_for(
    violation: InvariantViolation,
) -> crate::validation::engine::InvariantFailure {
    crate::validation::engine::InvariantFailure::new(
        InvariantExecutionPoint::CommitBoundary,
        InvariantFailureEffect::BlockCommit,
        violation,
    )
}

fn custom_provenance_with_visible_records(
    visible_entity_id: EntityId,
    visible_relation_id: RelationId,
) -> CustomInvariantProvenance {
    let touched_scope = TouchedStructuralSet::new(
        vec![visible_entity_id].into(),
        vec![visible_relation_id].into(),
        vec![PartitionId::new(7), PartitionId::new(9)].into(),
        Vec::new().into(),
        Vec::new().into(),
        Vec::new().into(),
        Vec::new().into(),
        Vec::new().into(),
    );
    CustomInvariantProvenance {
        observation_kind: InvariantObservationKind::Speculative,
        version_id: VersionId::new(20),
        current_version_id: VersionId::new(21),
        touched: touched_scope.provenance_summary(),
        counts: StructuralCountView::from_touched_scope(&touched_scope),
        traversal: CustomInvariantTraversalSummary {
            consumed_frontier: 1,
            consumed_steps: 2,
            remaining_frontier: 3,
            remaining_steps: 4,
            max_depth: 5,
        },
        // Synthetic projection fixture; this value does not assert runtime metering.
        work_units: std::num::NonZeroU64::new(17).unwrap(),
        proposal_identity: None,
    }
}

fn execution_result_with_proof_boundary(
    check_results: Vec<crate::validation::data::InvariantCheckResult>,
) -> InvariantExecutionResult {
    InvariantExecutionResult::executed(
        InvariantExecutionMetadata::new(
            InvariantExecutionPoint::CommitBoundary,
            InvariantObservationKind::Committed,
            crate::identity::data::VersionId(2),
            crate::identity::data::VersionId(2),
            InvariantGroupSet::empty(),
            InvariantGroupSet::empty(),
            InvariantCostClass::Touched,
            InvariantExecutionDisposition::Executed,
            None,
            false,
            RelationalExecutionModel::SingleLaneExecution,
            None,
            Vec::new(),
            Some(InvariantProofBoundarySummary::new(
                InvariantPlanScopeClass::PartitionScope,
                vec![InvariantScopeWideningCause::AllObservedPartitionScope],
                1,
                1,
            )),
            None,
        ),
        check_results,
    )
}
