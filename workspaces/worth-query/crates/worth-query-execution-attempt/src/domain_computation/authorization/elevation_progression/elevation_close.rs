use super::WorthQueryOperationAuthorizationDenial;
use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
use worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity;
use worth_query_installation::facade::{
    ApplicationOperationDecisionReadTarget, ApplicationOperationProgramTarget, ApplicationSchema,
};
use worth_relational::facade::identity::EntityId;

use crate::domain_computation::primary_graph::{
    WorthQueryApprovedElevation, WorthQueryElevationClosureKind,
};

pub mod binding;
pub use binding::WorthQueryElevationCloseBinding;

#[derive(Debug)]
pub struct WorthQueryElevationCloseDraft {
    pub elevation: EntityId,
    pub review: EntityId,
    pub closer: EntityId,
    pub closure_kind: WorthQueryElevationClosureKind,
    pub closed_at: AspectValue,
    pub closed_at_field: AspectFieldLocator,
    pub closed_status: AspectValue,
    pub approved_status: AspectValue,
    pub elevation_entity: String,
    pub status_field: AspectFieldLocator,
    pub approver_relation: worth_relational::facade::identity::KindId,
    pub reviewer_relation: worth_relational::facade::identity::KindId,
    pub required_decision_reads: Vec<ApplicationOperationDecisionReadTarget>,
    pub required_program_targets: Vec<ApplicationOperationProgramTarget>,
    pub lifecycle_effect: Option<worth_query_declaration::lifecycle_effect_derivation_authority::DerivedApplicationCapabilityLifecycleEffect>,
}

/// Refusal to authorize closing an approved elevation.
///
/// The elevation was not closed. [`Self::denial`] says why, and
/// [`Self::into_approved`] returns the approved elevation unchanged.
#[derive(Debug)]
pub struct WorthQueryElevationCloseAuthorizationDenial {
    denial: WorthQueryOperationAuthorizationDenial,
    approved: WorthQueryApprovedElevation,
}

impl WorthQueryElevationCloseAuthorizationDenial {
    pub const fn denial(&self) -> &WorthQueryOperationAuthorizationDenial {
        &self.denial
    }

    pub fn into_approved(self) -> WorthQueryApprovedElevation {
        self.approved
    }
}

impl WorthQueryElevationCloseAuthorizationDenial {
    pub fn new(
        denial: WorthQueryOperationAuthorizationDenial,
        approved: WorthQueryApprovedElevation,
    ) -> Self {
        Self { denial, approved }
    }
}
