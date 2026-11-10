use crate::basis::BasisAuthorityFamily;
use crate::basis::ExecutionPreflightBundle;
use crate::historical::{
    HistoricalEvaluationAdmission, HistoricalMaterializationPathMetadata,
    RequestedHistoricalPathClass, ResolvedHistoricalPathClass,
};
use crate::preview::AdmittedPreviewWorkflowFoundation;

use super::super::performance::{
    HistoricalMaterializationCostClass, QueryContextBudgetClass, QueryContextCostClass,
    QueryContextCounters, QueryContextPredictionDriftOutcome, QueryContextPredictionReport,
};
use super::types::{
    HistoricalAdmissionClass, QueryBasisContextRequest, QueryContextDriftOutcome,
    QueryContextFamily,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum QueryBasisBindingEvidence {
    Runtime {
        preflight: ExecutionPreflightBundle,
    },
    Historical {
        query_preflight: ExecutionPreflightBundle,
        admission: HistoricalEvaluationAdmission,
        metadata: HistoricalMaterializationPathMetadata,
    },
    PreviewDerived {
        foundation: AdmittedPreviewWorkflowFoundation,
    },
}

pub(crate) enum QueryBasisBindingEvidenceView<'a> {
    Runtime {
        preflight: &'a ExecutionPreflightBundle,
    },
    Historical {
        query_preflight: &'a ExecutionPreflightBundle,
        admission: &'a HistoricalEvaluationAdmission,
        metadata: &'a HistoricalMaterializationPathMetadata,
    },
    PreviewDerived {
        foundation: &'a AdmittedPreviewWorkflowFoundation,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryBasisContextBinding {
    pub(super) request: QueryBasisContextRequest,
    pub(super) query_digest: String,
    pub(super) basis_digest: String,
    pub(super) basis_authority_family: BasisAuthorityFamily,
    pub(super) drift_outcome: QueryContextDriftOutcome,
    pub(super) cost_class: QueryContextCostClass,
    pub(super) budget_class: QueryContextBudgetClass,
    pub(super) historical_admission_class: Option<HistoricalAdmissionClass>,
    pub(super) historical_materialization_cost_class: Option<HistoricalMaterializationCostClass>,
    pub(super) materialization_path_identity_source: Option<String>,
    pub(super) preview_provenance_identity_source: Option<String>,
    pub(super) prediction_report: Option<QueryContextPredictionReport>,
    pub(super) prediction_drift_outcome: Option<QueryContextPredictionDriftOutcome>,
    pub(super) evidence: QueryBasisBindingEvidence,
    pub(super) counters: QueryContextCounters,
}

impl QueryBasisContextBinding {
    #[cfg(test)]
    pub(crate) fn request(&self) -> &QueryBasisContextRequest {
        &self.request
    }

    pub fn query_digest(&self) -> &str {
        &self.query_digest
    }

    pub fn basis_digest(&self) -> &str {
        &self.basis_digest
    }

    pub fn basis_authority_family(&self) -> &BasisAuthorityFamily {
        &self.basis_authority_family
    }

    pub fn drift_outcome(&self) -> &QueryContextDriftOutcome {
        &self.drift_outcome
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

    pub fn materialization_path_identity_source(&self) -> Option<&str> {
        self.materialization_path_identity_source.as_deref()
    }

    pub fn preview_provenance_identity_source(&self) -> Option<&str> {
        self.preview_provenance_identity_source.as_deref()
    }

    pub fn prediction_report(&self) -> Option<&QueryContextPredictionReport> {
        self.prediction_report.as_ref()
    }

    pub fn prediction_drift_outcome(&self) -> Option<&QueryContextPredictionDriftOutcome> {
        self.prediction_drift_outcome.as_ref()
    }

    pub fn counters(&self) -> &QueryContextCounters {
        &self.counters
    }

    pub(crate) fn evidence(&self) -> QueryBasisBindingEvidenceView<'_> {
        match &self.evidence {
            QueryBasisBindingEvidence::Runtime { preflight } => {
                QueryBasisBindingEvidenceView::Runtime { preflight }
            }
            QueryBasisBindingEvidence::Historical {
                query_preflight,
                admission,
                metadata,
            } => QueryBasisBindingEvidenceView::Historical {
                query_preflight,
                admission,
                metadata,
            },
            QueryBasisBindingEvidence::PreviewDerived { foundation } => {
                QueryBasisBindingEvidenceView::PreviewDerived { foundation }
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmittedQueryBasisContext {
    pub(super) binding: QueryBasisContextBinding,
}

impl AdmittedQueryBasisContext {
    pub fn family(&self) -> &QueryContextFamily {
        self.binding.request.family()
    }

    pub fn declared_basis_label(&self) -> &str {
        self.binding.request.declared_basis_label()
    }

    pub fn query_digest(&self) -> &str {
        self.binding.query_digest()
    }

    pub fn basis_digest(&self) -> &str {
        self.binding.basis_digest()
    }

    pub fn basis_authority_family(&self) -> &BasisAuthorityFamily {
        self.binding.basis_authority_family()
    }

    pub fn cost_class(&self) -> &QueryContextCostClass {
        self.binding.cost_class()
    }

    pub fn budget_class(&self) -> &QueryContextBudgetClass {
        self.binding.budget_class()
    }

    pub fn historical_admission_class(&self) -> Option<&HistoricalAdmissionClass> {
        self.binding.historical_admission_class()
    }

    pub fn historical_materialization_cost_class(
        &self,
    ) -> Option<&HistoricalMaterializationCostClass> {
        self.binding.historical_materialization_cost_class()
    }

    pub fn materialization_path_identity_source(&self) -> Option<&str> {
        self.binding.materialization_path_identity_source()
    }

    pub fn preview_provenance_identity_source(&self) -> Option<&str> {
        self.binding.preview_provenance_identity_source()
    }

    pub fn prediction_report(&self) -> Option<&QueryContextPredictionReport> {
        self.binding.prediction_report()
    }

    pub fn prediction_drift_outcome(&self) -> Option<&QueryContextPredictionDriftOutcome> {
        self.binding.prediction_drift_outcome()
    }

    pub fn counters(&self) -> &QueryContextCounters {
        self.binding.counters()
    }

    pub fn drift_outcome(&self) -> &QueryContextDriftOutcome {
        self.binding.drift_outcome()
    }

    pub(crate) fn predicted_result_shape_width(&self) -> usize {
        match self.binding.evidence() {
            QueryBasisBindingEvidenceView::Runtime { preflight } => {
                preflight.plan().result_shape().binding_count()
            }
            QueryBasisBindingEvidenceView::Historical {
                query_preflight, ..
            } => query_preflight.plan().result_shape().binding_count(),
            QueryBasisBindingEvidenceView::PreviewDerived { foundation } => {
                foundation.shape_check_width()
            }
        }
    }

    pub(crate) fn admits_runtime_snapshot(
        &self,
        snapshot: &crate::memory_workspace::WorthQuerySnapshotIdentity,
    ) -> bool {
        let QueryBasisBindingEvidenceView::Historical {
            admission,
            metadata,
            ..
        } = self.binding.evidence()
        else {
            return false;
        };
        self.family() == &QueryContextFamily::HistoricalSnapshot
            && self.historical_admission_class() == Some(&HistoricalAdmissionClass::RuntimeRetained)
            && metadata.requested_path_class()
                == &RequestedHistoricalPathClass::RequestedRetainedSnapshotPath
            && metadata.resolved_path_class()
                == &ResolvedHistoricalPathClass::ResolvedRetainedSnapshotPath
            && snapshot
                .matches_admitted_historical_projection(admission.requested_path().basis_identity())
    }

    pub(crate) fn binding(&self) -> &QueryBasisContextBinding {
        &self.binding
    }
}
