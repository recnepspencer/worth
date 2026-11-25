use contracts::transactions::data::outcomes::commit_conflict::CommitConflictOwnerPort as _;
use worth_relational_contracts::internal_port as contracts;
pub mod diagnostic_fields;
pub mod diagnostic_value_terms;
pub mod field_shapes;
pub mod trace_entries;

use crate::publication::authority::diagnostic_emission::PreparationDiagnostics;
use contracts::diagnostics::data::artifacts::DeterminismExpectation;
use contracts::diagnostics::data::artifacts::DiagnosticCode;
use contracts::diagnostics::data::artifacts::DiagnosticsArtifactKind;
use contracts::diagnostics::data::artifacts::DiagnosticsScope;
use contracts::diagnostics::data::artifacts::RelationalDiagnosticArtifact;
use contracts::diagnostics::data::artifacts::RelationalDiagnosticsEntry;
use contracts::schema::continuity::errors::SchemaContinuityBundleIssue;
use contracts::schema::data::continuity::transition_plans::SchemaTransitionArtifact;
use contracts::schema::data::continuity::transition_plans::SchemaTransitionSummary;
use contracts::transactions::data::outcomes::commit_conflict::CommitConflict;
use contracts::transactions::data::outcomes::conflict_class::ConflictClass;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;

use super::{SchemaContinuityPlan, SchemaContinuityPrior};
use diagnostic_fields::diagnostics_fields;
use field_shapes::{
    schema_diff_atom_trace_fields, SchemaContinuityFailureFields, SchemaTransitionRejectedFields,
    SchemaTransitionSummaryFields,
};
use trace_entries::schema_transition_trace_entries;

