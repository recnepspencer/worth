use super::{AspectFieldLocator, AspectValue, WorthQueryOperationAuthorizationDenial};
use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
use worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity;
use worth_query_installation::facade::{
    ApplicationOperationDecisionReadTarget, ApplicationOperationProgramTarget, ApplicationSchema,
};
use worth_relational::facade::identity::{EntityId, KindId};

use crate::domain_computation::primary_graph::WorthQueryMandatoryReview;

pub mod binding;
pub use binding::WorthQueryMandatoryReviewBinding;

#[derive(Debug)]
pub struct WorthQueryMandatoryReviewDraft {
    pub elevation: EntityId,
    pub review: EntityId,
    pub reviewer: EntityId,
    pub reviewed_at: AspectValue,
    pub reviewed_at_field: AspectFieldLocator,
    pub terminal_status: AspectValue,
    pub completed_status: AspectValue,
    pub review_entity: String,
    pub review_status_field: AspectFieldLocator,
    pub approver_relation: KindId,
    pub reviewer_relation: KindId,
    pub required_decision_reads: Vec<ApplicationOperationDecisionReadTarget>,
    pub required_program_targets: Vec<ApplicationOperationProgramTarget>,
    pub lifecycle_effect: Option<worth_query_declaration::lifecycle_effect_derivation_authority::DerivedApplicationCapabilityLifecycleEffect>,
}

/// Refusal to authorize a mandatory review step.
///
/// The review was not admitted. [`Self::denial`] says why, and
/// [`Self::into_mandatory_review`] returns the pending review unchanged.
#[derive(Debug)]
pub struct WorthQueryMandatoryReviewAuthorizationDenial {
    denial: WorthQueryOperationAuthorizationDenial,
    mandatory: WorthQueryMandatoryReview,
}

impl WorthQueryMandatoryReviewAuthorizationDenial {
    pub const fn denial(&self) -> &WorthQueryOperationAuthorizationDenial {
        &self.denial
    }

    pub fn into_mandatory_review(self) -> WorthQueryMandatoryReview {
        self.mandatory
    }
}

#[doc(hidden)]
pub struct WorthQueryMandatoryReviewAuthorizationDenialAttemptParts {
    pub denial: WorthQueryOperationAuthorizationDenial,
    pub mandatory: WorthQueryMandatoryReview,
}

impl WorthQueryMandatoryReviewAuthorizationDenial {
    #[doc(hidden)]
    pub fn from_attempt_parts(
        _seal: crate::ports::WorthQueryAttemptCrateSeal,
        parts: WorthQueryMandatoryReviewAuthorizationDenialAttemptParts,
    ) -> Self {
        Self {
            denial: parts.denial,
            mandatory: parts.mandatory,
        }
    }
}
