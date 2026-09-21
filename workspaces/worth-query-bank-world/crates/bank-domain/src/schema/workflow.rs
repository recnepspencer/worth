use worth_query_decl::facade::{
    application_capability::{ApplicationCapabilityMarkerIdentity, ApplicationCapabilityRef},
    application_program::{ApplicationWorkflowSpec, ApplicationWorkflowSpecIdentity},
    portable_identity::WorthQueryPortableType,
};

use super::BankSchema;

#[path = "workflow/assessment.rs"]
mod assessment;
mod assessment_readiness;
#[path = "workflow/authority.rs"]
mod authority;
#[path = "workflow/control.rs"]
mod control;
mod settlement;

pub use assessment::*;
pub use assessment_readiness::*;
pub use authority::*;
pub use control::*;
pub use settlement::*;

pub struct ApprovedBusinessPaymentWorkflow;

impl ApplicationWorkflowSpec for ApprovedBusinessPaymentWorkflow {
    type Schema = BankSchema;

    const IDENTITY: ApplicationWorkflowSpecIdentity =
        ApplicationWorkflowSpecIdentity::new("worth.bank.approved-business-payment-workflow.v1");
}

macro_rules! payment_workflow_capability {
    ($name:ident, $identifier:literal, $portable:literal) => {
        pub struct $name;

        impl WorthQueryPortableType for $name {
            const PORTABLE_TYPE_NAME: &'static str = $portable;
        }

        impl ApplicationCapabilityMarkerIdentity for $name {
            type Schema = BankSchema;

            const IDENTIFIER: &'static str = $identifier;
        }

        impl $name {
            pub const fn reference() -> ApplicationCapabilityRef<BankSchema, Self> {
                ApplicationCapabilityRef::from_declaration()
            }
        }
    };
}

payment_workflow_capability!(
    ApprovedBusinessPaymentApproval,
    "ApprovedBusinessPaymentApproval",
    "worth.bank.approved-business-payment-approval.v1"
);
payment_workflow_capability!(
    ApprovedBusinessPaymentAuthoring,
    "ApprovedBusinessPaymentAuthoring",
    "worth.bank.approved-business-payment-authoring.v1"
);
payment_workflow_capability!(
    ApprovedBusinessPaymentInstanceStart,
    "ApprovedBusinessPaymentInstanceStart",
    "worth.bank.approved-business-payment-instance-start.v1"
);
payment_workflow_capability!(
    ApprovedBusinessPaymentAdvance,
    "ApprovedBusinessPaymentAdvance",
    "worth.bank.approved-business-payment-advance.v1"
);