pub(super) enum FailureTransitionView<'a> {
    Proposed(&'a contracts::schema::data::continuity::transition_plans::ProposedSchemaTransition),
    Artifact(&'a SchemaTransitionArtifact),
}

pub(crate) fn prepare_schema_continuity_diagnostics(
    branch_id: &contracts::history::data::coordinates::BranchId,
    plan: &SchemaContinuityPlan,
) -> Vec<RelationalDiagnosticArtifact> {
    let Some(transition) = &plan.schema_transition else {
        return Vec::new();
    };
    vec![
        schema_transition_summary_artifact(branch_id, plan, transition),
        RelationalDiagnosticArtifact::new(
            DiagnosticsScope::Schema,
            DiagnosticsArtifactKind::DetailedTrace,
            DeterminismExpectation::Required,
            schema_transition_trace_entries(branch_id, transition),
        ),
    ]
}

pub(super) fn schema_continuity_conflict_from_issue(
    diagnostics: PreparationDiagnostics<'_>,
    branch_id: &contracts::history::data::coordinates::BranchId,
    transition: Option<&SchemaTransitionArtifact>,
    issue: SchemaContinuityBundleIssue,
    envelope: &worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBody,
) -> TransactionCommitError {
    if descriptor_version_mismatch_issue(&issue) {
        diagnostics
            .instrumentation
            .count_descriptor_version_mismatch();
    }

    let class = schema_continuity_conflict_class(issue, transition, envelope);
    let conflict = CommitConflict::new(class);
    emit_schema_continuity_failure_diagnostic(
        diagnostics,
        branch_id,
        transition.map(FailureTransitionView::Artifact),
        None,
        &conflict,
    );
    TransactionCommitError::conflict(conflict)
}

pub(super) fn emit_schema_continuity_failure_diagnostic(
    diagnostics: PreparationDiagnostics<'_>,
    branch_id: &contracts::history::data::coordinates::BranchId,
    proposed_transition: Option<FailureTransitionView<'_>>,
    previous_envelope: Option<&SchemaContinuityPrior<'_>>,
    conflict: &CommitConflict,
) {
    let mut entries = vec![schema_continuity_failure_entry(
        branch_id,
        previous_envelope,
        conflict,
    )];

    if let Some(proposed_transition) = proposed_transition {
        entries.extend(rejected_schema_transition_entries(proposed_transition));
    }

    diagnostics.emit(
        DiagnosticsScope::Schema,
        DiagnosticsArtifactKind::Failure,
        entries,
    );
}

fn schema_transition_summary_artifact(
    branch_id: &contracts::history::data::coordinates::BranchId,
    plan: &SchemaContinuityPlan,
    transition: &SchemaTransitionArtifact,
) -> RelationalDiagnosticArtifact {
    let transition_summary = SchemaTransitionSummary::from_artifact(transition);
    RelationalDiagnosticArtifact::new(
        DiagnosticsScope::Schema,
        DiagnosticsArtifactKind::MinimalSummary,
        DeterminismExpectation::Required,
        vec![RelationalDiagnosticsEntry::new(
            DiagnosticCode::SchemaTransitionTraced,
            "schema continuity transition lowered into canonical commit artifacts",
            diagnostics_fields(SchemaTransitionSummaryFields {
                branch_id: branch_id.clone(),
                source_schema_id: transition.source_schema_id.clone(),
                source_schema_version_id: transition.source_schema_version_id,
                target_schema_id: transition.target_schema_id.clone(),
                target_schema_version_id: transition.target_schema_version_id,
                changed_atom_count: transition_summary.changed_atom_count,
                changed_strata: transition_summary.changed_strata,
                historical_interpretation: format!(
                    "{:?}",
                    transition_summary.historical_interpretation
                ),
                continuation: format!("{:?}", transition_summary.continuation),
                bridgeability: format!("{:?}", transition_summary.bridgeability),
                reconciliation: format!("{:?}", transition_summary.reconciliation),
                descriptor_semantics_version: plan.descriptor_semantics_version,
                descriptor_canonical_basis_version: transition
                    .continuation_descriptor
                    .bridge
                    .canonical_basis_version,
                normalized_boundary_count: transition
                    .continuation_descriptor
                    .normalized_boundary_count,
            }),
        )],
    )
}

fn descriptor_version_mismatch_issue(issue: &SchemaContinuityBundleIssue) -> bool {
    matches!(
        issue,
        SchemaContinuityBundleIssue::DescriptorSemanticsVersionMismatch { .. }
            | SchemaContinuityBundleIssue::DescriptorCanonicalBasisVersionMismatch { .. }
    )
}

fn schema_continuity_conflict_class(
    issue: SchemaContinuityBundleIssue,
    transition: Option<&SchemaTransitionArtifact>,
    envelope: &worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBody,
) -> ConflictClass {
    match issue {
        SchemaContinuityBundleIssue::IncompleteBundle
        | SchemaContinuityBundleIssue::ContinuationDescriptorDrift { .. }
        | SchemaContinuityBundleIssue::ContinuationBoundaryFingerprintMismatch { .. }
        | SchemaContinuityBundleIssue::DescriptorSemanticsVersionMismatch { .. }
        | SchemaContinuityBundleIssue::DescriptorCanonicalBasisVersionMismatch { .. }
        | SchemaContinuityBundleIssue::VisibleBridgeProofMismatch
        | SchemaContinuityBundleIssue::ReconciliationDescriptorDrift => {
            ConflictClass::UnsupportedBridgeDescriptor {
                detail: issue.detail(),
            }
        }
        SchemaContinuityBundleIssue::TargetSchemaVersionMismatch => {
            ConflictClass::UnsupportedBridgeDescriptor {
                detail: format!(
                    "{}: target {:?} envelope {:?}",
                    issue.detail(),
                    transition.map(|candidate| candidate.target_schema_version_id),
                    envelope.schema_version
                ),
            }
        }
        SchemaContinuityBundleIssue::LineageSchemaVersionMismatch => {
            ConflictClass::DirectionalityMismatchUnderCanonicalReconciliation {
                detail: format!(
                    "{}: lineage {:?} envelope {:?}",
                    issue.detail(),
                    envelope
                        .schema_reconciliation_descriptor
                        .as_ref()
                        .map(|descriptor| descriptor.resulting_lineage.resulting_schema_version_id),
                    envelope.schema_version
                ),
            }
        }
        SchemaContinuityBundleIssue::HistoricalReinterpretationViolation => {
            ConflictClass::HistoricalReinterpretationViolation {
                detail: issue.detail(),
            }
        }
    }
}

fn schema_continuity_failure_entry(
    branch_id: &contracts::history::data::coordinates::BranchId,
    previous_envelope: Option<&SchemaContinuityPrior<'_>>,
    conflict: &CommitConflict,
) -> RelationalDiagnosticsEntry {
    RelationalDiagnosticsEntry::new(
        DiagnosticCode::SchemaContinuityViolation,
        "schema continuity decision rejected during commit planning",
        diagnostics_fields(SchemaContinuityFailureFields {
            branch_id: branch_id.clone(),
            conflict_class: format!("{:?}", conflict.class),
            detail: conflict.detail.clone(),
            previous_schema_version: previous_envelope.map(|envelope| envelope.schema_version),
            previous_descriptor_semantics_version: previous_envelope
                .map(|envelope| envelope.descriptor_semantics_version),
        }),
    )
}

fn rejected_schema_transition_entries(
    proposed_transition: FailureTransitionView<'_>,
) -> Vec<RelationalDiagnosticsEntry> {
    let (
        source_schema_id,
        source_schema_version_id,
        target_schema_id,
        target_schema_version_id,
        diff_atoms,
    ) = match proposed_transition {
        FailureTransitionView::Proposed(transition) => (
            &transition.source_schema_id,
            transition.source_schema_version_id,
            &transition.target_schema_id,
            transition.target_schema_version_id,
            transition.diff_atoms.as_slice(),
        ),
        FailureTransitionView::Artifact(transition) => (
            &transition.source_schema_id,
            transition.source_schema_version_id,
            &transition.target_schema_id,
            transition.target_schema_version_id,
            transition.diff_atoms.as_slice(),
        ),
    };

    let mut entries = vec![RelationalDiagnosticsEntry::new(
        DiagnosticCode::SchemaTransitionClassified,
        "rejected schema transition proposal captured for diagnosis",
        diagnostics_fields(SchemaTransitionRejectedFields {
            source_schema_id: source_schema_id.clone(),
            source_schema_version_id,
            target_schema_id: target_schema_id.clone(),
            target_schema_version_id,
            changed_atom_count: diff_atoms.len(),
        }),
    )];

    entries.extend(diff_atoms.iter().enumerate().map(|(index, atom)| {
        RelationalDiagnosticsEntry::new(
            DiagnosticCode::SchemaTransitionClassified,
            format!("rejected schema diff atom {index} traced for diagnosis"),
            diagnostics_fields(schema_diff_atom_trace_fields(index, atom)),
        )
    }));

    entries
}
