use super::*;
use crate::application_query::*;

crate::worth_query_structured_value_binding!(pub(super) Parameters for () { identity: "test.correction.parameters.v1" });
crate::worth_query_application_query!(
    pub(super) HistoryQuery for MutationSchema, identity "test.correction.history.v1",
    parameters Parameters, result RenameResultBinding,
    scope Account => "Account", name "correction-history"
);
crate::worth_query_query_binding!(@binding
    pub(super) HistoryBinding for RenameInput, schema MutationSchema,
    identity "test.correction.history-binding.v1", input RenameInputBinding, query HistoryQuery,
    parameters Parameters, result RenameResultBinding,
    principal PrincipalBinding, mapping ExternalMapping, principal_entity Account,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope_binding ApplicationQueryFieldScope<AccountIdField>,
    scope_field_type AccountIdField,
    scope_field AccountIdField::at("AccountIdField"), limits results 1
);
