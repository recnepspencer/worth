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
    pub(super) input_capture:
        Option<super::super::input_capture::InputCaptureOwner<Schema, Binding>>,
    marker: PhantomData<fn() -> Schema>,
}

impl<Schema, Binding> TypedInstalledProducer<Schema, Binding>
where
    Schema: ApplicationSchema,
    Binding: WorthQueryApplicationProducerBinding<Schema>,
{
    /// Ordinary Fresh has no occurrence input-preparation payload. Healthy
    /// output verification happens before Fresh and remains a genuine read.
    pub(super) fn refuse_uncaptured_production(&self) -> Result<(), super::ProducerExecutionStop> {
        if self.input_capture.is_some() {
            return Err(super::denial(
                super::WorthQueryOutputDemandDenialKind::ForeignDemand,
                "producer requires its complete occurrence input preparation",
            ).with_commit_denial_kind(
                crate::domain_computation::primary_graph::WorthQueryApplicationCommitDenialKind::RequiredOutputPreparationMissing,
            ).into());
        }
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn new(
        provider: Arc<Binding::Provider>,
        mutation: WorthQueryInstalledApplicationMutationBinding<Schema, Operation<Schema, Binding>>,
        prepared_graph: worth_query_admission::integration::WorthQueryPreparedApplicationOperationGraphWork,
        source_query: WorthQueryInstalledApplicationQueryBinding<
            Schema,
            SourceBinding<Schema, Binding>,
        >,
        principal_validation: WorthQueryValidatedPrincipalBinding,
        input_capture: Option<super::super::input_capture::InputCaptureOwner<Schema, Binding>>,
    ) -> Self {
        Self {
            provider,
            mutation,
            source_query,
            principal_validation,
            prepared_graph,
            input_capture,
            marker: PhantomData,
        }
    }
}
