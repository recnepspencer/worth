use worth_query_decl::facade::application_program::{
    ApplicationWorkflowSpec, ApplicationWorkflowSpecIdentity,
};

use super::BankSchema;

#[path = "workflow/assessment.rs"]
mod assessment;
mod assessment_readiness;
#[path = "workflow/authority.rs"]
mod authority;
#[path = "workflow/control.rs"]
mod control;
mod grant_minting;
mod observation;
mod rejection_assessment;
mod rejection_assessment_binding;
mod rejection_readiness;
mod rejection_readiness_predicate;
mod settlement;
pub use observation::ApprovedBusinessPaymentRead;

pub use assessment::*;
pub use assessment_readiness::*;
pub use authority::*;
pub use control::*;
pub(crate) use grant_minting::install_payment_approval_grant_minting;
pub use rejection_assessment::*;
pub use rejection_assessment_binding::*;
pub use rejection_readiness::*;
pub use rejection_readiness_predicate::RejectedPaymentAssessmentPredicate;
pub use settlement::*;

pub struct ApprovedBusinessPaymentWorkflow;

impl ApplicationWorkflowSpec for ApprovedBusinessPaymentWorkflow {
    fn observation_policy(
    ) -> worth_query_decl::facade::application_program::ApplicationWorkflowObservationPolicy {
        observation::policy()
    }
    fn actor_policy(
    ) -> worth_query_decl::facade::application_program::ApplicationWorkflowActorPolicy {
        worth_query_decl::facade::application_program::ApplicationWorkflowActorPolicy::not_applicable()
    }

    type Schema = BankSchema;

    const IDENTITY: ApplicationWorkflowSpecIdentity =
        ApplicationWorkflowSpecIdentity::new("worth.bank.approved-business-payment-workflow.v1");
}
