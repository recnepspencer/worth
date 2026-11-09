//! Public read intent for the fixture's existing default open account query.
use super::{status_parameter, AccountSummaryParameters, AccountSummaryQuery};
use crate::domain_computation::primary_graph::tests::fixture::{
    AccountStatus, IdentityExecutionSchema, TestAccountSourceBinding,
};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryBinding, ApplicationQueryFieldScope, ApplicationQueryIntent,
    ApplicationQueryParameterSet,
};
impl ApplicationQueryIntent<IdentityExecutionSchema> for AccountSummaryParameters {
    type Binding = TestAccountSourceBinding;
    fn parameters(&self) -> ApplicationQueryParameterSet<AccountSummaryQuery> {
        ApplicationQueryParameterSet::new()
            .bind(status_parameter::<AccountSummaryQuery>(), "open".to_owned())
            .unwrap()
    }
    fn into_scope(
        self,
    ) -> <Self::Binding as ApplicationQueryBinding<IdentityExecutionSchema>>::ScopeBinding {
        ApplicationQueryFieldScope::new(AccountStatus::reference(), "open".to_owned())
    }
}
