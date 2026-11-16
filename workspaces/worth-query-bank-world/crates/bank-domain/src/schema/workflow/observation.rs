//! Workflow managers disclose the same payment scope through existing capability admission.
use super::{
    ApprovedBusinessPaymentAuthoring, ApprovedBusinessPaymentAuthoringOperation,
    ApprovedBusinessPaymentWorkflow,
};
use crate::{
    model::BankPrincipalId,
    schema::{
        ApprovePayment, ApprovePaymentInputBinding, BankPrincipalBinding, BankPrincipalIdBinding,
        BankSchema, ExternalPrincipalMapping, Principal,
    },
};
use worth_query_decl::facade::{
    application_program::*, application_schema::ApplicationPrincipalBindingRef,
};

/// Reuses the installed workflow-management capability, never a captured actor.
pub struct ApprovedBusinessPaymentRead;
impl ApplicationWorkflowReadBinding<BankSchema> for ApprovedBusinessPaymentRead {
    const IDENTITY: &'static str = "worth.bank.approved-business-payment.read.v1";
    type Capability = ApprovedBusinessPaymentAuthoring;
    type Input = ApprovePayment;
    type InputBinding = ApprovePaymentInputBinding;
    type Operation = ApprovedBusinessPaymentAuthoringOperation;
    type PrincipalBinding = BankPrincipalBinding;
    type Mapping = ExternalPrincipalMapping;
    type Principal = Principal;
    type PrincipalIdentity = BankPrincipalId;
    type PrincipalIdentityBinding = BankPrincipalIdBinding;
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        BankSchema,
        BankPrincipalBinding,
        ExternalPrincipalMapping,
        Principal,
        BankPrincipalId,
        BankPrincipalIdBinding,
    > {
        BankPrincipalBinding::reference()
    }
}

pub(super) fn policy() -> ApplicationWorkflowObservationPolicy {
    ApplicationWorkflowObservationPolicy::governed(
        ApplicationWorkflowReadRule::subject::<
            ApprovedBusinessPaymentWorkflow,
            ApprovedBusinessPaymentRead,
        >(),
        ApplicationWorkflowReadRule::definition::<
            ApprovedBusinessPaymentWorkflow,
            ApprovedBusinessPaymentRead,
        >(),
        None,
        ApplicationWorkflowAuditPolicy::NotApplicable,
    )
}
