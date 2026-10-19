//! Only the source owner can pair retained values with their query identity.
use super::{RetainedExpressionContext, WorthQueryObservedSource};
use std::sync::Arc;
use worth_foundational::expression_api::ExpressionValue;

#[derive(Clone)]
pub(in crate::domain_computation) struct ObservedQueryExpressionContext {
    query_identifier: Arc<String>,
    context: Arc<RetainedExpressionContext>,
}
impl ObservedQueryExpressionContext {
    pub(in crate::domain_computation::primary_graph) fn query_identifier(&self) -> &str {
        &self.query_identifier
    }
    pub(in crate::domain_computation::primary_graph) fn value(&self) -> &ExpressionValue {
        self.context.value()
    }
    pub(in crate::domain_computation::primary_graph) fn snapshot(
        &self,
    ) -> &worth_query_declaration::facade::application_query::ApplicationQueryExpressionSnapshot
    {
        self.context.snapshot()
    }
}
impl<Query> WorthQueryObservedSource<Query> {
    pub(in crate::domain_computation::primary_graph::application_query) fn retain_expression_binding(
        &self,
    ) -> Option<ObservedQueryExpressionContext> {
        self.expression_context
            .as_ref()
            .map(|context| ObservedQueryExpressionContext {
                query_identifier: Arc::clone(&self.query_identifier),
                context: Arc::clone(context),
            })
    }
}
