use worth_query_decl::facade::application_capability::{
    ApplicationCapabilityEntitySelector, ApplicationCapabilityRelatedEntitySelector,
    ApplicationCapabilityRequest, ApplicationCapabilityRequestProjection,
    ApplicationCapabilityRequestProjectionDenial,
};

use crate::{
    estate::EstateAction,
    schema::{
        AccountIdentity, BankSchema, CapabilityAccount, EstateActionContext, EstateCase,
        FreezeEstateAccountCapability,
    },
};

use super::estate_request;

impl ApplicationCapabilityRequest<BankSchema, FreezeEstateAccountCapability> for EstateAction {
    type Scope = EstateCase;
    type Context = EstateActionContext;

    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<BankSchema, Self::Scope, Self::Context>,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        let EstateAction::FreezeAccount { estate, account } = *self else {
            return Err(ApplicationCapabilityRequestProjectionDenial::input_variant(
                "FreezeEstateAccountOperation",
            ));
        };
        Ok(estate_request(self, estate).related_entity(
            ApplicationCapabilityRelatedEntitySelector::new(
                CapabilityAccount::reference(),
                ApplicationCapabilityEntitySelector::new(
                    AccountIdentity::reference(),
                    crate::schema::encoded_bank_value::<crate::schema::AccountIdBinding>(account),
                ),
            ),
        ))
    }
}
