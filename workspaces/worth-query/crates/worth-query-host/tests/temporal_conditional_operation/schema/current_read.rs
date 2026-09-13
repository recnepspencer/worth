use worth_query_decl::facade::{
    application_query::ApplicationQueryParameterSet, worth_query_query_binding,
    worth_query_structured_value_binding,
};

use super::{
    ExternalMapping, IntentFacts, IntentIdentityField, IntentQueryParametersBinding,
    IntentQueryResultBinding, Principal, TemporalHostSchema, TemporalIntent, TemporalIntentQuery,
    TemporalPrincipalBinding,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemporalIntentReadRequest {
    identity: String,
}

impl TemporalIntentReadRequest {
    pub fn new(identity: impl Into<String>) -> Self {
        Self {
            identity: identity.into(),
        }
    }

    fn into_identity(self) -> String {
        self.identity
    }
}

worth_query_structured_value_binding!(pub TemporalIntentReadRequestBinding for TemporalIntentReadRequest {
    identity: "worth.query.test.host.temporal.intent_read_request.v1"
});

worth_query_query_binding!(
    pub TemporalIntentCurrentReadBinding for TemporalIntentReadRequest, schema TemporalHostSchema,
    identity "worth.query.test.host.temporal.intent_current_read.v1",
    input TemporalIntentReadRequestBinding,
    query TemporalIntentQuery,
    parameters IntentQueryParametersBinding => |_| ApplicationQueryParameterSet::new(),
    result IntentQueryResultBinding,
    principal TemporalPrincipalBinding, mapping ExternalMapping, principal_entity Principal,
        principal_identity u64, identity_binding worth_query_decl::facade::application_schema::U64ApplicationValueBinding,
    scope TemporalIntent, IntentFacts, IntentIdentityField, String,
        worth_query_decl::facade::application_schema::ReadOnly,
        worth_query_decl::facade::application_schema::NoApplicationUnit,
    field IntentIdentityField::reference(),
    value TemporalIntentReadRequest::into_identity,
    limits results 1, work 64
);
