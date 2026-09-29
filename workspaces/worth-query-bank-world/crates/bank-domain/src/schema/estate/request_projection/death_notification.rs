use worth_query_decl::facade::application_capability::{
    ApplicationCapabilityRequest, ApplicationCapabilityRequestProjection,
    ApplicationCapabilityRequestProjectionDenial,
};

use crate::{
    estate::EstateAction,
    schema::{BankSchema, EstateActionContext, EstateCase, NotifyDeathEstateCapability},
};

use super::estate_request;

impl ApplicationCapabilityRequest<BankSchema, NotifyDeathEstateCapability> for EstateAction {
    type Scope = EstateCase;
    type Context = EstateActionContext;

    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<BankSchema, Self::Scope, Self::Context>,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        let EstateAction::NotifyDeath { estate, .. } = *self else {
            return Err(ApplicationCapabilityRequestProjectionDenial::input_variant(
                "NotifyDeathEstateOperation",
            ));
        };
        Ok(estate_request(self, estate))
    }
}
