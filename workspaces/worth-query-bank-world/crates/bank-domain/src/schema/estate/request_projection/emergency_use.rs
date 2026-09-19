use worth_query_decl::facade::application_capability::{
    ApplicationCapabilityEntitySelector, ApplicationCapabilityRequest,
    ApplicationCapabilityRequestContext, ApplicationCapabilityRequestProjection,
    ApplicationCapabilityRequestProjectionDenial,
};

use crate::{
    estate::EstateAction,
    schema::{
        BankSchema, EmergencyAccessIdentityField, EstateActionContext, EstateCase,
        EstateCaseIdentityField, ViewEstateEmergencyProtectionCapability,
    },
};

impl ApplicationCapabilityRequest<BankSchema, ViewEstateEmergencyProtectionCapability>
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
        let EstateAction::ViewRestrictedEstateWithEmergencyAccess {
            estate,
            access,
            field,
        } = *self
        else {
            return Err(ApplicationCapabilityRequestProjectionDenial::input_variant(
                "ViewRestrictedEstateWithEmergencyAccess",
            ));
        };
        Ok(ApplicationCapabilityRequestProjection::new(
            ApplicationCapabilityEntitySelector::new(
                EstateCaseIdentityField::reference(),
                crate::schema::encoded_bank_value::<crate::schema::EstateCaseIdBinding>(estate),
            ),
            crate::schema::encoded_bank_value::<crate::schema::EstateCapabilityOperationBinding>(
                self.operation(),
            ),
            crate::schema::encoded_bank_value::<crate::schema::EstateCapabilityPurposeBinding>(
                self.purpose(),
            ),
            ApplicationCapabilityRequestContext::new(EstateActionContext::reference()),
        )
        .elevation(ApplicationCapabilityEntitySelector::new(
            EmergencyAccessIdentityField::reference(),
            crate::schema::encoded_bank_value::<crate::schema::EmergencyAccessIdBinding>(access),
        ))
        .field(crate::schema::encoded_bank_value::<
            crate::schema::RestrictedBankFieldBinding,
        >(field)))
    }
}
