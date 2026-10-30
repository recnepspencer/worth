//! Installed typed callbacks; root erasure stays private to their actual adapter.
use super::invocation::{decoded_provider, provider_result};
use super::*;
use crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity as Session;
use std::{any::Any, sync::Arc};
use worth_execution::{ExecutionArray, ExecutionRequest};
use worth_query_declaration::facade::{
    application_query::ApplicationManagedIndexBinding, application_schema::ApplicationSchema,
};
use worth_query_installation::facade::WorthQueryInstalledManagedIndex;

mod execution;
pub(super) use execution::ErasedIndexExecutor;

pub(in crate::domain_computation::primary_graph) trait InstalledIndexProvider<S, I>:
    Send + Sync
where
    S: ApplicationSchema,
    I: ApplicationManagedIndexBinding<S>,
{
    fn installed(&self) -> &WorthQueryInstalledManagedIndex<S, I>;
    fn build(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        build_session: Session,
        image: source_rows::SourceImageView<'_>,
    ) -> Result<Arc<dyn Any + Send + Sync>, WorthQueryManagedIndexWorkDenial>;
    fn candidates(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        image: source_rows::SourceImageView<'_>,
        nodes: &(dyn Any + Send + Sync),
        probe: &worth_foundational::facade::AspectValue,
    ) -> Result<ExecutionArray<WorthQueryManagedIndexSourceSlot>, WorthQueryManagedIndexWorkDenial>;
}

pub(super) struct TypedIndexProvider<S: ApplicationSchema, I: ApplicationManagedIndexBinding<S>, P>
{
    pub(super) installed: WorthQueryInstalledManagedIndex<S, I>,
    pub(super) provider: P,
}

impl<S, I, P> InstalledIndexProvider<S, I> for TypedIndexProvider<S, I, P>
where
    S: ApplicationSchema + 'static,
    I: ApplicationManagedIndexBinding<S>,
    P: WorthQueryManagedIndexProvider<S, I>,
{
    fn installed(&self) -> &WorthQueryInstalledManagedIndex<S, I> {
        &self.installed
    }
    fn build(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        build_session: Session,
        image: source_rows::SourceImageView<'_>,
    ) -> Result<Arc<dyn Any + Send + Sync>, WorthQueryManagedIndexWorkDenial> {
        let nodes = build::build::<S, I, P>(execution, &self.provider, build_session, image)?;
        node_backing::retain::<S, I, P>(execution, nodes).map_err(Into::into)
    }
    fn candidates(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        image: source_rows::SourceImageView<'_>,
        nodes: &(dyn Any + Send + Sync),
        probe: &worth_foundational::facade::AspectValue,
    ) -> Result<ExecutionArray<WorthQueryManagedIndexSourceSlot>, WorthQueryManagedIndexWorkDenial>
    {
        decoded_provider(execution, image.bindings, |control, session| {
            let probe = session
                .decode_managed_probe::<I::ProbeBinding>(probe)
                .ok_or_else(|| {
                    control.refuse(WorthQueryManagedIndexStop::ValueDecode(
                        session
                            .denial()
                            .expect("failed managed probe latches its refusal"),
                    ))
                })?;
            let nodes = nodes
                .downcast_ref::<node_backing::ProviderNodeBacking<S, I, P>>()
                .ok_or_else(|| {
                    WorthQueryManagedIndexWorkDenial::Controlled(
                        WorthQueryManagedIndexStop::ForeignNode,
                    )
                })?;
            let source = WorthQueryManagedIndexSourceRows::<S, I>::new(image, session, control);
            let mut candidates =
                WorthQueryManagedIndexCandidateWriter::allocate(execution, &source, control)?;
            let result =
                self.provider
                    .query(&source, &nodes.nodes, &probe, &mut candidates, control);
            source.check()?;
            provider_result(execution, control, result)?;
            candidates.finish(execution).map_err(Into::into)
        })
    }
}
