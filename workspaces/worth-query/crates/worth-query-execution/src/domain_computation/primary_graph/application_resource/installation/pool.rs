use super::super::{
    provider::{IngressBacking, WorthQueryApplicationResourceProvider},
    storage::{NativeResourceSlot, ResourceStorage},
};
use super::*;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenial as Denial;
use worth_execution::{ExecutionRequest, ExecutionRequestArrayBuilder};
use worth_query_installation::facade::WorthQueryInstalledApplicationResource;
use worth_runtime_bridge::facade::{
    BridgeConditionalRuntimeBuilder, BridgeOwnedAsyncRequestResponseDeclaration,
};

pub(in crate::domain_computation::primary_graph::application_resource) struct InstalledResourceBinding<
    Schema: ApplicationSchema,
    Binding: ApplicationResourceBinding<Schema>,
> {
    pub(in crate::domain_computation::primary_graph::application_resource) installed:
        WorthQueryInstalledApplicationResource<Schema, Binding>,
    pub(in crate::domain_computation::primary_graph::application_resource) provider:
        Arc<dyn WorthQueryApplicationResourceProvider<Schema, Binding, Role = Binding::Provider>>,
    pub(in crate::domain_computation::primary_graph::application_resource) clock:
        Arc<dyn ResourceClock>,
    pub(in crate::domain_computation::primary_graph::application_resource) storage:
        Arc<ResourceStorage>,
    pub(in crate::domain_computation::primary_graph::application_resource) prepared: BTreeMap<
        worth_query_installation::facade::WorthQueryResourceAccessPurpose,
        worth_query_admission::facade::graph_obligation::WorthQueryPreparedResourceReadGraphWork,
    >,
}

impl<Schema: ApplicationSchema, Binding: ApplicationResourceBinding<Schema>>
    InstalledResourceBinding<Schema, Binding>
{
    pub(super) fn install(
        execution: &ExecutionRequest<'_, '_>,
        builder: &mut BridgeConditionalRuntimeBuilder,
        support: &worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupportSnapshot,
        installed: WorthQueryInstalledApplicationResource<Schema, Binding>,
        provider: Arc<
            dyn WorthQueryApplicationResourceProvider<Schema, Binding, Role = Binding::Provider>,
        >,
        clock: Arc<dyn ResourceClock>,
    ) -> Result<Self, Denial> {
        let parts = installed.definition().parts();
        let prepared = worth_query_installation::facade::WorthQueryResourceAccessPurpose::ALL
            .into_iter()
            .map(|purpose| {
                worth_query_admission::facade::graph_obligation::prepare_resource_read_graph_work(
                    &installed, purpose, support,
                )
                .map(|prepared| (purpose, prepared))
                .map_err(|cause| denial(format!("{cause:?}")))
            })
            .collect::<Result<BTreeMap<_, _>, _>>()?;
        let profile = parts.profile;
        let delay = profile
            .retry_delay_milliseconds
            .checked_mul(1_000_000)
            .ok_or_else(|| denial("retry duration overflows source nanoseconds"))?;
        let timeout = profile
            .timeout_milliseconds
            .checked_mul(1_000_000)
            .ok_or_else(|| denial("timeout duration overflows source nanoseconds"))?;
        let limits = parts.limits;
        let live = usize::try_from(limits.maximum_live)
            .map_err(|_| denial("resource pool width overflow"))?;
        let retained = usize::try_from(limits.maximum_retained_results)
            .map_err(|_| denial("resource result width overflow"))?;
        let slots = usize::try_from(limits.ingress_slots_per_request)
            .map_err(|_| denial("resource ingress width overflow"))?;
        let frame_bytes = usize::try_from(limits.maximum_frame_bytes)
            .map_err(|_| denial("resource frame width overflow"))?;
        let identity = format!(
            "application-resource:{}:{}:{}",
            installed.binding_identity().package_identity().render_hex(),
            installed.binding_identity().schema_identity().render_hex(),
            Binding::IDENTITY
        );
        // This selects the Native payload family; actual typed installation,
        // exact source/attempt and provider verification remain admission truth.
        use sha2::{Digest, Sha256};
        let hash = Sha256::digest(identity.as_bytes());
        let payload_contract = u64::from_le_bytes(hash[..8].try_into().expect("SHA256 prefix"));
        let mut nodes = ExecutionRequestArrayBuilder::allocate(*execution, live)
            .map_err(|cause| denial(format!("{cause:?}")))?;
        for ordinal in 0..live {
            let ingress = IngressBacking::allocate(execution, slots, frame_bytes)
                .map_err(|cause| denial(format!("{cause:?}")))?;
            let declaration = builder
                .install_owned_async_request_response(
                    BridgeOwnedAsyncRequestResponseDeclaration::new(
                        format!("{identity}:{ordinal}"),
                        payload_contract,
                        limits.maximum_frame_bytes,
                        profile.retry_max_attempts,
                        delay,
                        timeout,
                    )
                    .with_managed_queue_capacity(
                        std::num::NonZeroU64::new(u64::from(limits.ingress_slots_per_request))
                            .expect("installed finite ingress"),
                    )
                    .with_process_local_retention(),
                )
                .map_err(|cause| denial(format!("{cause:?}")))?;
            nodes
                .push(NativeResourceSlot::new(
                    declaration,
                    ingress,
                    (ordinal + 1 < live).then_some(ordinal + 1),
                ))
                .map_err(|cause| denial(format!("{cause:?}")))?;
        }
        let nodes = nodes.seal().map_err(|cause| denial(format!("{cause:?}")))?;
        let storage = ResourceStorage::allocate(execution, nodes, retained)
            .map_err(|cause| denial(format!("{cause:?}")))?;
        Ok(Self {
            installed,
            provider,
            clock,
            storage,
            prepared,
        })
    }
}
