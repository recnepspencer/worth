use super::failures::InvariantFailure;
use crate::diagnostics::data::DiagnosticCode;
use crate::publication::bundle::PublicationStage;
use crate::validation::data::{
    InvariantClass, InvariantCostClass, InvariantDecisionKind, InvariantFailureEffect,
    InvariantGroupSet, InvariantReportedRule, InvariantRule, InvariantVerdict, InvariantViolation,
    InvariantViolationFields,
};
use crate::validation::engine::{
    InvariantExecutionDisposition, InvariantExecutionMetadata, InvariantObservationKind,
    InvariantPlanScopeClass, InvariantProofBoundarySummary, InvariantScopeWideningCause,
};

#[test]
fn invariant_failure_converts_to_commit_and_publication_errors() {
    let failure = InvariantFailure::new(
        crate::validation::data::InvariantExecutionPoint::SnapshotPublication,
        InvariantFailureEffect::BlockPublication,
        InvariantViolation {
            class: InvariantClass::SnapshotAudit,
            code: DiagnosticCode::InvariantViolation,
            detail: "detail".to_string(),
            fields: InvariantViolationFields::None,
        },
    );

    let conflict = failure.clone().into_commit_conflict();
    assert_eq!(conflict.code(), DiagnosticCode::InvariantViolation);
    assert_eq!(conflict.detail(), "detail".to_string());

    let publication = failure.into_publication_error(PublicationStage::InvariantCheck);
    assert_eq!(publication.stage, PublicationStage::InvariantCheck);
    assert_eq!(publication.detail, "detail".to_string());
}

#[test]
fn skipped_result_retains_execution_metadata_without_checks() {
    let metadata = InvariantExecutionMetadata::new(
        crate::validation::data::InvariantExecutionPoint::CommitBoundary,
        InvariantObservationKind::Committed,
        crate::identity::data::VersionId(2),
        crate::identity::data::VersionId(1),
        crate::validation::data::InvariantGroupSet::from_mask(0b111),
        crate::validation::data::InvariantGroupSet::empty(),
        InvariantCostClass::Partition,
        InvariantExecutionDisposition::SkippedByMayBreakMask,
        None,
        true,
        None,
        Vec::new(),
        Some(InvariantProofBoundarySummary::new(
            InvariantPlanScopeClass::BroaderScope,
            vec![InvariantScopeWideningCause::AllObservedPartitionScope],
            1,
            0,
        )),
        None,
    );

    let result = crate::validation::engine::InvariantExecutionResult::skipped(metadata);

    assert!(result.results().is_empty());
    assert_eq!(
        result.metadata().disposition(),
        InvariantExecutionDisposition::SkippedByMayBreakMask
    );
    assert!(result.metadata().has_merged_plan());
}

#[test]
fn executed_result_builds_decision_log_from_results() {
    let metadata = InvariantExecutionMetadata::new(
        crate::validation::data::InvariantExecutionPoint::CommitBoundary,
        InvariantObservationKind::Committed,
        crate::identity::data::VersionId(2),
        crate::identity::data::VersionId(2),
        InvariantGroupSet::empty(),
        InvariantGroupSet::empty(),
        InvariantCostClass::Touched,
        InvariantExecutionDisposition::Executed,
        None,
        false,
        None,
        Vec::new(),
        None,
        None,
    );
    let result = crate::validation::engine::InvariantExecutionResult::executed(
        metadata,
        vec![crate::validation::data::InvariantCheckResult {
            execution_point: crate::validation::data::InvariantExecutionPoint::CommitBoundary,
            failure_effect: InvariantFailureEffect::BlockCommit,
            rule: InvariantReportedRule::Native(InvariantRule::MaxMergedIntents(1)),
            witness: crate::validation::data::InvariantWitnessKey::pass(),
            groups: InvariantGroupSet::empty(),
            cost: InvariantCostClass::Touched,
            custom_provenance: None,
            verdict: InvariantVerdict::Pass,
        }],
    );

    assert_eq!(result.decision_log().len(), 1);
    assert_eq!(
        result.decision_log()[0].decision,
        InvariantDecisionKind::Passed
    );
}

