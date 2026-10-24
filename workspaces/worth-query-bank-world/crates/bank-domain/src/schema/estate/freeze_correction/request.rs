use crate::{estate::EstateAction, schema::*};
use worth_query_decl::facade::application_capability::*;

impl ApplicationCapabilityRequest<BankSchema, RestoreFrozenEstateAccountCapability>
    for RestoreFrozenEstateAccount
{
    type Scope = EstateCase;
    type Context = EstateActionContext;
    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<BankSchema, EstateCase, EstateActionContext>,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        <EstateAction as ApplicationCapabilityRequest<BankSchema, FreezeEstateAccountCapability>>::capability_request(
            &EstateAction::FreezeAccount { estate: self.estate, account: self.account },
        )
    }
}
