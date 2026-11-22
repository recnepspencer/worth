use crate::schema::{AccountIdentity, BankSchema};

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
    pub AccountActivityQueryBinding for AccountActivityRequest, schema BankSchema, context crate::schema::BankBindingContext,
    identity "worth.bank.account-activity-query-binding.v1",
    input AccountActivityRequestBinding,
    query AccountActivityQuery,
    parameters AccountActivityQueryParametersBinding => |_| worth_query_decl::facade::application_query::ApplicationQueryParameterSet::new(),
    result AccountActivityQueryResultBinding,
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