#[test]
fn proof_boundary_artifact_uses_typed_diagnostic_labels() {
    let metadata = InvariantExecutionMetadata::new(
        crate::validation::data::InvariantExecutionPoint::CommitBoundary,
        InvariantObservationKind::Committed,
        crate::identity::data::VersionId(2),
        crate::identity::data::VersionId(2),
        InvariantGroupSet::empty(),
        InvariantGroupSet::empty(),
        InvariantCostClass::Touched,
        InvariantExecutionDisposition::Executed,
        None,
        false,
        None,
        Vec::new(),
        Some(InvariantProofBoundarySummary::new(
            InvariantPlanScopeClass::PartitionScope,
            vec![InvariantScopeWideningCause::AllObservedPartitionScope],
            3,
            2,
        )),
        None,
    );
    let result =
        crate::validation::engine::InvariantExecutionResult::executed(metadata, Vec::new());
    let artifact = result
        .proof_boundary_artifact()
        .expect("proof boundary artifact");

    assert_eq!(
        artifact.scope_class(),
        InvariantPlanScopeClass::PartitionScope
    );
    assert_eq!(
        artifact.widened_causes(),
        &[InvariantScopeWideningCause::AllObservedPartitionScope]
    );
    assert_eq!(artifact.packet_count(), 3);
    assert_eq!(artifact.touched_partition_count(), 2);
}

#[test]
fn failure_artifact_preserves_failure_effect_and_nested_proof_boundary() {
    let metadata = InvariantExecutionMetadata::new(
        crate::validation::data::InvariantExecutionPoint::CommitBoundary,
        InvariantObservationKind::Committed,
        crate::identity::data::VersionId(2),
        crate::identity::data::VersionId(2),
        InvariantGroupSet::empty(),
        InvariantGroupSet::empty(),
        InvariantCostClass::Touched,
        InvariantExecutionDisposition::Executed,
        None,
        false,
        None,
        Vec::new(),
        Some(InvariantProofBoundarySummary::new(
            InvariantPlanScopeClass::PartitionScope,
            vec![InvariantScopeWideningCause::AllObservedPartitionScope],
            1,
            1,
        )),
        None,
    );
    let violation = InvariantViolation {
        class: InvariantClass::CommitBoundary,
        code: DiagnosticCode::InvariantViolation,
        detail: "detail".to_string(),
        fields: InvariantViolationFields::None,
    };
    let result = crate::validation::engine::InvariantExecutionResult::executed(
        metadata,
        vec![crate::validation::data::InvariantCheckResult {
            execution_point: crate::validation::data::InvariantExecutionPoint::CommitBoundary,
            failure_effect: InvariantFailureEffect::BlockCommit,
            rule: InvariantReportedRule::Native(InvariantRule::MaxMergedIntents(1)),
            witness: crate::validation::data::InvariantWitnessKey::pass(),
            groups: InvariantGroupSet::empty(),
            cost: InvariantCostClass::Touched,
            custom_provenance: None,
            verdict: InvariantVerdict::Violation(violation.clone()),
        }],
    );
    let failure = InvariantFailure::new(
        crate::validation::data::InvariantExecutionPoint::CommitBoundary,
        InvariantFailureEffect::BlockCommit,
        violation,
    );
    let artifact = result.failure_artifact(&failure);

    assert_eq!(
        artifact.execution_point(),
        crate::validation::data::InvariantExecutionPoint::CommitBoundary
    );
    assert_eq!(
        artifact.failure_effect(),
        InvariantFailureEffect::BlockCommit
    );
    assert_eq!(artifact.violation(), &InvariantViolationFields::None);
    assert!(artifact.custom_provenance().is_none());

    let proof_boundary = artifact
        .proof_boundary()
        .expect("failure artifact proof boundary");
    assert_eq!(
        proof_boundary.scope_class(),
        InvariantPlanScopeClass::PartitionScope
    );
    assert_eq!(
        proof_boundary.widened_causes(),
        &[InvariantScopeWideningCause::AllObservedPartitionScope]
    );
    assert_eq!(proof_boundary.packet_count(), 1);
    assert_eq!(proof_boundary.touched_partition_count(), 1);
}
