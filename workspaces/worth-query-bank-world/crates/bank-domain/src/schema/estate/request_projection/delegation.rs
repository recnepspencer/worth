use worth_query_decl::facade::application_capability::{
    ApplicationCapabilityDelegationRequest, ApplicationCapabilityDelegationRequestProjection,
    ApplicationCapabilityDelegationRequestProjectionDenial, ApplicationCapabilityEntitySelector,
    ApplicationCapabilityRelatedEntitySelector, ApplicationCapabilityRequestContext,
    ApplicationCapabilityRequestProjection, ApplicationCapabilityValueBinding,
};

use crate::{
    estate::EstateAction,
    schema::{
        AccountIdentity, BankSchema, BranchIdentityField, CapabilityAccount, CapabilityBranch,
        CapabilityDelegationLimitField, CapabilityGrantIdentityField, CapabilityInstitution,
        CapabilityValidFromField, CapabilityValidThroughField, CapabilityWorkflowStageField,
        EstateActionContext, EstateCase, EstateCaseIdentityField, InstitutionIdentityField,
        PrincipalIdentityField,
    },
};

use super::super::*;

macro_rules! delegation_request {
    ($($capability:ty),+ $(,)?) => {
        $(
            impl ApplicationCapabilityDelegationRequest<BankSchema, $capability> for EstateAction {
                type Scope = EstateCase;
                type Context = EstateActionContext;

                fn delegation_request(
                    &self,
                ) -> Result<
                    ApplicationCapabilityDelegationRequestProjection<
                        BankSchema,
                        Self::Scope,
                        Self::Context,
                    >,
                    ApplicationCapabilityDelegationRequestProjectionDenial,
                > {
                    project(*self)
                }
            }
        )+
    };
}

delegation_request!(
    NotifyDeathEstateCapability,
    RetransmitDeathNoticeEstateCapability,
    FreezeEstateAccountCapability,
    OpenEstateCaseCapability,
    RecognizeEstateExecutorCapability,
    DelegateEstateCapability,
    RevokeEstateCapability,
    RequestEstateEmergencyAccessCapability,
    ApproveEstateEmergencyAccessCapability,
    RevokeEstateEmergencyAccessCapability,
    CompleteEstateMandatoryReviewCapability,
    ReleaseEstateCapability,
    DisburseEstateCapability,
    ViewEstateAdministrationCapability,
    ViewEstateIdentityVerificationCapability,
    ViewEstateLegalComplianceCapability,
    ViewEstateEmergencyProtectionCapability,
    ViewEstateMandatoryReviewCapability,
);

fn project(
    action: EstateAction,
) -> Result<
    ApplicationCapabilityDelegationRequestProjection<BankSchema, EstateCase, EstateActionContext>,
    ApplicationCapabilityDelegationRequestProjectionDenial,
> {
    let EstateAction::DelegateCapability {
        estate,
        parent,
        child,
    } = action
    else {
        return Err(
            ApplicationCapabilityDelegationRequestProjectionDenial::input_variant(
                "DelegateEstateCapabilityOperation",
            ),
        );
    };
    let scope = child.scope;
    if scope.estate != estate {
        return Err(
            ApplicationCapabilityDelegationRequestProjectionDenial::input_variant(
                "delegated capability estate",
            ),
        );
    }
    ApplicationCapabilityDelegationRequestProjection::new(
        target(scope),
        ApplicationCapabilityEntitySelector::new(
            CapabilityGrantIdentityField::reference(),
            crate::schema::encoded_bank_value::<crate::schema::CapabilityGrantIdBinding>(parent),
        ),
        ApplicationCapabilityEntitySelector::new(
            PrincipalIdentityField::reference(),
            crate::schema::encoded_bank_value::<crate::schema::BankPrincipalIdBinding>(
                child.grantee,
            ),
        ),
        format!("bank-capability-grant:{}", child.id.get()),
        ApplicationCapabilityValueBinding::new(
            CapabilityGrantIdentityField::reference(),
            crate::schema::encoded_bank_value::<crate::schema::CapabilityGrantIdBinding>(child.id),
        ),
        ApplicationCapabilityValueBinding::new(
            CapabilityWorkflowStageField::reference(),
            crate::schema::encoded_bank_value::<crate::schema::EstateWorkflowStageBinding>(
                scope.workflow_stage,
            ),
        ),
        ApplicationCapabilityValueBinding::new(
            CapabilityValidFromField::reference(),
            crate::schema::encoded_bank_value::<crate::schema::EstateMomentBinding>(
                scope.validity.not_before(),
            ),
        ),
        ApplicationCapabilityValueBinding::new(
            CapabilityValidThroughField::reference(),
            crate::schema::encoded_bank_value::<crate::schema::EstateMomentBinding>(
                scope.validity.not_after(),
            ),
        ),
        ApplicationCapabilityValueBinding::new(
            CapabilityDelegationLimitField::reference(),
            crate::schema::encoded_bank_value::<crate::schema::DelegationLimitBinding>(
                scope.delegation,
            ),
        ),
        [
            ApplicationCapabilityRelatedEntitySelector::new(
                CapabilityInstitution::reference(),
                ApplicationCapabilityEntitySelector::new(
                    InstitutionIdentityField::reference(),
                    crate::schema::encoded_bank_value::<crate::schema::InstitutionIdBinding>(
                        scope.institution,
                    ),
                ),
            ),
            ApplicationCapabilityRelatedEntitySelector::new(
                CapabilityBranch::reference(),
                ApplicationCapabilityEntitySelector::new(
                    BranchIdentityField::reference(),
                    crate::schema::encoded_bank_value::<crate::schema::BranchIdBinding>(
                        scope.branch,
                    ),
                ),
            ),
        ],
    )
}

fn target(
    scope: crate::estate::EstateCapabilityScope,
) -> ApplicationCapabilityRequestProjection<BankSchema, EstateCase, EstateActionContext> {
    let mut target = ApplicationCapabilityRequestProjection::new(
        ApplicationCapabilityEntitySelector::new(
            EstateCaseIdentityField::reference(),
            crate::schema::encoded_bank_value::<crate::schema::EstateCaseIdBinding>(scope.estate),
        ),
        crate::schema::encoded_bank_value::<crate::schema::EstateCapabilityOperationBinding>(
            scope.operation,
        ),
        crate::schema::encoded_bank_value::<crate::schema::EstateCapabilityPurposeBinding>(
            scope.purpose,
        ),
        ApplicationCapabilityRequestContext::new(EstateActionContext::reference()),
    );
    if let Some(account) = scope.account {
        target = target.related_entity(ApplicationCapabilityRelatedEntitySelector::new(
            CapabilityAccount::reference(),
            ApplicationCapabilityEntitySelector::new(
                AccountIdentity::reference(),
                crate::schema::encoded_bank_value::<crate::schema::AccountIdBinding>(account),
            ),
        ));
    }
    if let Some(field) = scope.field {
        target = target.field(crate::schema::encoded_bank_value::<
            crate::schema::RestrictedBankFieldBinding,
        >(field));
    }
    if let Some(amount) = scope.amount_ceiling {
        target = target.magnitude(crate::schema::encoded_bank_value::<
            crate::schema::UsdMoneyBinding,
        >(amount));
    }
    target
}
