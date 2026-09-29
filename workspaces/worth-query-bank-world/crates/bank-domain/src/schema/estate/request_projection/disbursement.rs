use worth_query_decl::facade::application_capability::{
    ApplicationCapabilityEntitySelector, ApplicationCapabilityRelatedEntitySelector,
    ApplicationCapabilityRequest, ApplicationCapabilityRequestProjection,
    ApplicationCapabilityRequestProjectionDenial,
};

use crate::{
    estate::EstateAction,
    schema::{
        AccountIdentity, BankSchema, CapabilityAccount, DisburseEstateCapability,
        EstateActionContext, EstateCase,
    },
};

use super::estate_request;

impl ApplicationCapabilityRequest<BankSchema, DisburseEstateCapability> for EstateAction {
    type Scope = EstateCase;
    type Context = EstateActionContext;

    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<BankSchema, Self::Scope, Self::Context>,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        let EstateAction::DisburseEstate(disbursement) = *self else {
            return Err(ApplicationCapabilityRequestProjectionDenial::input_variant(
                "DisburseEstateOperation",
            ));
        };
        Ok(estate_request(self, disbursement.estate)
            .related_entity(ApplicationCapabilityRelatedEntitySelector::new(
                CapabilityAccount::reference(),
                ApplicationCapabilityEntitySelector::new(
                    AccountIdentity::reference(),
                    crate::schema::encoded_bank_value::<crate::schema::AccountIdBinding>(
                        disbursement.source_account,
                    ),
                ),
            ))
            .magnitude(crate::schema::encoded_bank_value::<
                crate::schema::UsdMoneyBinding,
            >(disbursement.amount)))
    }
}
