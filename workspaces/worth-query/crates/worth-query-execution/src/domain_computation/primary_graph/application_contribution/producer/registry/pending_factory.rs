//! Cold producer execution preparation against the final provider support.

use std::{marker::PhantomData, sync::Arc};

use worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupportSnapshot;
use worth_query_admission::integration::prepare_application_operation_graph_work;
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationMutationBinding,
    WorthQueryInstalledApplicationQueryBinding, WorthQueryInstalledPackageIndex,
};

use super::{denial, DenialKind};
use crate::domain_computation::primary_graph::application_attempt::application_resource_request;
use crate::domain_computation::primary_graph::application_contribution::producer::{
    InstalledProducerExecutor, ProducerSourceBinding, ProducerSourceQuery, ProducerSourceValue,
    TypedInstalledProducer, WorthQueryApplicationProducerBinding,
    WorthQueryApplicationProducerProvider,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenial;

pub(super) trait PendingProducerExecutor<Schema>: Send + Sync {
    fn prepare(
        self: Box<Self>,
        support: &WorthQueryExecutionResourceSupportSnapshot,
        packages: &WorthQueryInstalledPackageIndex,
    ) -> Result<Arc<dyn InstalledProducerExecutor<Schema>>, WorthQueryPrimaryGraphInstallationDenial>;
}

pub(super) struct TypedPendingProducer<Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
{
    provider: Arc<Binding::Provider>,
    mutation: WorthQueryInstalledApplicationMutationBinding<Schema, Binding::Operation>,
    source_query:
        WorthQueryInstalledApplicationQueryBinding<Schema, ProducerSourceBinding<Schema, Binding>>,
    marker: PhantomData<fn() -> Schema>,
}

impl<Schema, Binding> TypedPendingProducer<Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
{
    pub(super) fn new(
        provider: Arc<Binding::Provider>,
        mutation: WorthQueryInstalledApplicationMutationBinding<Schema, Binding::Operation>,
        source_query: WorthQueryInstalledApplicationQueryBinding<
            Schema,
            ProducerSourceBinding<Schema, Binding>,
        >,
    ) -> Self {
        Self {
            provider,
            mutation,
            source_query,
            marker: PhantomData,
        }
    }
}

impl<Schema, Binding> PendingProducerExecutor<Schema> for TypedPendingProducer<Schema, Binding>
where
    Schema: ApplicationSchema + 'static,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
    Binding::Provider: WorthQueryApplicationProducerProvider<Schema, Binding>,
    ProducerSourceValue<Schema, Binding>:
        crate::domain_computation::primary_graph::WorthQueryApplicationProjection<
                Schema,
                ProducerSourceQuery<Schema, Binding>,
            > + 'static,
    ProducerSourceQuery<Schema, Binding>: 'static,
{
    fn prepare(
        self: Box<Self>,
        support: &WorthQueryExecutionResourceSupportSnapshot,
        packages: &WorthQueryInstalledPackageIndex,
    ) -> Result<Arc<dyn InstalledProducerExecutor<Schema>>, WorthQueryPrimaryGraphInstallationDenial>
    {
        let request = application_resource_request(self.mutation.operation().contracts())
            .ok_or_else(|| denial(DenialKind::ProducerGraphWorkRejected, Binding::IDENTITY))?;
        let prepared =
            prepare_application_operation_graph_work(self.mutation.operation(), &request, support)
                .map_err(|error| {
                    denial(
                        DenialKind::ProducerGraphWorkRejected,
                        format!("{}: {error:?}", Binding::IDENTITY),
                    )
                })?;
        let principal_validation = packages
            .retain_validated_principal_binding(self.source_query.principal_binding())
            .map_err(|error| {
                denial(
                    DenialKind::ForeignProducerBinding,
                    format!("{}: {error:?}", Binding::IDENTITY),
                )
            })?;
        Ok(Arc::new(TypedInstalledProducer::<Schema, Binding>::new(
            self.provider,
            self.mutation,
            prepared,
            self.source_query,
            principal_validation,
        )))
    }
}
