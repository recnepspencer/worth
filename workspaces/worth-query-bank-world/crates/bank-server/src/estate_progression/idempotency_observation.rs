//! Current Bank authority observes a lifecycle key without minting phase custody.
use crate::{BankAuthenticatedPrincipal, BankEstateProgressionDenial, BankIdentityRuntime};
use bank_domain::{
    estate::EstateAction,
    proposals::BankIdempotencyKey,
    schema::{
        ApproveEstateEmergencyAccessCapability, ApproveEstateEmergencyAccessOperation,
        BankPrincipalBinding, BankSchema, CompleteEstateMandatoryReviewCapability,
        CompleteEstateMandatoryReviewOperation, EstateCase, RequestEstateEmergencyAccessCapability,
        RequestEstateEmergencyAccessOperation, RevokeEstateEmergencyAccessCapability,
        RevokeEstateEmergencyAccessOperation,
    },
};
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryRequestScope,
    application_entry::{
        WorthQueryApplicationRequestMutationDenial,
        WorthQueryCapabilityLifecycleIdempotencyDenial as Denial,
    },
    declaration::{
        application_capability::{ApplicationCapabilityRef, ApplicationCapabilityRequest},
        application_schema::{
            ApplicationOperationMarkerIdentity, ApplicationOperationRef,
            ApplicationStructuredValueBinding, TypedMutationPreconditions,
        },
    },
    primary_graph::WorthQueryApplicationIdempotencyResolution,
};
impl BankIdentityRuntime {
    /// Resolves original immutable meaning under fresh Bank authorization. A
    /// recorded answer is not a Requested, Approved or Mandatory stage owner.
    pub fn observe_estate_lifecycle_idempotency(
        &self,
        principal: &BankAuthenticatedPrincipal,
        action: EstateAction,
        key: &BankIdempotencyKey,
        request: &WorthQueryRequestScope,
    ) -> Result<WorthQueryApplicationIdempotencyResolution, BankEstateProgressionDenial> {
        match action {
            EstateAction::RequestEmergencyAccess { .. } => self.observe_lifecycle_operation(
                principal,
                action,
                key,
                request,
                RequestEstateEmergencyAccessCapability::reference(),
                RequestEstateEmergencyAccessOperation::reference(),
            ),
            EstateAction::ApproveEmergencyAccess { .. } => self.observe_lifecycle_operation(
                principal,
                action,
                key,
                request,
                ApproveEstateEmergencyAccessCapability::reference(),
                ApproveEstateEmergencyAccessOperation::reference(),
            ),
            EstateAction::RevokeEmergencyAccess { .. } => self.observe_lifecycle_operation(
                principal,
                action,
                key,
                request,
                RevokeEstateEmergencyAccessCapability::reference(),
                RevokeEstateEmergencyAccessOperation::reference(),
            ),
            EstateAction::CompleteMandatoryReview { .. } => self.observe_lifecycle_operation(
                principal,
                action,
                key,
                request,
                CompleteEstateMandatoryReviewCapability::reference(),
                CompleteEstateMandatoryReviewOperation::reference(),
            ),
            EstateAction::NotifyDeath { .. }
            | EstateAction::RetransmitDeathNotice { .. }
            | EstateAction::FreezeAccount { .. }
            | EstateAction::OpenEstateCase { .. }
            | EstateAction::RecognizeExecutor { .. }
            | EstateAction::DelegateCapability { .. }
            | EstateAction::RevokeCapability { .. }
            | EstateAction::ReleaseEstate { .. }
            | EstateAction::DisburseEstate(_)
            | EstateAction::ViewRestrictedEstate { .. }
            | EstateAction::ViewRestrictedEstateWithEmergencyAccess { .. } => {
                Err(BankEstateProgressionDenial::CommandInput(
                    "estate-lifecycle-idempotency-observation",
                ))
            }
        }
    }
    fn observe_lifecycle_operation<Capability, Operation>(
        &self,
        principal: &BankAuthenticatedPrincipal,
        action: EstateAction,
        key: &BankIdempotencyKey,
        request: &WorthQueryRequestScope,
        capability: ApplicationCapabilityRef<BankSchema, Capability>,
        operation: ApplicationOperationRef<BankSchema, Operation, EstateAction>,
    ) -> Result<WorthQueryApplicationIdempotencyResolution, BankEstateProgressionDenial>
    where
        Operation: ApplicationOperationMarkerIdentity<BankSchema> + 'static,
        Operation::InputBinding: ApplicationStructuredValueBinding<Value = EstateAction>,
        EstateAction: ApplicationCapabilityRequest<BankSchema, Capability, Scope = EstateCase>,
    {
        self.request(principal, request)
            .observe_capability_lifecycle_idempotency_in_program(
                self.application_program(),
                BankPrincipalBinding::reference(),
                capability,
                operation,
                action,
                key,
                TypedMutationPreconditions::<BankSchema, Operation, EstateCase>::default(),
            )
            .map_err(map_observation_denial)
    }
}
fn map_observation_denial(denial: Denial) -> BankEstateProgressionDenial {
    match denial {
        Denial::ExecutionRequest(cause) => BankEstateProgressionDenial::ApplicationEntry(
            WorthQueryApplicationRequestMutationDenial::ExecutionRequest(cause),
        ),
        Denial::Program(cause) => BankEstateProgressionDenial::ProgramAction(cause),
        Denial::ProgramMismatch => BankEstateProgressionDenial::ProgramMismatch,
        Denial::PrincipalBindingInstallation(cause) => {
            BankEstateProgressionDenial::PrincipalBindingInstallation(cause)
        }
        Denial::IdentityEncoding(cause) => BankEstateProgressionDenial::IdentityEncoding(cause),
        Denial::CapabilityInstallation(cause) => {
            BankEstateProgressionDenial::from_capability_installation(cause)
        }
        Denial::OperationInstallation(cause) => {
            BankEstateProgressionDenial::from_operation_installation(cause)
        }
        Denial::ProductSelection(cause) => {
            BankEstateProgressionDenial::from_product_selection(cause)
        }
        Denial::PrincipalResolution(cause) => {
            BankEstateProgressionDenial::PrincipalResolution(cause)
        }
        Denial::Authorization(cause) => BankEstateProgressionDenial::from_authorization(cause),
        Denial::Idempotency(cause) => BankEstateProgressionDenial::from_idempotency(cause),
    }
}
