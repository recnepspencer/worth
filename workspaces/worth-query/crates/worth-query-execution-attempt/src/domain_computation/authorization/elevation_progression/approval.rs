use super::{WorthQueryOperationAuthorizationDenial, WorthQueryRequestedElevation};
use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
use worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity;
use worth_query_installation::facade::{
    ApplicationOperationDecisionReadTarget, ApplicationOperationProgramTarget, ApplicationSchema,
};
use worth_relational::facade::identity::{EntityId, KindId};

pub mod binding;
pub use binding::WorthQueryElevationApprovalBinding;

pub struct WorthQueryElevationApprovalBindingPermit(());

impl WorthQueryElevationApprovalBindingPermit {
    fn mint() -> Self {
        Self(())
    }
}

pub struct WorthQueryElevationApprovalDraft {
    pub elevation: EntityId,
    pub review: EntityId,
    pub approver: EntityId,
    pub approved_status: AspectValue,
    pub elevation_entity: String,
    pub status_field: AspectFieldLocator,
    pub approver_relation: KindId,
    pub reviewer_relation: KindId,
    pub required_decision_reads: Vec<ApplicationOperationDecisionReadTarget>,
    pub required_program_targets: Vec<ApplicationOperationProgramTarget>,
    pub lifecycle_effect: Option<worth_query_declaration::lifecycle_effect_derivation_authority::DerivedApplicationCapabilityLifecycleEffect>,
}

/// Refusal to authorize approving a requested elevation.
///
/// Nothing was approved. [`Self::denial`] says why, and
/// [`Self::into_requested`] returns the requested elevation so the caller can
/// try again or replay a recorded approval.
#[derive(Debug)]
pub struct WorthQueryElevationApprovalAuthorizationDenial {
    denial: WorthQueryOperationAuthorizationDenial,
    requested: WorthQueryRequestedElevation,
}

impl WorthQueryElevationApprovalAuthorizationDenial {
    pub const fn denial(&self) -> &WorthQueryOperationAuthorizationDenial {
        &self.denial
    }

    pub fn into_requested(self) -> WorthQueryRequestedElevation {
        self.requested
    }
}

impl WorthQueryElevationApprovalAuthorizationDenial {
    pub fn new(
        denial: WorthQueryOperationAuthorizationDenial,
        requested: WorthQueryRequestedElevation,
    ) -> Self {
        Self { denial, requested }
    }
}
