use super::*;

impl WorthQueryOutputDemandSettlement {
    pub(in crate::domain_computation::primary_graph) fn from_current_accepted<Schema>(
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        bound: BoundCurrentAcceptedOutput<'_>,
        producer_identity: &str,
        output_family_identity: &str,
        producer_contacts_in_this_demand: usize,
    ) -> Result<Arc<Self>, WorthQueryOutputDemandDenial>
    where
        Schema: worth_query_installation::facade::ApplicationSchema,
    {
        let completion = bound.completion();

        let (posture, receipt, stable, correspondence, restored_source) =
            match &completion.authority {
                WorthQueryAcceptedOutputAuthority::Committed(receipt) => (
                    WorthQueryOutputSettlementPosture::Performed,
                    Some(SettlementReceiptCustody::Ready(completion.clone())),
                    None,
                    receipt.retain_output_correspondence(),
                    None,
                ),
                WorthQueryAcceptedOutputAuthority::Stable(stable) => (
                    WorthQueryOutputSettlementPosture::StableReused,
                    None,
                    Some(stable.clone()),
                    Arc::clone(stable.output_correspondence()),
                    None,
                ),
                // The bound proof is a clean row: this restored output was
                // already compared in full on this runtime.
                WorthQueryAcceptedOutputAuthority::Restored(restored) => (
                    restored.settlement_posture(),
                    None,
                    None,
                    Arc::clone(&restored.correspondence),
                    Some(WorthQueryRestoredOutputSource::from(restored)),
                ),
            };
        let readiness_delivery = restored_source
            .is_none()
            .then(|| completion.readiness.clone());
        Ok(Arc::new(Self {
            posture,
            runtime_authority: runtime.runtime.authority_identity().as_u64(),
            schema_binding: runtime.installed_schema.binding_identity(),
            producer_identity: producer_identity.to_owned().into(),
            output_family_identity: output_family_identity.to_owned(),
            receipt,
            stable,
            output_correspondence: correspondence,
            restored_source,
            readiness_delivery,
            producer_contacts_in_this_demand,
            observation: WorthQueryApplicationReadObservation::from_product_funded(
                runtime,
                bound.product().retained_clone(),
            ),
        }))
    }
}
