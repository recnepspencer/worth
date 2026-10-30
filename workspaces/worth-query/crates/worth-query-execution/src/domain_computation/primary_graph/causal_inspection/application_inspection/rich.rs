//! Explicit rich selection and descriptive views; these values grant no read permission.
use super::super::{
    inventory::CausalEvidenceFamily, materialization::CausalInspectionMaterializationPolicy,
};
pub use crate::domain_computation::primary_graph::application_query::{
    WorthQueryMaterializedFieldValue, WorthQueryMaterializedQueryField,
    WorthQueryMaterializedQueryProjection, WorthQueryMaterializedQueryRelation,
    WorthQueryMaterializedQueryRow, WorthQueryProjectionMaterializationRefusal,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationMaterializationRequest {
    families: u64,
}
impl WorthQueryApplicationMaterializationRequest {
    pub const fn family(family: CausalEvidenceFamily) -> Self {
        Self {
            families: 1 << (family as u32),
        }
    }
    pub const fn with_family(mut self, family: CausalEvidenceFamily) -> Self {
        self.families |= 1 << (family as u32);
        self
    }
    pub const fn requests(self, family: CausalEvidenceFamily) -> bool {
        self.families & (1 << (family as u32)) != 0
    }
    pub fn requested_families(self) -> impl Iterator<Item = CausalEvidenceFamily> {
        CausalEvidenceFamily::ALL
            .into_iter()
            .filter(move |family| self.requests(*family))
    }
    pub const fn policy(self) -> CausalInspectionMaterializationPolicy {
        CausalInspectionMaterializationPolicy::OfflineInterpretableArtifact
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryMaterializedFamilyOutcome {
    NotRequested,
    Provided,
    OriginalProjectionNotRetained,
    RequestedButNotProvidedBySelectedObservationMode,
}
pub(super) enum OriginalQueryDetail {
    NotRequested,
    NotRetained,
    Provided(WorthQueryMaterializedQueryProjection),
}
pub struct WorthQueryApplicationRichMaterialization {
    pub(super) request: WorthQueryApplicationMaterializationRequest,
    pub(super) original: OriginalQueryDetail,
}
impl WorthQueryApplicationRichMaterialization {
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
            OriginalQueryDetail::NotRetained => Outcome::OriginalProjectionNotRetained,
            OriginalQueryDetail::Provided(_) => Outcome::Provided,
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
