use super::{
    Account, AccountSummaryParameters, AccountSummaryResult, ActivitySequenceResult,
    IdentityExecutionSchema,
};
use worth_query_declaration::worth_query_application_query;

worth_query_declaration::worth_query_structured_value_binding!(pub AccountSummaryQueryParametersBinding for AccountSummaryParameters { identity: "AccountSummaryParameters" });
worth_query_declaration::worth_query_structured_value_binding!(pub AccountSummaryQueryResultBinding for AccountSummaryResult { identity: "worth.query.test.execution.account_summary.result.v1" });
worth_query_application_query!(
    pub AccountSummaryQuery for IdentityExecutionSchema,
    identity "AccountSummaryQuery",
    parameters AccountSummaryQueryParametersBinding,
    result AccountSummaryQueryResultBinding,
    scope Account => "Account",
    name "account_summary"
);

worth_query_declaration::worth_query_structured_value_binding!(pub ScopedAccountSummaryQueryParametersBinding for AccountSummaryParameters { identity: "AccountSummaryParameters" });
worth_query_declaration::worth_query_structured_value_binding!(pub ScopedAccountSummaryQueryResultBinding for AccountSummaryResult { identity: "worth.query.test.execution.account_summary.result.v1" });
worth_query_application_query!(
    pub ScopedAccountSummaryQuery for IdentityExecutionSchema,
    identity "ScopedAccountSummaryQuery",
    parameters ScopedAccountSummaryQueryParametersBinding,
    result ScopedAccountSummaryQueryResultBinding,
    scope Account => "Account",
    name "scoped_account_summary"
);

worth_query_declaration::worth_query_structured_value_binding!(pub CrossRootQueryParametersBinding for AccountSummaryParameters { identity: "AccountSummaryParameters" });
worth_query_declaration::worth_query_structured_value_binding!(pub CrossRootQueryResultBinding for ActivitySequenceResult { identity: "worth.query.test.execution.activity_sequence.result.v1" });
worth_query_application_query!(
    pub CrossRootQuery for IdentityExecutionSchema,
    identity "CrossRootQuery",
    parameters CrossRootQueryParametersBinding,
    result CrossRootQueryResultBinding,
    scope Account => "Account",
    name "cross_root"
);

worth_query_declaration::worth_query_structured_value_binding!(pub GovernedAccountSummaryQueryParametersBinding for AccountSummaryParameters { identity: "AccountSummaryParameters" });
worth_query_declaration::worth_query_structured_value_binding!(pub GovernedAccountSummaryQueryResultBinding for AccountSummaryResult { identity: "worth.query.test.execution.account_summary.result.v1" });
worth_query_application_query!(
    pub GovernedAccountSummaryQuery for IdentityExecutionSchema,
    identity "GovernedAccountSummaryQuery",
    parameters GovernedAccountSummaryQueryParametersBinding,
    result GovernedAccountSummaryQueryResultBinding,
    scope Account => "Account",
    name "governed_account_summary"
);

worth_query_declaration::worth_query_structured_value_binding!(pub OrderedAccountSummaryQueryParametersBinding for AccountSummaryParameters { identity: "AccountSummaryParameters" });
worth_query_declaration::worth_query_structured_value_binding!(pub OrderedAccountSummaryQueryResultBinding for AccountSummaryResult { identity: "worth.query.test.execution.account_summary.result.v1" });
worth_query_application_query!(
    pub OrderedAccountSummaryQuery for IdentityExecutionSchema,
    identity "OrderedAccountSummaryQuery",
    parameters OrderedAccountSummaryQueryParametersBinding,
    result OrderedAccountSummaryQueryResultBinding,
    scope Account => "Account",
    name "ordered_account_summary"
);
