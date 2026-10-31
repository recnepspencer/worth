//! Stable and restored settlements carry only their genuine readonly posture.
use super::*;

impl WorthQueryOutputDemandSettlement {
    pub(in crate::domain_computation::primary_graph) fn from_restoration<Schema>(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        restored: &super::super::WorthQueryRestoredAcceptedOutput,
        output_family_identity: &str,
        producer_contacts_in_this_demand: usize,
    ) -> Result<Arc<Self>, WorthQueryOutputDemandDenial>
    where
        Schema: worth_query_installation::facade::ApplicationSchema,
    {
        let authority = runtime
            .product_runtime
            .recovered_root_authority
            .as_ref()
            .filter(|authority| authority.product_branch() == &restored.observation)
            .ok_or_else(|| {
                WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                    "restored output is not bound to this World's recovered root authority",
                )
            })?;
        let product = runtime
            .product_runtime
            .lease_from_observation(authority.product_branch().clone())
            .map_err(|error| {
                WorthQueryOutputDemandDenial::product_selection(
                    error,
                    "recovered output root is no longer current",
                )
            })?;
        Ok(Arc::new(Self {
            posture: restored.settlement_posture(),
            runtime_authority: runtime.runtime.authority_identity().as_u64(),
            schema_binding: runtime.installed_schema.binding_identity(),
            producer_identity: SettlementProducerIdentity::from_checkpoint(&restored.checkpoint),
            output_family_identity: output_family_identity.to_owned(),
            receipt: None,
            stable: None,
            output_correspondence: Arc::clone(&restored.correspondence),
            restored_source: Some(restored.into()),
            readiness_delivery: None,
            producer_contacts_in_this_demand,
            observation: WorthQueryApplicationReadObservation::from_settled_product(
                runtime,
                observation::OriginalSettledProducerObservation::restored(
                    execution,
                    runtime,
                    restored,
                    product.read_lease_ref(),
                )?,
                product.read_lease(),
            ),
        }))
    }

    pub(in crate::domain_computation::primary_graph) fn from_stable<Schema>(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        stable: &super::super::super::output_lineage::PublishedStableLineage,
        selected: &WorthQuerySelectedProductOperation<'_, Schema>,
        producer_identity: &str,
        output_family_identity: &str,
        producer_contacts_in_this_demand: usize,
    ) -> Result<Arc<Self>, WorthQueryOutputDemandDenial>
    where
        Schema: worth_query_installation::facade::ApplicationSchema,
    {
        Ok(Arc::new(Self {
            posture: WorthQueryOutputSettlementPosture::StableReused,
            runtime_authority: runtime.runtime.authority_identity().as_u64(),
            schema_binding: runtime.installed_schema.binding_identity(),
            producer_identity: producer_identity.to_owned().into(),
            output_family_identity: output_family_identity.to_owned(),
            receipt: None,
            stable: Some(stable.clone()),
            output_correspondence: Arc::clone(stable.output_correspondence()),
            restored_source: None,
            readiness_delivery: Some(
                WorthQueryOutputReadinessDeliveryEvidence::without_execution(),
            ),
            producer_contacts_in_this_demand,
            observation: WorthQueryApplicationReadObservation::from_settled_product(
                runtime,
                observation::OriginalSettledProducerObservation::stable(
                    execution,
                    runtime,
                    stable,
                    selected.product().read_lease_ref(),
                )?,
                selected.product().read_lease(),
            ),
        }))
    }
}
