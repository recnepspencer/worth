use worth_query_decl::facade::application_capability::{
    ApplicationCapabilityElevationRequest, ApplicationCapabilityElevationRequestProjection,
    ApplicationCapabilityElevationRequestProjectionDenial, ApplicationCapabilityEntitySelector,
    ApplicationCapabilityRequest, ApplicationCapabilityRequestContext,
    ApplicationCapabilityRequestProjection, ApplicationCapabilityRequestProjectionDenial,
    ApplicationCapabilityValueBinding,
};

use crate::{
    estate::{EstateAction, EstateCapabilityOperation, EstateCapabilityPurpose},
    schema::{
        ApproveEstateEmergencyAccessCapability, BankSchema, CapabilityGrantIdentityField,
        CompleteEstateMandatoryReviewCapability, EmergencyAccessIdentityField,
        EmergencyAccessReasonField, EstateActionContext, EstateCase, EstateCaseIdentityField,
        EstateEmergencyAccessSlot, EstateMandatoryReviewSlot, MandatoryReviewIdentityField,
        RequestEstateEmergencyAccessCapability, RequestEstateEmergencyAccessOperation,
        RevokeEstateEmergencyAccessCapability,
    },
};

impl ApplicationCapabilityRequest<BankSchema, RequestEstateEmergencyAccessCapability>
    for EstateAction
{
    type Scope = EstateCase;
    type Context = EstateActionContext;

    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<BankSchema, Self::Scope, Self::Context>,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        let EstateAction::RequestEmergencyAccess { estate, .. } = *self else {
            return Err(ApplicationCapabilityRequestProjectionDenial::input_variant(
                "RequestEstateEmergencyAccessOperation",
            ));
        };
        Ok(estate_transition_request(
            self,
            estate,
            ApplicationCapabilityRequestContext::new(EstateActionContext::reference()),
        ))
    }
}

macro_rules! elevation_transition_request {
    ($capability:ty, $variant:ident, $operation:literal) => {
        impl ApplicationCapabilityRequest<BankSchema, $capability> for EstateAction {
            type Scope = EstateCase;
            type Context = EstateActionContext;

            fn capability_request(
                &self,
            ) -> Result<
                ApplicationCapabilityRequestProjection<BankSchema, Self::Scope, Self::Context>,
                ApplicationCapabilityRequestProjectionDenial,
            > {
                let EstateAction::$variant { estate, access } = *self else {
                    return Err(ApplicationCapabilityRequestProjectionDenial::input_variant(
                        $operation,
                    ));
                };
                Ok(estate_transition_request(
                    self,
                    estate,
                    elevation_context(access),
                ))
            }
        }
    };
}

elevation_transition_request!(
    ApproveEstateEmergencyAccessCapability,
    ApproveEmergencyAccess,
    "ApproveEstateEmergencyAccessOperation"
);
elevation_transition_request!(
    RevokeEstateEmergencyAccessCapability,
    RevokeEmergencyAccess,
    "RevokeEstateEmergencyAccessOperation"
);

impl ApplicationCapabilityRequest<BankSchema, CompleteEstateMandatoryReviewCapability>
    for EstateAction
{
    type Scope = EstateCase;
    type Context = EstateActionContext;

    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<BankSchema, Self::Scope, Self::Context>,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        let EstateAction::CompleteMandatoryReview {
            estate,
            access,
            review,
        } = *self
        else {
            return Err(ApplicationCapabilityRequestProjectionDenial::input_variant(
                "CompleteEstateMandatoryReviewOperation",
            ));
        };
        let context = elevation_context(access).entity(
            EstateMandatoryReviewSlot::reference(),
            ApplicationCapabilityEntitySelector::new(
                MandatoryReviewIdentityField::reference(),
                crate::schema::encoded_bank_value::<crate::schema::MandatoryReviewIdBinding>(
                    review,
                ),
            ),
        );
        Ok(estate_transition_request(self, estate, context))
    }
}

