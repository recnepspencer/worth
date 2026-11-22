//! Explicit rich selection and descriptive views; these values grant no read permission.
use super::super::{
    inventory::CausalEvidenceFamily, materialization::CausalInspectionMaterializationPolicy,
};
pub use crate::domain_computation::primary_graph::application_query::{
    WorthQueryMaterializedFieldValue, WorthQueryMaterializedQueryField,
    WorthQueryMaterializedQueryProjection, WorthQueryMaterializedQueryRelation,
    WorthQueryMaterializedQueryRow, WorthQueryProjectionMaterializationRefusal,
};

pub(super) mod source;
use super::connection::{
    ProducerExplanation, WorthQueryApplicationProducerConnection,
    WorthQueryApplicationProducerEvaluation,
};
use source::SourceDetail;
pub use source::{
    WorthQueryMaterializedSourceEffects, WorthQueryMaterializedSourceField,
    WorthQueryMaterializedSourceFieldEffect,
};

pub use worth_query_execution_query::facade::domain_computation::primary_graph::causal_inspection::WorthQueryApplicationMaterializationRequest;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryMaterializedFamilyOutcome {
    NotRequested,
    Provided,
    OriginalProjectionNotRetained,
    UnsupportedSourceProjectionShape,
    RequestedButNotProvidedBySelectedObservationMode,
}
pub(super) enum OriginalQueryDetail {
    NotRequested,
    Unselected,
    NotRetained,
    Provided(WorthQueryMaterializedQueryProjection),
}
pub struct WorthQueryApplicationRichMaterialization {
    pub(super) request: WorthQueryApplicationMaterializationRequest,
    pub(super) original: OriginalQueryDetail,
    pub(super) source: SourceDetail,
    pub(super) producer: Option<ProducerExplanation>,
    pub(super) history: super::history::HistoryDetail,
}
impl WorthQueryApplicationRichMaterialization {
    pub fn conditional_history_outcome(
        &self,
    ) -> super::history::WorthQueryConditionalHistoryOutcome {
        self.history.outcome()
    }
    pub fn conditional_history(
        &self,
    ) -> Option<&super::history::WorthQueryMaterializedConditionalHistory> {
        match &self.history {
            super::history::HistoryDetail::Provided(view) => Some(view),
            _ => None,
        }
    }

    pub fn request(&self) -> WorthQueryApplicationMaterializationRequest {
        self.request
    }
    pub fn policy(&self) -> CausalInspectionMaterializationPolicy {
        self.request.policy()
    }
    pub fn family_outcome(
        &self,
        family: CausalEvidenceFamily,
    ) -> WorthQueryMaterializedFamilyOutcome {
        use WorthQueryMaterializedFamilyOutcome as Outcome;
        if !self.request.requests(family) {
            return Outcome::NotRequested;
        }
        if family != CausalEvidenceFamily::QueryInspection {
            return Outcome::RequestedButNotProvidedBySelectedObservationMode;
        }
        match self.original {
            OriginalQueryDetail::NotRequested => Outcome::NotRequested,
            OriginalQueryDetail::Unselected => {
                Outcome::RequestedButNotProvidedBySelectedObservationMode
            }
            OriginalQueryDetail::NotRetained => Outcome::OriginalProjectionNotRetained,
            OriginalQueryDetail::Provided(_) => Outcome::Provided,
        }
    }
    pub fn source_outcome(&self) -> WorthQueryMaterializedFamilyOutcome {
        self.source.outcome()
    }
    pub fn source_effects(&self) -> Option<&WorthQueryMaterializedSourceEffects> {
        match &self.source {
            SourceDetail::Provided(source) => Some(source),
            _ => None,
        }
    }
    pub fn producer_outcome(&self) -> WorthQueryMaterializedFamilyOutcome {
        if !self.request.requests_producer_evidence() {
            WorthQueryMaterializedFamilyOutcome::NotRequested
        } else if self.producer.is_some() {
            WorthQueryMaterializedFamilyOutcome::Provided
        } else {
            WorthQueryMaterializedFamilyOutcome::RequestedButNotProvidedBySelectedObservationMode
        }
    }
    pub fn producer_connection(&self) -> Option<&WorthQueryApplicationProducerConnection> {
        match &self.producer {
            Some(ProducerExplanation::Creation(connection)) => Some(connection),
            _ => None,
        }
    }
    pub fn producer_evaluation(&self) -> Option<&WorthQueryApplicationProducerEvaluation> {
        match &self.producer {
            Some(ProducerExplanation::Evaluation(evaluation)) => Some(evaluation),
            _ => None,
        }
    }
    pub(super) fn producer_label(&self) -> &'static str {
        match &self.producer {
            Some(ProducerExplanation::Creation(_)) => "provided_creation",
            Some(ProducerExplanation::Evaluation(_)) => "provided_output_evaluation",
            _ if self.request.requests_producer_evidence() => "not_selected",
            _ => "not_requested",
        }
    }
    pub fn original_query(&self) -> Option<&WorthQueryMaterializedQueryProjection> {
        match &self.original {
            OriginalQueryDetail::Provided(value) => Some(value),
            _ => None,
        }
    }
    pub(super) fn projection_label(&self) -> &'static str {
        match self.original {
            OriginalQueryDetail::NotRequested => "not_requested",
            OriginalQueryDetail::Unselected => "not_selected",
            OriginalQueryDetail::NotRetained => "original_projection_not_retained",
            OriginalQueryDetail::Provided(_) => "provided_original_projection",
        }
    }
}
/// A fixed report reference already assembled after fresh Query admission.
pub struct WorthQueryMaterializedEvidenceReference<'a> {
    pub(super) binding: &'a worth_runtime_bridge::facade::BridgeCausalEvidenceBinding,
}
impl WorthQueryMaterializedEvidenceReference<'_> {
    pub fn owner(&self) -> &'static str {
        self.binding.owner().as_str()
    }
    pub fn family(&self) -> &'static str {
        self.binding.family().as_str()
    }
    pub fn reference_identity(&self) -> &str {
        self.binding.reference_for_reporting()
    }
    pub fn binding_identity(&self) -> &str {
        self.binding.binding_digest_for_reporting()
    }
    pub fn retained_record_identity(&self) -> Option<&str> {
        self.binding.retained_record_digest_for_reporting()
    }
}

pub use super::super::inventory::CausalEvidenceFamily as WorthQueryApplicationEvidenceFamily;
pub use super::super::materialization::CausalInspectionMaterializationPolicy as WorthQueryApplicationMaterializationPolicy;

const _: () = assert!(CausalEvidenceFamily::ALL.len() <= 64);

mod court_doors_for_test;

pub use source::SourceMaterializationDenial;
