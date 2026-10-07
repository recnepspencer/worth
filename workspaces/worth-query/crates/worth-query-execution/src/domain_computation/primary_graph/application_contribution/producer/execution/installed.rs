//! Cold installed authorities retained by one typed producer executor.

use std::{marker::PhantomData, sync::Arc};

use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationMutationBinding,
    WorthQueryInstalledApplicationQueryBinding, WorthQueryValidatedPrincipalBinding,
};

use super::{Operation, SourceBinding};
use crate::domain_computation::primary_graph::application_contribution::producer::WorthQueryApplicationProducerBinding;

pub(in crate::domain_computation::primary_graph::application_contribution::producer) struct TypedInstalledProducer<
    Schema,
    Binding,
> where
    Schema: ApplicationSchema,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
{
    pub(super) provider: Arc<Binding::Provider>,
    /// Compiled once while installing this declared producer; final executor
    /// custody owns the cold proof until producer retirement.
    pub(super) mutation:
        WorthQueryInstalledApplicationMutationBinding<Schema, Operation<Schema, Binding>>,
    /// The same installed source Query authority checked during producer
    /// setup, moved into this executor instead of reissued on every Ready.
    pub(super) source_query:
        WorthQueryInstalledApplicationQueryBinding<Schema, SourceBinding<Schema, Binding>>,
    pub(super) principal_validation: WorthQueryValidatedPrincipalBinding,
    /// Static graph/resource decisions compiled against final provider support.
    pub(super) prepared_graph:
        worth_query_admission::integration::WorthQueryPreparedApplicationOperationGraphWork,
    marker: PhantomData<fn() -> Schema>,
}

impl<Schema, Binding> TypedInstalledProducer<Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
{
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn new(
        provider: Arc<Binding::Provider>,
        mutation: WorthQueryInstalledApplicationMutationBinding<Schema, Operation<Schema, Binding>>,
        prepared_graph: worth_query_admission::integration::WorthQueryPreparedApplicationOperationGraphWork,
        source_query: WorthQueryInstalledApplicationQueryBinding<
            Schema,
            SourceBinding<Schema, Binding>,
        >,
        principal_validation: WorthQueryValidatedPrincipalBinding,
    ) -> Self {
        Self {
            provider,
            mutation,
            source_query,
            principal_validation,
            prepared_graph,
            marker: PhantomData,
        }
    }
}
