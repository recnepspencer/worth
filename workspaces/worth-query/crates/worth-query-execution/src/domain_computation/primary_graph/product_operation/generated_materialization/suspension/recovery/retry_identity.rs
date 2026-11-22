use super::*;
use crate::equality_type_id::EqualityTypeId as TypeId;
use worth_query_execution_attempt::facade::ports::PrimaryGraphProviderInternalPort as _;
use worth_query_execution_lineage::facade::ports::PrimaryGraphIntegrationHandleInternalPort as _;
use worth_query_execution_query::facade::ports::ApplicationRuntimeCoreInternalPort as _;

pub(super) fn retry_matches<Schema, Producer>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    retry: &SuspensionRetry,
) -> bool
where
    Schema: ApplicationSchema,
    Producer: WorthQueryApplicationProducerBinding<Schema>,
{
    if retry.producer.runtime_authority != runtime.inner.runtime.authority_identity().as_u64()
        || retry.producer.schema != runtime.inner.installed_schema.binding_identity()
        || retry.producer.binding_type != TypeId::of::<Producer>()
        || retry.producer.binding_identity != Producer::IDENTITY
        || retry.producer.provider_identity != Producer::Provider::SEMANTIC_IDENTITY
        || runtime
            .inner
            .installed_producers
            .provider::<Producer>()
            .is_none()
    {
        return false;
    }
    runtime
        .inner
        .runtime
        .primary_provider_ref()
        .graph_ref()
        .output_lineage_ref()
        .lock()
        .expect("application output lineage lock is available")
        .qualified_output::<Producer::Operation>(
            retry.producer.runtime_authority,
            &retry.producer.schema,
            retry.producer.scope,
            retry.producer.output_occurrence,
            retry.producer.output_generation,
            retry.producer.runtime_source_identity,
            retry.producer.checkpoint_source_identity,
        )
        .is_some_and(|exact| {
            exact.source_identity == retry.producer.recorded_source_identity
                && exact.runtime_authority == retry.producer.runtime_authority
                && exact.schema == retry.producer.schema
                && exact.scope == retry.producer.scope
                && Arc::ptr_eq(&exact.correspondence, &retry.correspondence)
        })
}
