//! An owner call probes a declared managed Parent through the handler's own
//! issued decision read, never a cold sweep or a second index path.
use super::*;
use crate::domain_computation::primary_graph::WorthQueryApplicationProjection;
use worth_query_declaration::facade::{
    application_query::{
        ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeResolution,
    },
    application_schema::ApplicationStructuredValueBinding,
};

type Value<S, B> =
    <<B as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;

impl<Schema, Operation> WorthQueryComputationReader<'_, '_, '_, Schema, Operation>
where
    Schema: ApplicationSchema,
{
    /// Reads the declared managed Parent `B` with a fresh Source READ.
    ///
    /// The same contract as `DecisionReader::index_query` applies: the
    /// operation declares the Parent with `operation_managed_index_read` and
    /// its projected fields as reads, the intent resolves to the operation's
    /// own admitted scope, and only the committed admitted basis is read. An
    /// authored stage reading an applied candidate is refused as
    /// `ContextUnavailable`.
    ///
    /// The whole Source condition the read observed is recorded with the
    /// owner call that made it. A later run never carries that call by
    /// comparing it: a retained managed Source fact is not observable without
    /// a fresh read, so the call is made again.
    pub fn index_query<B>(
        &mut self,
        intent: B::Input,
    ) -> Result<Vec<Value<Schema, B>>, WorthQueryComputationReadDenial>
    where
        B: ApplicationQueryBinding<Schema>,
        B::Input: ApplicationQueryIntent<Schema, Binding = B>,
        B::ScopeBinding: ApplicationQueryScopeResolution<Schema, B::PrincipalIdentity>,
        Value<Schema, B>: WorthQueryApplicationProjection<Schema, B::Query>,
    {
        crate::domain_computation::primary_graph::invariant_projection::operation_reader::managed_index::computation_index_query::<Schema, Operation, B>(
            self.reader,
            self.source_query,
            intent,
        )
        .map_err(WorthQueryComputationReadDenial::ManagedIndex)
    }
}
