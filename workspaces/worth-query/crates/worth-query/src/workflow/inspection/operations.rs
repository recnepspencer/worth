use super::evidence::{WorkflowAuthorityOutcomeArtifact, WorkflowReplayBundle};
#[cfg(test)]
use super::identities::{conflict_scope_identity, post_merge_scope_identity};
use super::identities::{
    delivery_or_failure_identity, workflow_authoritative_outcome_identity,
    workflow_authority_request_identity, workflow_replay_bundle_identity,
};
#[cfg(test)]
use super::model::{
    ConflictInspectionFamily, PostMergeInspectionFamily, WorkflowInspectionError,
    WorkflowInspectionFailureClass,
};
use super::model::{WorkflowAuthorityOutcomeFamily, WorkflowStalenessOutcome};
#[cfg(test)]
use super::report::{
    ConflictInspectionRow, PostMergeInspectionRow, QueryConflictInspectionArtifact,
    QueryPostMergeInspectionArtifact,
};
use crate::evidence_identity::WorthQueryEvidenceIdentity;
#[cfg(test)]
use crate::workflow::foundation::WorkflowDeclarationFamily;
use crate::workflow::foundation::{QueryWorkflowDeclaration, WorkflowPredictionDriftOutcome};
#[cfg(test)]
use crate::workflow::inspection_projection::{
    relational_merge_class_admission, relational_merge_class_label, relational_merge_class_shape,
};
use crate::workflow::lowering::LoweredMergeWorkflowDeclaration;
use crate::workflow::lowering::{LoweredMutationIntentDeclaration, QueryWritebackDeclaration};
use crate::workflow::performance::{
    WorkflowBudgetOutcome, WorkflowLoweringCounters, WorkflowPredictionReport,
};
#[cfg(test)]
use crate::workflow::performance::{WorkflowInspectionBudget, WorkflowInspectionCounters};
#[cfg(test)]
use worth_relational::facade::merge::{
    NormalizedRelationalMergeRequest, RelationalMergeInspectionArtifact,
};

#[cfg(test)]
pub(crate) fn inspect_merge_conflicts(
    declaration: &QueryWorkflowDeclaration,
    merge_declaration: &LoweredMergeWorkflowDeclaration,
    relational_inspection: &RelationalMergeInspectionArtifact,
) -> Result<QueryConflictInspectionArtifact, WorkflowInspectionError> {
    if declaration.request().declaration_family()
        != &WorkflowDeclarationFamily::ConflictInspectionNarrow
    {
        return Err(WorkflowInspectionError::new(
            WorkflowInspectionFailureClass::UnsupportedInspectionFamily,
            "conflict inspection requires an admitted conflict inspection declaration",
            WorkflowInspectionCounters {
                workflow_inspection_denial_width: 1,
                ..WorkflowInspectionCounters::default()
            },
        ));
    }

    if declaration.binding().query_identity()
        != merge_declaration.declaration().binding().query_identity()
        || declaration.binding().basis_identity()
            != merge_declaration.declaration().binding().basis_identity()
    {
        return Err(WorkflowInspectionError::new(
            WorkflowInspectionFailureClass::UnsupportedInspectionFamily,
            "conflict inspection declaration must bind the same query and basis identity as the lowered merge declaration",
            WorkflowInspectionCounters {
                workflow_inspection_denial_width: 1,
                ..WorkflowInspectionCounters::default()
            },
        ));
    }

    let normalized_merge_request = NormalizedRelationalMergeRequest::from_execution_request(
        merge_declaration.merge_request().clone(),
    )
    .map_err(|_| {
        WorkflowInspectionError::new(
            WorkflowInspectionFailureClass::RelationalInspectionMismatch,
            "lowered merge request could not be normalized for relational inspection comparison",
            WorkflowInspectionCounters {
                workflow_inspection_denial_width: 1,
                ..WorkflowInspectionCounters::default()
            },
        )
    })?;

    if &normalized_merge_request != relational_inspection.request() {
        return Err(WorkflowInspectionError::new(
            WorkflowInspectionFailureClass::RelationalInspectionMismatch,
            "relational merge inspection artifact must match the lowered merge request exactly",
            WorkflowInspectionCounters {
                workflow_inspection_denial_width: 1,
                ..WorkflowInspectionCounters::default()
            },
        ));
    }

    let rows = relational_inspection
        .rows()
        .iter()
        .map(|row| {
            let merge_class_shape = relational_merge_class_shape(row);
            let merge_class = relational_merge_class_label(row);
            let merge_class_admission = relational_merge_class_admission(row);
            let conflict_scope_identity = conflict_scope_identity(
                declaration,
                merge_declaration,
                merge_class_shape.family,
                merge_class_shape.class,
                merge_class_admission.as_str(),
                row.row_digest(),
            )
            .expect("positive fixture identity fits canonical digest bounds");
            ConflictInspectionRow {
                workflow_basis_digest: declaration.binding().basis_for_reporting().to_string(),
                merge_class,
                merge_class_admission,
                target_basis_digest: merge_declaration.merge_request().target_branch().0.clone(),
                source_basis_digest: merge_declaration.merge_request().source_branch().0.clone(),
                conflict_scope_digest: conflict_scope_identity.as_str().to_string(),
                authority_target_family: merge_declaration
                    .declaration()
                    .report()
                    .authority_target_family()
                    .clone(),
            }
        })
        .collect::<Vec<_>>();
    let row_width = rows.len();

    Ok(QueryConflictInspectionArtifact {
        declaration_digest: declaration.report().declaration_digest().to_string(),
        family: ConflictInspectionFamily::MergeWorkflowNarrow,
        budget: WorkflowInspectionBudget::ConflictInspectionNarrow,
        prediction_report: WorkflowPredictionReport {
            predicted_declaration_width: 1,
            predicted_inspection_width: row_width,
            predicted_lowering_width: 1,
            predicted_freshness_width: 1,
            predicted_denial_width: 1,
        },
        drift_outcome: WorkflowPredictionDriftOutcome::WithinBudget,
        rows,
        counters: WorkflowInspectionCounters {
            workflow_inspection_count: 1,
            workflow_conflict_inspection_count: 1,
            workflow_post_merge_inspection_count: 0,
            workflow_inspection_row_width: row_width,
            workflow_inspection_merge_class_width: row_width,
            workflow_inspection_denial_width: 0,
            workflow_executor_rediscovery_count: 0,
        },
    })
}

