use super::super::basis::{
    historical_admission_of, materialization_identity_of, preview_identity_of,
    AdmittedQueryBasisContext, HistoricalAdmissionClass, QueryContextAdmissionError,
    QueryContextAdmissionFailureClass, QueryContextDriftOutcome, QueryContextFamily,
};
use super::super::execution::QueryContextExecutionArtifact;
use super::super::identity::{
    compose_query_basis_counter_snapshot_digest, compose_query_basis_replay_digest,
};
use super::super::performance::{
    HistoricalMaterializationCostClass, QueryContextBudgetClass, QueryContextCostClass,
    QueryContextCounters, QueryContextPredictionDriftOutcome, QueryContextPredictionReport,
};
use super::super::scoped::ScopedQueryBasisContext;
use crate::basis::BasisAuthorityFamily;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryBasisMetadata {
    query_digest: String,
    basis_digest: String,
    basis_authority_family: BasisAuthorityFamily,
    basis_family: QueryContextFamily,
    cost_class: QueryContextCostClass,
    budget_class: QueryContextBudgetClass,
    historical_admission_class: Option<HistoricalAdmissionClass>,
    historical_materialization_cost_class: Option<HistoricalMaterializationCostClass>,
    requested_path_identity: Option<String>,
    admitted_path_identity: Option<String>,
    resolved_path_identity: Option<String>,
    materialization_path_identity: Option<String>,
    preview_provenance_identity: Option<String>,
    result_digest: String,
    drift_outcome: QueryContextDriftOutcome,
    prediction_report: Option<QueryContextPredictionReport>,
    prediction_drift_outcome: Option<QueryContextPredictionDriftOutcome>,
}

impl QueryBasisMetadata {
    pub fn query_digest(&self) -> &str {
        &self.query_digest
    }

    pub fn basis_digest(&self) -> &str {
        &self.basis_digest
    }

    pub fn basis_authority_family(&self) -> &BasisAuthorityFamily {
        &self.basis_authority_family
    }

    pub fn basis_family(&self) -> &QueryContextFamily {
        &self.basis_family
    }

    pub fn cost_class(&self) -> &QueryContextCostClass {
        &self.cost_class
    }

    pub fn budget_class(&self) -> &QueryContextBudgetClass {
        &self.budget_class
    }

    pub fn historical_admission_class(&self) -> Option<&HistoricalAdmissionClass> {
        self.historical_admission_class.as_ref()
    }

    pub fn historical_materialization_cost_class(
        &self,
    ) -> Option<&HistoricalMaterializationCostClass> {
        self.historical_materialization_cost_class.as_ref()
    }

    pub fn requested_path_identity(&self) -> Option<&str> {
        self.requested_path_identity.as_deref()
    }

    pub fn admitted_path_identity(&self) -> Option<&str> {
        self.admitted_path_identity.as_deref()
    }

    pub fn resolved_path_identity(&self) -> Option<&str> {
        self.resolved_path_identity.as_deref()
    }

    pub fn materialization_path_identity(&self) -> Option<&str> {
        self.materialization_path_identity.as_deref()
    }

    pub fn preview_provenance_identity(&self) -> Option<&str> {
        self.preview_provenance_identity.as_deref()
    }

    pub fn preview_identity(&self) -> Option<&str> {
        self.preview_provenance_identity()
    }

    pub fn result_digest(&self) -> &str {
        &self.result_digest
    }

    pub fn drift_outcome(&self) -> &QueryContextDriftOutcome {
        &self.drift_outcome
    }

    pub fn prediction_report(&self) -> Option<&QueryContextPredictionReport> {
        self.prediction_report.as_ref()
    }

    pub fn prediction_drift_outcome(&self) -> Option<&QueryContextPredictionDriftOutcome> {
        self.prediction_drift_outcome.as_ref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryBasisResultBundle {
    context: ScopedQueryBasisContext,
    execution: QueryContextExecutionArtifact,
    metadata: QueryBasisMetadata,
    replay_digest: String,
    counter_snapshot_digest: String,
}

impl QueryBasisResultBundle {
    pub fn context(&self) -> &ScopedQueryBasisContext {
        &self.context
    }

    pub fn execution(&self) -> &QueryContextExecutionArtifact {
        &self.execution
    }

    pub fn metadata(&self) -> &QueryBasisMetadata {
        &self.metadata
    }

    pub fn replay_digest(&self) -> &str {
        &self.replay_digest
    }

    pub fn counter_snapshot_digest(&self) -> &str {
        &self.counter_snapshot_digest
    }
}

pub(crate) fn attach_legacy_query_basis_metadata(
    context: &AdmittedQueryBasisContext,
    result: &QueryContextExecutionArtifact,
) -> Result<QueryBasisMetadata, QueryContextAdmissionError> {
    if context.query_digest() != result.query_digest() {
        return Err(QueryContextAdmissionError::new(
            QueryContextAdmissionFailureClass::BasisSubstitutionForbidden,
            "result metadata attachment requires the original admitted query digest",
            QueryContextCounters::for_denial(false, true),
        ));
    }

    Ok(QueryBasisMetadata {
        query_digest: context.query_digest().to_string(),
        basis_digest: context.basis_digest().to_string(),
        basis_authority_family: context.basis_authority_family().clone(),
        basis_family: context.family().clone(),
        cost_class: context.cost_class().clone(),
        budget_class: context.budget_class().clone(),
        historical_admission_class: result
            .historical_admission_class()
            .cloned()
            .or_else(|| historical_admission_of(context)),
        historical_materialization_cost_class: result
            .historical_materialization_cost_class()
            .cloned()
            .or_else(|| context.historical_materialization_cost_class().cloned()),
        requested_path_identity: result.requested_path_identity().map(ToString::to_string),
        admitted_path_identity: result.admitted_path_identity().map(ToString::to_string),
        resolved_path_identity: result.resolved_path_identity().map(ToString::to_string),
        materialization_path_identity: result
            .materialization_path_identity()
            .map(ToString::to_string)
            .or_else(|| materialization_identity_of(context)),
        preview_provenance_identity: result
            .preview_provenance_identity()
            .map(ToString::to_string)
            .or_else(|| preview_identity_of(context)),
        result_digest: result.result_digest().to_string(),
        drift_outcome: context.drift_outcome().clone(),
        prediction_report: context.prediction_report().cloned(),
        prediction_drift_outcome: Some(result.prediction_drift_outcome().clone()),
    })
}

pub(crate) fn build_legacy_query_basis_result_bundle(
    context: &ScopedQueryBasisContext,
    execution: QueryContextExecutionArtifact,
) -> Result<QueryBasisResultBundle, QueryContextAdmissionError> {
    let legacy_context = context.context();
    let metadata = attach_legacy_query_basis_metadata(legacy_context, &execution)?;
    let replay_digest = compose_query_basis_replay_digest(
        legacy_context,
        execution.result_digest(),
        metadata.result_digest(),
        metadata.prediction_drift_outcome(),
    )
    .map_err(|cause| {
        QueryContextAdmissionError::identity_refused(cause, context.counters().clone())
    })?;
    let counter_snapshot_digest =
        compose_query_basis_counter_snapshot_digest(legacy_context, &execution);

    Ok(QueryBasisResultBundle {
        context: context.clone(),
        execution,
        metadata,
        replay_digest,
        counter_snapshot_digest,
    })
}
