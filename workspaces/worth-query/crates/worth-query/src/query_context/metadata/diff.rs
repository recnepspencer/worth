use super::super::basis::{
    ComparisonBasisFamily, QueryContextAdmissionError, QueryContextAdmissionFailureClass,
    QueryContextDriftOutcome,
};
use super::super::comparison::{AdmittedDiffQueryContext, QueryDiffChangeSetArtifact};
use super::super::execution::QueryContextExecutionArtifact;
use super::super::identity::{
    compose_query_diff_counter_snapshot_digest, compose_query_diff_replay_digest,
};
use super::super::performance::{
    QueryContextBudgetClass, QueryContextCostClass, QueryContextCounters,
    QueryContextPredictionDriftOutcome, QueryContextPredictionReport,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiffQueryMetadata {
    query_digest: String,
    comparison_basis_family: ComparisonBasisFamily,
    left_basis_digest: String,
    right_basis_digest: String,
    result_shape_digest: String,
    left_result_digest: String,
    right_result_digest: String,
    comparison_result_digest: String,
    cost_class: QueryContextCostClass,
    budget_class: QueryContextBudgetClass,
    prediction_report: QueryContextPredictionReport,
    prediction_drift_outcome: QueryContextPredictionDriftOutcome,
    drift_outcome: QueryContextDriftOutcome,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryDiffResultBundle {
    context: AdmittedDiffQueryContext,
    change_set: QueryDiffChangeSetArtifact,
    metadata: DiffQueryMetadata,
    replay_digest: String,
    counter_snapshot_digest: String,
}

impl QueryDiffResultBundle {
    pub fn context(&self) -> &AdmittedDiffQueryContext {
        &self.context
    }

    pub fn change_set(&self) -> &QueryDiffChangeSetArtifact {
        &self.change_set
    }

    pub fn metadata(&self) -> &DiffQueryMetadata {
        &self.metadata
    }

    pub fn replay_digest(&self) -> &str {
        &self.replay_digest
    }

    pub fn counter_snapshot_digest(&self) -> &str {
        &self.counter_snapshot_digest
    }
}

impl DiffQueryMetadata {
    pub fn query_digest(&self) -> &str {
        &self.query_digest
    }

    pub fn comparison_basis_family(&self) -> &ComparisonBasisFamily {
        &self.comparison_basis_family
    }

    pub fn left_basis_digest(&self) -> &str {
        &self.left_basis_digest
    }

    pub fn right_basis_digest(&self) -> &str {
        &self.right_basis_digest
    }

    pub fn left_result_digest(&self) -> &str {
        &self.left_result_digest
    }

    pub fn result_shape_digest(&self) -> &str {
        &self.result_shape_digest
    }

    pub fn right_result_digest(&self) -> &str {
        &self.right_result_digest
    }

    pub fn comparison_result_digest(&self) -> &str {
        &self.comparison_result_digest
    }

    pub fn cost_class(&self) -> &QueryContextCostClass {
        &self.cost_class
    }

    pub fn budget_class(&self) -> &QueryContextBudgetClass {
        &self.budget_class
    }

    pub fn prediction_report(&self) -> &QueryContextPredictionReport {
        &self.prediction_report
    }

    pub fn prediction_drift_outcome(&self) -> &QueryContextPredictionDriftOutcome {
        &self.prediction_drift_outcome
    }

    pub fn drift_outcome(&self) -> &QueryContextDriftOutcome {
        &self.drift_outcome
    }
}

pub fn attach_diff_query_metadata(
    context: &AdmittedDiffQueryContext,
    left_result: &QueryContextExecutionArtifact,
    right_result: &QueryContextExecutionArtifact,
    change_set: &QueryDiffChangeSetArtifact,
) -> Result<DiffQueryMetadata, QueryContextAdmissionError> {
    if left_result.query_digest() != context.left().query_digest()
        || right_result.query_digest() != context.right().query_digest()
    {
        return Err(QueryContextAdmissionError::new(
            QueryContextAdmissionFailureClass::DiffScopeMismatch,
            "diff metadata requires result envelopes that match the admitted context pair",
            QueryContextCounters::for_diff_denial(true, false),
        ));
    }

    if left_result.basis_digest() != context.left().basis_digest()
        || right_result.basis_digest() != context.right().basis_digest()
    {
        return Err(QueryContextAdmissionError::new(
            QueryContextAdmissionFailureClass::ComparisonShapeMismatch,
            "diff metadata requires execution artifacts bound to the admitted basis pair",
            QueryContextCounters::for_diff_denial(false, false),
        ));
    }

    if left_result.result_shape_digest() != right_result.result_shape_digest() {
        return Err(QueryContextAdmissionError::new(
            QueryContextAdmissionFailureClass::ComparisonShapeMismatch,
            "diff metadata requires execution artifacts that preserve one declared result-shape identity",
            QueryContextCounters::for_diff_denial(false, false),
        ));
    }

    if change_set.left_basis_digest() != context.left().basis_digest()
        || change_set.right_basis_digest() != context.right().basis_digest()
        || change_set.query_digest() != context.left().query_digest()
    {
        return Err(QueryContextAdmissionError::new(
            QueryContextAdmissionFailureClass::ComparisonShapeMismatch,
            "diff metadata requires the realized query-shaped change set for the admitted basis pair",
            QueryContextCounters::for_diff_denial(false, false),
        ));
    }

    Ok(DiffQueryMetadata {
        query_digest: context.left().query_digest().to_string(),
        comparison_basis_family: context.family().clone(),
        left_basis_digest: context.left().basis_digest().to_string(),
        right_basis_digest: context.right().basis_digest().to_string(),
        result_shape_digest: left_result.result_shape_digest().to_string(),
        left_result_digest: left_result.result_digest().to_string(),
        right_result_digest: right_result.result_digest().to_string(),
        comparison_result_digest: change_set.result_digest().to_string(),
        cost_class: context.cost_class().clone(),
        budget_class: context.budget_class().clone(),
        prediction_report: context.prediction_report().clone(),
        prediction_drift_outcome: change_set.prediction_drift_outcome().clone(),
        drift_outcome: context.drift_outcome().clone(),
    })
}

pub fn build_query_diff_result_bundle(
    context: &AdmittedDiffQueryContext,
    change_set: QueryDiffChangeSetArtifact,
    left_result: &QueryContextExecutionArtifact,
    right_result: &QueryContextExecutionArtifact,
) -> Result<QueryDiffResultBundle, QueryContextAdmissionError> {
    let metadata = attach_diff_query_metadata(context, left_result, right_result, &change_set)?;
    let replay_digest = compose_query_diff_replay_digest(
        context,
        change_set.result_digest(),
        change_set.prediction_drift_outcome(),
    )
    .map_err(|cause| {
        QueryContextAdmissionError::identity_refused(cause, context.counters().clone())
    })?;
    let counter_snapshot_digest = compose_query_diff_counter_snapshot_digest(context, &change_set);

    Ok(QueryDiffResultBundle {
        context: context.clone(),
        change_set,
        metadata,
        replay_digest,
        counter_snapshot_digest,
    })
}
