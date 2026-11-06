use crate::{
    model::BankPrincipalId,
    schema::{
        AccountIdentity, BankPrincipalBinding, BankPrincipalIdBinding, BankSchema,
        ExternalPrincipalMapping, Principal,
    },
};

use super::{
    AccountActivityLiveCause, AccountActivityQuery, AccountActivityQueryParametersBinding,
    AccountActivityQueryResultBinding, AccountActivityRequest,
};
use crate::schema::Posting;

worth_query_decl::facade::worth_query_structured_value_binding!(
    pub AccountActivityRequestBinding for AccountActivityRequest {
        identity: "AccountActivityRequest"
    }
);
worth_query_decl::facade::worth_query_query_binding!(
    pub AccountActivityQueryBinding for AccountActivityRequest, schema BankSchema,
    identity "worth.bank.account-activity-query-binding.v1",
    input AccountActivityRequestBinding,
    query AccountActivityQuery,
    parameters AccountActivityQueryParametersBinding => |_| worth_query_decl::facade::application_query::ApplicationQueryParameterSet::new(),
    result AccountActivityQueryResultBinding,
    principal BankPrincipalBinding, mapping ExternalPrincipalMapping, principal_entity Principal,
        principal_identity BankPrincipalId, identity_binding BankPrincipalIdBinding,
    scope AccountIdentity,
    field AccountIdentity::at("AccountIdentity"),
    value AccountActivityRequest::account,
    limits results 1_024
);

impl worth_query_decl::facade::application_query::ApplicationLiveQueryIntent<BankSchema>
    for AccountActivityRequest
{
    type Target = Posting;
    type LiveCause = AccountActivityLiveCause;
}
