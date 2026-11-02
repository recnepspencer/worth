//! Ordinary declared index queries use the admitted decision reader's original request.
use super::*;
use crate::domain_computation::primary_graph::{
    HandlerExecutionDenial, WorthQueryApplicationProjection,
};
use worth_query_declaration::facade::{
    application_query::{
        ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeResolution,
    },
    application_schema::ApplicationStructuredValueBinding,
};
impl<S: ApplicationSchema, Binding: ApplicationMutationBinding<S>>
    DecisionReader<'_, '_, '_, S, Binding>
{
    /// Read the declared managed Parent and retain its complete Source condition.
    /// A fresh Source READ and the original operation's declared reads are required.
    pub fn index_query<B>(
        &mut self,
        intent: B::Input,
    ) -> Result<
        Vec<<B::ResultBinding as ApplicationStructuredValueBinding>::Value>,
        HandlerExecutionDenial,
    >
    where
        B: ApplicationQueryBinding<S>,
        B::Input: ApplicationQueryIntent<S, Binding = B>,
        B::ScopeBinding: ApplicationQueryScopeResolution<S, B::PrincipalIdentity>,
        <B::ResultBinding as ApplicationStructuredValueBinding>::Value:
            WorthQueryApplicationProjection<S, B::Query>,
    {
        self.checkpoint().map_err(HandlerExecutionDenial::new)?;
        self.context_use
            .set(self.context_use.get().source_query_permission());
        self.reader.index_query::<B>(intent)
    }
}