impl ApplicationCapabilityElevationRequest<BankSchema, RequestEstateEmergencyAccessOperation>
    for EstateAction
{
    type Scope = EstateCase;
    type Context = EstateActionContext;

    fn elevation_request(
        &self,
    ) -> Result<
        ApplicationCapabilityElevationRequestProjection<BankSchema, Self::Scope, Self::Context>,
        ApplicationCapabilityElevationRequestProjectionDenial,
    > {
        let EstateAction::RequestEmergencyAccess {
            estate,
            access,
            review,
            grant,
            reason,
            field,
            duration,
        } = *self
        else {
            return Err(
                ApplicationCapabilityElevationRequestProjectionDenial::input_variant(
                    "RequestEstateEmergencyAccessOperation",
                ),
            );
        };
        ApplicationCapabilityElevationRequestProjection::new(
            elevation_target_request(estate, field),
            ApplicationCapabilityEntitySelector::new(
                CapabilityGrantIdentityField::reference(),
                crate::schema::encoded_bank_value::<crate::schema::CapabilityGrantIdBinding>(grant),
            ),
            format!("estate-emergency-access-{}", access.get()),
            ApplicationCapabilityValueBinding::new(
                EmergencyAccessIdentityField::reference(),
                crate::schema::encoded_bank_value::<crate::schema::EmergencyAccessIdBinding>(
                    access,
                ),
            ),
            format!("estate-mandatory-review-{}", review.get()),
            ApplicationCapabilityValueBinding::new(
                MandatoryReviewIdentityField::reference(),
                crate::schema::encoded_bank_value::<crate::schema::MandatoryReviewIdBinding>(
                    review,
                ),
            ),
            ApplicationCapabilityValueBinding::new(
                EmergencyAccessReasonField::reference(),
                crate::schema::encoded_bank_value::<crate::schema::EmergencyAccessReasonBinding>(
                    reason,
                ),
            ),
            duration,
        )
    }
}

fn elevation_context(
    access: crate::estate::EmergencyAccessId,
) -> ApplicationCapabilityRequestContext<BankSchema, EstateActionContext> {
    ApplicationCapabilityRequestContext::new(EstateActionContext::reference()).entity(
        EstateEmergencyAccessSlot::reference(),
        ApplicationCapabilityEntitySelector::new(
            EmergencyAccessIdentityField::reference(),
            crate::schema::encoded_bank_value::<crate::schema::EmergencyAccessIdBinding>(access),
        ),
    )
}

fn estate_transition_request(
    action: &EstateAction,
    estate: crate::estate::EstateCaseId,
    context: ApplicationCapabilityRequestContext<BankSchema, EstateActionContext>,
) -> ApplicationCapabilityRequestProjection<BankSchema, EstateCase, EstateActionContext> {
    ApplicationCapabilityRequestProjection::new(
        ApplicationCapabilityEntitySelector::new(
            EstateCaseIdentityField::reference(),
            crate::schema::encoded_bank_value::<crate::schema::EstateCaseIdBinding>(estate),
        ),
        crate::schema::encoded_bank_value::<crate::schema::EstateCapabilityOperationBinding>(
            action.operation(),
        ),
        crate::schema::encoded_bank_value::<crate::schema::EstateCapabilityPurposeBinding>(
            action.purpose(),
        ),
        context,
    )
}

fn elevation_target_request(
    estate: crate::estate::EstateCaseId,
    field: crate::estate::RestrictedBankField,
) -> ApplicationCapabilityRequestProjection<BankSchema, EstateCase, EstateActionContext> {
    ApplicationCapabilityRequestProjection::new(
        ApplicationCapabilityEntitySelector::new(
            EstateCaseIdentityField::reference(),
            crate::schema::encoded_bank_value::<crate::schema::EstateCaseIdBinding>(estate),
        ),
        crate::schema::encoded_bank_value::<crate::schema::EstateCapabilityOperationBinding>(
            EstateCapabilityOperation::ViewRestrictedEstate,
        ),
        crate::schema::encoded_bank_value::<crate::schema::EstateCapabilityPurposeBinding>(
            EstateCapabilityPurpose::EmergencyProtection,
        ),
        ApplicationCapabilityRequestContext::new(EstateActionContext::reference()),
    )
    .field(crate::schema::encoded_bank_value::<
        crate::schema::RestrictedBankFieldBinding,
    >(field))
}