pub fn shape_mutation_authority_outcome(
    declaration: &LoweredMutationIntentDeclaration,
) -> WorkflowAuthorityOutcomeArtifact {
    shape_authority_outcome(
        declaration.declaration(),
        WorkflowAuthorityOutcomeFamily::MutationLoweringAdmitted,
        declaration.lowering_identity(),
        declaration.counters().clone(),
        1,
    )
}

pub fn shape_merge_authority_outcome(
    declaration: &LoweredMergeWorkflowDeclaration,
) -> WorkflowAuthorityOutcomeArtifact {
    shape_authority_outcome(
        declaration.declaration(),
        WorkflowAuthorityOutcomeFamily::MergeLoweringAdmitted,
        declaration.lowering_identity(),
        declaration.counters().clone(),
        1,
    )
}

pub fn shape_writeback_authority_outcome(
    declaration: &QueryWritebackDeclaration,
) -> WorkflowAuthorityOutcomeArtifact {
    shape_authority_outcome(
        declaration.declaration(),
        WorkflowAuthorityOutcomeFamily::WritebackLoweringAdmitted,
        declaration.lowering_identity(),
        declaration.counters().clone(),
        1,
    )
}

#[cfg(test)]
pub(crate) fn inspect_post_merge_outcome(
    declaration: &QueryWorkflowDeclaration,
    outcome: &WorkflowAuthorityOutcomeArtifact,
) -> Result<QueryPostMergeInspectionArtifact, WorkflowInspectionError> {
    if declaration.request().declaration_family()
        != &WorkflowDeclarationFamily::PostMergeInspectionNarrow
    {
        return Err(WorkflowInspectionError::new(
            WorkflowInspectionFailureClass::UnsupportedInspectionFamily,
            "post-merge inspection requires an admitted post-merge inspection declaration",
            WorkflowInspectionCounters {
                workflow_inspection_denial_width: 1,
                ..WorkflowInspectionCounters::default()
            },
        ));
    }

    if !matches!(
        outcome.family(),
        WorkflowAuthorityOutcomeFamily::MergeLoweringAdmitted
            | WorkflowAuthorityOutcomeFamily::WritebackLoweringAdmitted
    ) {
        return Err(WorkflowInspectionError::new(
            WorkflowInspectionFailureClass::NonAuthoritativeOutcomeForbidden,
            "post-merge inspection requires a merge or writeback authority outcome artifact",
            WorkflowInspectionCounters {
                workflow_inspection_denial_width: 1,
                ..WorkflowInspectionCounters::default()
            },
        ));
    }

    if declaration.binding().query_identity() != outcome.source_query_identity()
        || declaration.binding().basis_identity() != outcome.source_basis_identity()
    {
        return Err(WorkflowInspectionError::new(
            WorkflowInspectionFailureClass::UnsupportedInspectionFamily,
            "post-merge inspection declaration must bind the same query and basis identity as the authoritative outcome",
            WorkflowInspectionCounters {
                workflow_inspection_denial_width: 1,
                ..WorkflowInspectionCounters::default()
            },
        ));
    }

    let row = PostMergeInspectionRow {
        authoritative_outcome_basis_digest: outcome.source_basis_digest().to_string(),
        authority_target_family: outcome.authority_target_family().clone(),
        authoritative_commit_or_outcome_digest: outcome.authoritative_outcome_digest().to_string(),
        post_merge_scope_digest: post_merge_scope_identity(declaration, outcome)
            .as_str()
            .to_string(),
        merge_or_writeback_origin_digest: outcome.source_declaration_digest().to_string(),
        inspection_result_family: PostMergeInspectionFamily::AuthoritativeOutcomeNarrow
            .as_str()
            .to_string(),
    };

    Ok(QueryPostMergeInspectionArtifact {
        origin_digest: declaration.report().declaration_digest().to_string(),
        family: PostMergeInspectionFamily::AuthoritativeOutcomeNarrow,
        budget: WorkflowInspectionBudget::PostMergeInspectionNarrow,
        prediction_report: WorkflowPredictionReport {
            predicted_declaration_width: 1,
            predicted_inspection_width: 1,
            predicted_lowering_width: 1,
            predicted_freshness_width: 1,
            predicted_denial_width: 1,
        },
        drift_outcome: WorkflowPredictionDriftOutcome::WithinBudget,
        rows: vec![row],
        counters: WorkflowInspectionCounters {
            workflow_inspection_count: 1,
            workflow_conflict_inspection_count: 0,
            workflow_post_merge_inspection_count: 1,
            workflow_inspection_row_width: 1,
            workflow_inspection_merge_class_width: 1,
            workflow_inspection_denial_width: 0,
            workflow_executor_rediscovery_count: 0,
        },
    })
}

