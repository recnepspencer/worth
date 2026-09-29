/// The prepared owners an intent declaration resolves its operability and
/// payload sources against.
pub(crate) struct UiIntentSourcePlans<'plan> {
    pub(crate) query: &'plan worth_ui_query_binding::WorthUiQueryBindingPlan,
    pub(crate) application_facts: &'plan crate::declaration::UiIntentApplicationFactPlan,
    pub(crate) expressions: &'plan crate::runtime::expression::UiExpressionCatalog,
}
