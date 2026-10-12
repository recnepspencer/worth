use bank_domain::{
    estate::{
        EstateAction, EstateCapabilityDelegationRequest, EstateCapabilityOperation,
        EstateCapabilityPurpose,
    },
    schema::*,
};
use worth_query_host::facade::{
    declaration::application_schema::TypedMutationPreconditions,
    domain::WorthQueryInstalledApplicationOperation,
};

use super::activation::{AdmittedDelegation, DelegationAccess};
use crate::{estate_progression::BankEstateProgressionDenial, BankIdentityRuntime};

pub(super) fn authorize_target(
    phase: &worth_query_host::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    runtime: &BankIdentityRuntime,
    access: DelegationAccess,
    operation: &WorthQueryInstalledApplicationOperation<
        BankSchema,
        DelegateEstateCapabilityOperation,
        EstateAction,
    >,
    child: EstateCapabilityDelegationRequest,
) -> Result<AdmittedDelegation, BankEstateProgressionDenial> {
    macro_rules! authorize {
        ($capability:ty, $operation:ty) => {{
            let target = runtime
                .application_runtime()
                .installed_schema()
                .capability(<$capability>::reference(), <$operation>::reference())
                .map_err(BankEstateProgressionDenial::from_capability_installation)?;
            runtime
                .application_runtime()
                .authorize_capability_delegation(
                    &phase,
                    access,
                    &target,
                    operation,
                    TypedMutationPreconditions::default(),
                )
                .map_err(BankEstateProgressionDenial::from_authorization)
        }};
    }
    match (child.scope.operation, child.scope.purpose) {
        (EstateCapabilityOperation::NotifyDeath, EstateCapabilityPurpose::EstateAdministration) => {
            authorize!(NotifyDeathEstateCapability, NotifyDeathEstateOperation)
        }
        (
            EstateCapabilityOperation::RetransmitDeathNotice,
            EstateCapabilityPurpose::EstateAdministration,
        ) => authorize!(
            RetransmitDeathNoticeEstateCapability,
            RetransmitDeathNoticeEstateOperation
        ),
        (
            EstateCapabilityOperation::FreezeAccount,
            EstateCapabilityPurpose::EstateAdministration,
        ) => authorize!(FreezeEstateAccountCapability, FreezeEstateAccountOperation),
        (
            EstateCapabilityOperation::OpenEstateCase,
            EstateCapabilityPurpose::EstateAdministration,
        ) => authorize!(OpenEstateCaseCapability, OpenEstateCaseOperation),
        (
            EstateCapabilityOperation::RecognizeExecutor,
            EstateCapabilityPurpose::LegalCompliance,
        ) => authorize!(
            RecognizeEstateExecutorCapability,
            RecognizeEstateExecutorOperation
        ),
        (
            EstateCapabilityOperation::ReleaseEstate,
            EstateCapabilityPurpose::EstateAdministration,
        ) => authorize!(ReleaseEstateCapability, ReleaseEstateOperation),
        (
            EstateCapabilityOperation::DisburseEstate,
            EstateCapabilityPurpose::EstateDisbursement,
        ) => authorize!(DisburseEstateCapability, DisburseEstateOperation),
        (
            EstateCapabilityOperation::ViewRestrictedEstate,
            EstateCapabilityPurpose::EstateAdministration,
        ) => authorize!(
            ViewEstateAdministrationCapability,
            ViewRestrictedEstateOperation
        ),
        (
            EstateCapabilityOperation::ViewRestrictedEstate,
            EstateCapabilityPurpose::IdentityVerification,
        ) => authorize!(
            ViewEstateIdentityVerificationCapability,
            ViewRestrictedEstateOperation
        ),
        (
            EstateCapabilityOperation::ViewRestrictedEstate,
            EstateCapabilityPurpose::LegalCompliance,
        ) => authorize!(
            ViewEstateLegalComplianceCapability,
            ViewRestrictedEstateOperation
        ),
        (
            EstateCapabilityOperation::ViewRestrictedEstate,
            EstateCapabilityPurpose::EmergencyProtection,
        ) => authorize!(
            ViewEstateEmergencyProtectionCapability,
            ViewRestrictedEstateOperation
        ),
        (
            EstateCapabilityOperation::ViewRestrictedEstate,
            EstateCapabilityPurpose::MandatoryReview,
        ) => authorize!(
            ViewEstateMandatoryReviewCapability,
            ViewRestrictedEstateOperation
        ),
        _ => Err(BankEstateProgressionDenial::CommandInput(
            "delegated capability target",
        )),
    }
}
