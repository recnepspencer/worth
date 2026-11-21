use super::{WorthQueryOperationAuthorizationDenial, WorthQueryRequestedElevation};
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
use worth_query_declaration::facade::application_schema::{
    ApplicationOperationMarkerIdentity, TypedMutationPreconditions,
};
use worth_query_installation::facade::{
    ApplicationOperationDecisionReadTarget, ApplicationOperationProgramTarget, ApplicationSchema,
    WorthQueryInstalledApplicationOperation,
};
use worth_relational::facade::identity::{EntityId, KindId};

use super::super::capability_registry::{
    WorthQueryElevationLifecycleOperationRole, WorthQueryInstalledCapabilityPlan,
};
use super::transition_contract::{approval_program_targets, lifecycle_decision_reads};

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

#[doc(hidden)]
pub struct WorthQueryElevationApprovalAuthorizationDenialAttemptParts {
    pub denial: WorthQueryOperationAuthorizationDenial,
    pub requested: WorthQueryRequestedElevation,
}

impl WorthQueryElevationApprovalAuthorizationDenial {
    #[doc(hidden)]
    pub fn from_attempt_parts(
        _seal: crate::ports::WorthQueryAttemptCrateSeal,
        parts: WorthQueryElevationApprovalAuthorizationDenialAttemptParts,
    ) -> Self {
        Self {
            denial: parts.denial,
            requested: parts.requested,
        }
    }
}
