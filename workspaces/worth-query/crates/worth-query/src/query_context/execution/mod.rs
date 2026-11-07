use crate::execution::execute_preflight_bundle;
use crate::identity::ResultDigest;

use super::identity::compose_preview_derived_result_shape_digest;

mod artifact;
mod synthetic;

#[cfg(test)]
mod test_support;

use super::basis::{
    AdmittedQueryBasisContext, HistoricalAdmissionClass, QueryContextAdmissionError,
    QueryContextAdmissionFailureClass,
};
use super::historical::{
    admitted_path_identity, materialization_path_identity, requested_path_identity,
    resolved_path_identity,
};
use super::performance::{QueryContextCounters, QueryContextPredictionDriftOutcome};

pub use artifact::{
    QueryContextExecutionArtifact, QueryContextExecutionCounters, QueryContextExecutionFamily,
};

pub(crate) fn execute_legacy_query_basis_context(
    context: &AdmittedQueryBasisContext,
) -> Result<QueryContextExecutionArtifact, QueryContextAdmissionError> {
    let carries_count_aggregate_plan = match context.binding().evidence() {
        super::basis::QueryBasisBindingEvidenceView::Runtime { preflight } => {
            preflight_is_count_aggregate(preflight)
        }
        super::basis::QueryBasisBindingEvidenceView::Historical {
            query_preflight, ..
        } => preflight_is_count_aggregate(query_preflight),
        super::basis::QueryBasisBindingEvidenceView::PreviewDerived { .. } => false,
    };
    if carries_count_aggregate_plan {
        return Err(QueryContextAdmissionError::new(
            QueryContextAdmissionFailureClass::UnsupportedHistoricalBasis,
            "legacy query-basis execution cannot produce aggregate results",
            QueryContextCounters::for_denial(false, false),
        ));
    }
    match context.binding().evidence() {
        super::basis::QueryBasisBindingEvidenceView::Runtime { preflight } => {
            let execution = execute_preflight_bundle(preflight)
                .expect("runtime preflight execution should succeed");
            let family = if context.family().as_str() == "current_branch_head" {
                QueryContextExecutionFamily::RuntimeCurrent
            } else {
                QueryContextExecutionFamily::RuntimeBranch
            };
            Ok(QueryContextExecutionArtifact {
                query_digest: execution.report().query_digest().as_str().to_string(),
                basis_digest: execution.report().basis_digest().as_str().to_string(),
                result_digest: execution.report().result_digest().as_str().to_string(),
                result_shape_digest: preflight
                    .plan()
                    .result_shape()
                    .canonical_result_shape_digest()
                    .as_str()
                    .to_string(),
                rows: execution.rows().to_vec(),
                family,
                cost_class: context.cost_class().clone(),
                budget_class: context.budget_class().clone(),
                prediction_report: context.prediction_report().cloned(),
                prediction_drift_outcome: QueryContextPredictionDriftOutcome::WithinBudget,
                historical_admission_class: None,
                historical_materialization_cost_class: None,
                requested_path_identity: None,
                admitted_path_identity: None,
                resolved_path_identity: None,
                materialization_path_identity: None,
                preview_provenance_identity: None,
                materialized_fact_posture: None,
                counters: QueryContextExecutionCounters {
                    context_execution_count: 1,
                    materialized_row_count: execution.rows().len(),
                    result_shape_width: preflight.plan().result_shape().binding_count(),
                    executor_rediscovery_count: 0,
                },
            })
        }
        super::basis::QueryBasisBindingEvidenceView::Historical {
            query_preflight,
            admission,
            metadata,
        } => {
            let requested_path_class = admission.requested_path().requested_path_class().as_str();
            let materialization_identity = materialization_path_identity(metadata);
            let requested_path_identity = requested_path_identity(metadata);
            let admitted_path_identity = admitted_path_identity(metadata);
            let resolved_path_identity = resolved_path_identity(metadata);
            let historical_admission_class = context
                .historical_admission_class()
                .cloned()
                .ok_or_else(|| {
                    QueryContextAdmissionError::new(
                        QueryContextAdmissionFailureClass::UnsupportedHistoricalBasis,
                        "historical execution requires an admitted historical basis class",
                        QueryContextCounters::for_denial(true, false),
                    )
                })?;
            let historical_materialization_cost_class = context
                .historical_materialization_cost_class()
                .cloned()
                .ok_or_else(|| {
                    QueryContextAdmissionError::new(
                        QueryContextAdmissionFailureClass::UnsupportedHistoricalMaterializationPathClass,
                        "historical execution requires explicit materialization cost posture",
                        QueryContextCounters::for_denial(true, false),
                    )
                })?;
            if historical_admission_class == HistoricalAdmissionClass::RuntimeReconstruction
                && query_preflight.plan().result_shape().binding_count() > 1
            {
                return Err(QueryContextAdmissionError::new(
                    QueryContextAdmissionFailureClass::HistoricalPathTooBroadDenied,
                    "historical execution denies reconstruction lanes that would broaden beyond the admitted narrow result shape",
                    QueryContextCounters::for_historical_broadening_denial(),
                ));
            }
            let rows = synthetic::synthetic_rows(
                query_preflight,
                context.basis_digest(),
                Some(requested_path_class),
                Some(materialization_identity.as_str()),
            );
            let result_digest = synthetic::synthetic_result_digest(
                query_preflight,
                context.basis_digest(),
                &rows,
                Some(requested_path_class),
                Some(materialization_identity.as_str()),
            );
            Ok(QueryContextExecutionArtifact {
                query_digest: context.query_digest().to_string(),
                basis_digest: context.basis_digest().to_string(),
                result_digest: result_digest.as_str().to_string(),
                result_shape_digest: query_preflight
                    .plan()
                    .result_shape()
                    .canonical_result_shape_digest()
                    .as_str()
                    .to_string(),
                rows,
                family: QueryContextExecutionFamily::HistoricalMaterialized,
                cost_class: context.cost_class().clone(),
                budget_class: context.budget_class().clone(),
                prediction_report: context.prediction_report().cloned(),
                prediction_drift_outcome: QueryContextPredictionDriftOutcome::WithinBudget,
                historical_admission_class: Some(historical_admission_class),
                historical_materialization_cost_class: Some(historical_materialization_cost_class),
                requested_path_identity: Some(requested_path_identity),
                admitted_path_identity: Some(admitted_path_identity),
                resolved_path_identity: Some(resolved_path_identity),
                materialization_path_identity: Some(materialization_identity),
                preview_provenance_identity: None,
                materialized_fact_posture: None,
                counters: QueryContextExecutionCounters {
                    context_execution_count: 1,
                    materialized_row_count: query_preflight.plan().result_shape().binding_count(),
                    result_shape_width: query_preflight.plan().result_shape().binding_count(),
                    executor_rediscovery_count: 0,
                },
            })
        }
        super::basis::QueryBasisBindingEvidenceView::PreviewDerived { foundation } => {
            let preview_identity = context
                .preview_provenance_identity_source()
                .map(ToString::to_string)
                .ok_or_else(|| {
                    QueryContextAdmissionError::new(
                        QueryContextAdmissionFailureClass::PreviewProvenanceRequired,
                        "preview-derived execution requires explicit admitted preview provenance",
                        QueryContextCounters::for_denial(false, false),
                    )
                })?;
            let rows = (0..foundation.shape_check_width())
                .map(|index| {
                    format!(
                        "preview:{}:{}:{}:{}",
                        foundation.validated_query_digest().as_str(),
                        context.basis_digest(),
                        preview_identity,
                        index
                    )
                })
                .collect::<Vec<_>>();
            let materialized_row_count = rows.len();
            let result_digest = ResultDigest::from_parts(
                &rows
                    .iter()
                    .enumerate()
                    .map(|(index, value)| format!("row:{}:{}", index, value))
                    .chain(std::iter::once(format!("query:{}", context.query_digest())))
                    .chain(std::iter::once(format!("basis:{}", context.basis_digest())))
                    .chain(std::iter::once(format!("preview:{}", preview_identity)))
                    .chain(std::iter::once(format!(
                        "foundation:{}",
                        foundation.artifact_for_reporting()
                    )))
                    .collect::<Vec<_>>(),
            );
            Ok(QueryContextExecutionArtifact {
                query_digest: context.query_digest().to_string(),
                basis_digest: context.basis_digest().to_string(),
                result_digest: result_digest.as_str().to_string(),
                result_shape_digest: compose_preview_derived_result_shape_digest(
                    foundation.validated_query_digest().as_str(),
                    foundation.shape_check_width(),
                )
                .map_err(|cause| {
                    QueryContextAdmissionError::identity_refused(
                        cause,
                        context.binding().counters().clone(),
                    )
                })?,
                rows,
                family: QueryContextExecutionFamily::PreviewDerivedHistorical,
                cost_class: context.cost_class().clone(),
                budget_class: context.budget_class().clone(),
                prediction_report: context.prediction_report().cloned(),
                prediction_drift_outcome: QueryContextPredictionDriftOutcome::WithinBudget,
                historical_admission_class: None,
                historical_materialization_cost_class: None,
                requested_path_identity: None,
                admitted_path_identity: None,
                resolved_path_identity: None,
                materialization_path_identity: None,
                preview_provenance_identity: Some(preview_identity),
                materialized_fact_posture: None,
                counters: QueryContextExecutionCounters {
                    context_execution_count: 1,
                    materialized_row_count,
                    result_shape_width: foundation.shape_check_width(),
                    executor_rediscovery_count: 0,
                },
            })
        }
    }
}

fn preflight_is_count_aggregate(preflight: &crate::basis::ExecutionPreflightBundle) -> bool {
    preflight.plan().collection().is_some_and(|collection| {
        matches!(
            collection.planning_context().result_family(),
            crate::collection::CollectionResultFamily::CountAggregate
        )
    })
}