pub fn build_workflow_replay_bundle(
    outcome: &WorkflowAuthorityOutcomeArtifact,
) -> WorkflowReplayBundle {
    let delivery_or_failure_identity = delivery_or_failure_identity(outcome);
    let bundle_identity = workflow_replay_bundle_identity(outcome, &delivery_or_failure_identity);

    let counters = outcome.counters().with_replay_bundle_issued();

    WorkflowReplayBundle {
        bundle_identity,
        query_identity: outcome.source_query_identity().clone(),
        plan_identity: outcome.source_plan_identity().clone(),
        basis_identity: outcome.source_basis_identity().clone(),
        declaration_identity: outcome.source_declaration_identity().clone(),
        authority_target_family: outcome.authority_target_family().clone(),
        authority_request_identity: outcome.authority_request_identity().clone(),
        authoritative_outcome_identity: outcome.authoritative_outcome_identity().clone(),
        delivery_or_failure_identity,
        counters,
    }
}

fn shape_authority_outcome(
    declaration: &QueryWorkflowDeclaration,
    family: WorkflowAuthorityOutcomeFamily,
    request_identity: &WorthQueryEvidenceIdentity,
    counters: WorkflowLoweringCounters,
    realized_width: usize,
) -> WorkflowAuthorityOutcomeArtifact {
    let authority_request_identity =
        workflow_authority_request_identity(family.clone(), request_identity);
    let authoritative_outcome_identity =
        workflow_authoritative_outcome_identity(declaration, &family, &authority_request_identity);

    WorkflowAuthorityOutcomeArtifact {
        family,
        authority_target_family: declaration.report().authority_target_family().clone(),
        source_query_identity: declaration.binding().query_identity().clone(),
        source_plan_identity: declaration.binding().source_identity().clone(),
        source_basis_identity: declaration.binding().basis_identity().clone(),
        source_declaration_identity: declaration.report().declaration_identity().clone(),
        authority_request_identity,
        authoritative_outcome_identity,
        cost_class: declaration.report().cost_class().clone(),
        budget_class: declaration.report().budget_class().clone(),
        budget_outcome: WorkflowBudgetOutcome::WithinBudget,
        prediction_report: WorkflowPredictionReport {
            predicted_declaration_width: 1,
            predicted_inspection_width: 1,
            predicted_lowering_width: 1,
            predicted_freshness_width: 1,
            predicted_denial_width: 1,
        },
        prediction_drift_outcome: WorkflowPredictionDriftOutcome::WithinBudget,
        freshness_outcome: WorkflowStalenessOutcome::StillFresh,
        explicit_rebind: None,
        realized_width,
        counters,
    }
}
