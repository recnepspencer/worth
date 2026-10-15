//! Source preparation and original descriptor construction, without manual effect transitions.
use super::WorthQueryPendingApplicationLane;
use crate::application_entry::mutation::request::WorthQueryMutationSourceUnprepared;
use crate::application_entry::mutation::WorthQueryApplicationMutationRequest;
use crate::application_entry::WorthQueryMutationSourcePrepared;
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIntent, ApplicationMutationSourceExpectation,
    NoApplicationMutationSource,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationSchema, TypedMutationPreconditions,
};

/// Captures original meaning once. This builder cannot execute, recover or prepare a native mutation.
#[must_use = "retain the operation before advancing with fresh request authority"]
pub struct WorthQueryOperationBuilder<
    'application,
    'principal,
    'scope,
    Schema,
    Intent,
    SourcePreparation = WorthQueryMutationSourceUnprepared,
> where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
{
    pub(in crate::application_entry) native: WorthQueryApplicationMutationRequest<
        'application,
        'principal,
        'scope,
        Schema,
        Intent,
        SourcePreparation,
    >,
}

impl<'a, 'p, 's, S: ApplicationSchema, I: ApplicationMutationIntent<S>>
    WorthQueryOperationBuilder<'a, 'p, 's, S, I>
{
    pub fn expect_source(
        self,
        source: worth_query_execution::facade::primary_graph::WorthQueryObservedSource<<<I::Binding as ApplicationMutationBinding<S>>::SourceExpectation as ApplicationMutationSourceExpectation<S>>::Query>,
    ) -> WorthQueryOperationBuilder<'a, 'p, 's, S, I, WorthQueryMutationSourcePrepared> {
        WorthQueryOperationBuilder {
            native: self.native.expect_source(source),
        }
    }
    pub fn expect_result_set(
        self,
        source: worth_query_execution::facade::primary_graph::WorthQueryObservedResultSet<<<I::Binding as ApplicationMutationBinding<S>>::SourceExpectation as ApplicationMutationSourceExpectation<S>>::Query>,
    ) -> WorthQueryOperationBuilder<'a, 'p, 's, S, I, WorthQueryMutationSourcePrepared> {
        WorthQueryOperationBuilder {
            native: self.native.expect_result_set(source),
        }
    }
    pub fn without_source(
        self,
    ) -> WorthQueryOperationBuilder<'a, 'p, 's, S, I, WorthQueryMutationSourcePrepared>
    where
        I::Binding: ApplicationMutationBinding<S, SourceExpectation = NoApplicationMutationSource>,
    {
        WorthQueryOperationBuilder {
            native: self.native.without_source(),
        }
    }
}

impl<'a, 'p, 's, S: ApplicationSchema, I: ApplicationMutationIntent<S>, SourcePreparation>
    WorthQueryOperationBuilder<'a, 'p, 's, S, I, SourcePreparation>
{
    pub fn preconditions(
        self,
        preconditions: TypedMutationPreconditions<S, <I::Binding as ApplicationMutationBinding<S>>::Operation, <<I::Binding as ApplicationMutationBinding<S>>::ScopeBinding as worth_query_declaration::facade::application_operation::ApplicationMutationScopeBinding<S>>::Scope>,
    ) -> Self {
        Self {
            native: self.native.preconditions(preconditions),
        }
    }
    /// Only the bound workflow lane retains its issued native phase entrance.
    /// Fixed and source-only execution/recovery are unavailable on this keyed view.
    pub fn idempotency<'key>(
        self,
        key: &'key <I::Binding as ApplicationMutationBinding<S>>::IdempotencyKey,
    ) -> WorthQueryPendingApplicationLane<'a, 'p, 's, 'key, S, I, SourcePreparation> {
        WorthQueryPendingApplicationLane {
            native: self.native.idempotency(key),
        }
    }
}
