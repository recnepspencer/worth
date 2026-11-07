use crate::basis::{BasisAuthorityFamily, ExecutionPreflightBundle};
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::historical::{
    HistoricalCapabilityDescriptor, HistoricalEvaluationAdmission,
    HistoricalMaterializationPathMetadata, RequestedHistoricalPathClass,
    ResolvedHistoricalPathClass,
};
use crate::preview::{AdmittedPreviewWorkflowFoundation, PreviewWorkflowFoundationRequest};
use worth_runtime_bridge::facade::bridge_identity_reporting_label;

use super::super::historical::{
    drift_outcome_for_historical, historical_admission_class, materialization_path_cost_class,
    materialization_path_identity,
};
use super::super::performance::{
    QueryContextBudgetClass, QueryContextCostClass, QueryContextCounters,
    QueryContextPredictionDriftOutcome, QueryContextPredictionReport,
};
use super::binding::{
    AdmittedQueryBasisContext, QueryBasisBindingEvidence, QueryBasisContextBinding,
};
use super::types::{
    HistoricalAdmissionClass, QueryBasisContextRequest, QueryContextAdmissionError,
    QueryContextAdmissionFailureClass, QueryContextDriftOutcome, QueryContextFamily,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueryContextBindingSource<'a> {
    RuntimeCurrent(&'a ExecutionPreflightBundle),
    RuntimeBranch(&'a ExecutionPreflightBundle),
    Historical {
        query_preflight: &'a ExecutionPreflightBundle,
        admission: &'a HistoricalEvaluationAdmission,
        metadata: &'a HistoricalMaterializationPathMetadata,
    },
    HistoricalCapability(&'a HistoricalCapabilityDescriptor),
    PreviewDerivedHistorical(&'a AdmittedPreviewWorkflowFoundation),
}

pub(crate) fn bind_legacy_query_basis_context(
    request: QueryBasisContextRequest,
    source: QueryContextBindingSource<'_>,
) -> Result<QueryBasisContextBinding, QueryContextAdmissionError> {
    match source {
        QueryContextBindingSource::RuntimeCurrent(preflight) => {
            bind_runtime_context(request, preflight, true)
        }
        QueryContextBindingSource::RuntimeBranch(preflight) => {
            bind_runtime_context(request, preflight, false)
        }
        QueryContextBindingSource::Historical {
            query_preflight,
            admission,
            metadata,
        } => bind_historical_context(request, query_preflight, admission, metadata),
        QueryContextBindingSource::HistoricalCapability(capability) => {
            if capability.admitted_path_class().is_none() {
                return Err(QueryContextAdmissionError::new(
                    QueryContextAdmissionFailureClass::StoreBackedHistoricalDeferred,
                    "store-backed historical context remains explicit deferred debt",
                    QueryContextCounters::for_denial(true, false),
                ));
            }

            Err(QueryContextAdmissionError::new(
                QueryContextAdmissionFailureClass::NonQueryOwnedHistoricalArtifact,
                "historical capability descriptors are not admitted query-owned basis artifacts",
                QueryContextCounters::for_denial(true, false),
            ))
        }
        QueryContextBindingSource::PreviewDerivedHistorical(foundation) => {
            bind_preview_context(request, foundation)
        }
    }
}

pub(crate) fn admit_legacy_query_basis_context(
    binding: QueryBasisContextBinding,
) -> Result<AdmittedQueryBasisContext, QueryContextAdmissionError> {
    if matches!(
        binding.drift_outcome(),
        QueryContextDriftOutcome::ExplicitHistoricalDenial
    ) {
        return Err(QueryContextAdmissionError::new(
            QueryContextAdmissionFailureClass::UnsupportedHistoricalBasis,
            "denied historical basis cannot be admitted",
            binding.counters().clone(),
        ));
    }

    Ok(AdmittedQueryBasisContext { binding })
}

pub(crate) fn bind_runtime_context(
    request: QueryBasisContextRequest,
    preflight: &ExecutionPreflightBundle,
    current: bool,
) -> Result<QueryBasisContextBinding, QueryContextAdmissionError> {
    if preflight.basis().identity().authority_family() != &BasisAuthorityFamily::Runtime {
        return Err(QueryContextAdmissionError::new(
            QueryContextAdmissionFailureClass::InvalidBasisPairing,
            "runtime query contexts require runtime-authoritative preflight bundles",
            QueryContextCounters::for_denial(false, false),
        ));
    }

    let family_matches = match (request.family(), current) {
        (QueryContextFamily::CurrentBranchHead, true) => true,
        (QueryContextFamily::BranchHead, false) => true,
        (QueryContextFamily::CurrentBranchHead, false) => false,
        (QueryContextFamily::BranchHead, true) => false,
        (QueryContextFamily::HistoricalSnapshot, _) => false,
        (QueryContextFamily::HistoricalCommit, _) => false,
        (QueryContextFamily::PreviewDerivedHistorical, _) => false,
        (QueryContextFamily::DiffComparison, _) => false,
    };
    if !family_matches {
        return Err(QueryContextAdmissionError::new(
            QueryContextAdmissionFailureClass::InvalidBasisPairing,
            "runtime source does not match the declared query context family",
            QueryContextCounters::for_denial(false, false),
        ));
    }

    let cost_class = if current {
        QueryContextCostClass::CurrentHeadNarrow
    } else {
        QueryContextCostClass::BranchHeadNarrow
    };

    Ok(QueryBasisContextBinding {
        query_digest: preflight
            .plan()
            .query()
            .validated_query_digest()
            .as_str()
            .to_string(),
        basis_digest: preflight.basis().proof().digest().as_str().to_string(),
        basis_authority_family: preflight.basis().identity().authority_family().clone(),
        drift_outcome: QueryContextDriftOutcome::BasisExact,
        cost_class,
        budget_class: QueryContextBudgetClass::NarrowSingleBasis,
        historical_admission_class: None,
        historical_materialization_cost_class: None,
        materialization_path_identity_source: None,
        preview_provenance_identity_source: None,
        prediction_report: Some(QueryContextPredictionReport::for_runtime_binding()),
        prediction_drift_outcome: Some(QueryContextPredictionDriftOutcome::PendingExecution),
        evidence: QueryBasisBindingEvidence::Runtime {
            preflight: preflight.clone(),
        },
        request,
        counters: QueryContextCounters::for_runtime_basis_binding(),
    })
}

fn bind_historical_context(
    request: QueryBasisContextRequest,
    query_preflight: &ExecutionPreflightBundle,
    admission: &HistoricalEvaluationAdmission,
    metadata: &HistoricalMaterializationPathMetadata,
) -> Result<QueryBasisContextBinding, QueryContextAdmissionError> {
    let historical_family = match request.family() {
        QueryContextFamily::HistoricalSnapshot => true,
        QueryContextFamily::HistoricalCommit => true,
        QueryContextFamily::CurrentBranchHead => false,
        QueryContextFamily::BranchHead => false,
        QueryContextFamily::PreviewDerivedHistorical => false,
        QueryContextFamily::DiffComparison => false,
    };
    if !historical_family {
        return Err(QueryContextAdmissionError::new(
            QueryContextAdmissionFailureClass::InvalidBasisPairing,
            "historical evidence can only bind historical query context families",
            QueryContextCounters::for_denial(true, false),
        ));
    }

    if request.declared_basis_label() != admission.requested_path().basis_identity() {
        return Err(QueryContextAdmissionError::new(
            QueryContextAdmissionFailureClass::BasisSubstitutionForbidden,
            "historical context binding forbids basis substitution after admission",
            QueryContextCounters::for_denial(true, true),
        ));
    }

    let admission_class = historical_admission_class(admission);
    let store_authority =
        query_preflight.basis().identity().authority_family() == &BasisAuthorityFamily::Store;
    if store_authority
        && (admission_class != HistoricalAdmissionClass::RuntimeRetained
            || metadata.requested_path_class()
                != &RequestedHistoricalPathClass::RequestedRetainedSnapshotPath
            || metadata.resolved_path_class()
                != &ResolvedHistoricalPathClass::ResolvedRetainedSnapshotPath)
    {
        return Err(QueryContextAdmissionError::new(
            QueryContextAdmissionFailureClass::StoreBackedHistoricalDeferred,
            "store-backed historical query contexts are only admitted for the retained-snapshot slice proven in Milestone 10",
            QueryContextCounters::for_denial(true, false),
        ));
    }
    let materialization_identity = materialization_path_identity(metadata);
    let materialization_cost_class = materialization_path_cost_class(admission);
    let cost_class = match admission_class {
        HistoricalAdmissionClass::RuntimeRetained => {
            QueryContextCostClass::HistoricalRetainedBounded
        }
        HistoricalAdmissionClass::RuntimeReplay => QueryContextCostClass::HistoricalReplayBounded,
        HistoricalAdmissionClass::RuntimeReconstruction => {
            QueryContextCostClass::HistoricalReconstructionBounded
        }
        HistoricalAdmissionClass::StoreDeferredDebt => {
            QueryContextCostClass::HistoricalReconstructionBounded
        }
    };

    Ok(QueryBasisContextBinding {
        query_digest: query_preflight
            .plan()
            .query()
            .validated_query_digest()
            .as_str()
            .to_string(),
        basis_digest: worth_query_evidence_identity(WorthQueryEvidenceScope::BasisDigest)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "query_context_historical_basis_binding_v1",
            )
            .field_value(
                WorthQueryEvidenceTag::new("basis_identity"),
                admission.requested_path().basis_identity(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("requested_path"),
                admission.requested_path().requested_path_class().as_str(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("materialization"),
                &materialization_identity,
            )
            .seal()
            .map_err(|cause| {
                QueryContextAdmissionError::identity_refused(
                    cause,
                    QueryContextCounters::for_identity_refusal(true),
                )
            })?
            .as_str()
            .to_string(),
        basis_authority_family: query_preflight
            .basis()
            .identity()
            .authority_family()
            .clone(),
        drift_outcome: drift_outcome_for_historical(admission),
        cost_class,
        budget_class: QueryContextBudgetClass::HistoricalBounded,
        historical_admission_class: Some(admission_class),
        historical_materialization_cost_class: Some(materialization_cost_class),
        materialization_path_identity_source: Some(materialization_identity),
        preview_provenance_identity_source: None,
        prediction_report: Some(QueryContextPredictionReport::for_historical_binding()),
        prediction_drift_outcome: Some(QueryContextPredictionDriftOutcome::PendingExecution),
        evidence: QueryBasisBindingEvidence::Historical {
            query_preflight: query_preflight.clone(),
            admission: admission.clone(),
            metadata: metadata.clone(),
        },
        request,
        counters: QueryContextCounters::for_historical_basis_binding(),
    })
}

fn bind_preview_context(
    request: QueryBasisContextRequest,
    foundation: &AdmittedPreviewWorkflowFoundation,
) -> Result<QueryBasisContextBinding, QueryContextAdmissionError> {
    let preview_family = match request.family() {
        QueryContextFamily::PreviewDerivedHistorical => true,
        QueryContextFamily::CurrentBranchHead => false,
        QueryContextFamily::BranchHead => false,
        QueryContextFamily::HistoricalSnapshot => false,
        QueryContextFamily::HistoricalCommit => false,
        QueryContextFamily::DiffComparison => false,
    };
    if !preview_family {
        return Err(QueryContextAdmissionError::new(
            QueryContextAdmissionFailureClass::InvalidBasisPairing,
            "preview-derived evidence can only bind preview-derived historical contexts",
            QueryContextCounters::for_denial(false, false),
        ));
    }

    if foundation.request_family() != &PreviewWorkflowFoundationRequest::compare_basis_pair() {
        return Err(QueryContextAdmissionError::new(
            QueryContextAdmissionFailureClass::PreviewProvenanceRequired,
            "preview-derived contexts require compare-basis provenance",
            QueryContextCounters::for_denial(false, false),
        ));
    }

    Ok(QueryBasisContextBinding {
        query_digest: foundation.validated_query_digest().as_str().to_string(),
        basis_digest: foundation.artifact_for_reporting().to_string(),
        basis_authority_family: BasisAuthorityFamily::Runtime,
        drift_outcome: QueryContextDriftOutcome::BasisExact,
        cost_class: QueryContextCostClass::PreviewDerivedHistoricalBounded,
        budget_class: QueryContextBudgetClass::PreviewDerivedBounded,
        historical_admission_class: None,
        historical_materialization_cost_class: None,
        materialization_path_identity_source: None,
        preview_provenance_identity_source: Some(
            bridge_identity_reporting_label(
                &foundation
                    .preview_session_identity()
                    .bridge_admission_evidence(),
            )
            .to_string(),
        ),
        prediction_report: Some(QueryContextPredictionReport::for_preview_binding()),
        prediction_drift_outcome: Some(QueryContextPredictionDriftOutcome::PendingExecution),
        evidence: QueryBasisBindingEvidence::PreviewDerived {
            foundation: foundation.clone(),
        },
        request,
        counters: QueryContextCounters::for_preview_basis_binding(),
    })
}

pub(crate) fn historical_admission_of(
    context: &AdmittedQueryBasisContext,
) -> Option<super::types::HistoricalAdmissionClass> {
    context.historical_admission_class().cloned()
}

pub(crate) fn materialization_identity_of(context: &AdmittedQueryBasisContext) -> Option<String> {
    context
        .materialization_path_identity_source()
        .map(ToString::to_string)
}

pub(crate) fn preview_identity_of(context: &AdmittedQueryBasisContext) -> Option<String> {
    context
        .preview_provenance_identity_source()
        .map(ToString::to_string)
}
