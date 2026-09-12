use worth_query_decl::facade::application_schema::{NoApplicationUnit, ReadOnly};

use crate::{
    model::{AccountId, BankPrincipalId},
    schema::{
        Account, AccountIdentity, BankPrincipalBinding, BankPrincipalIdBinding, BankSchema,
        ExternalPrincipalMapping, Identity, Principal,
    },
};

use super::{
    AccountActivityQuery, AccountActivityQueryParametersBinding, AccountActivityQueryResultBinding,
    AccountActivityRequest,
};

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
    scope Account, Identity, AccountIdentity, AccountId, ReadOnly, NoApplicationUnit,
    field AccountIdentity::reference(),
    value AccountActivityRequest::account,
    limits results 1_024, work 100_000
);
