use super::*;

impl WorthQueryOutputDemandSettlement {
    pub(in crate::domain_computation::primary_graph) fn from_current_accepted<Schema>(
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        bound: BoundCurrentAcceptedOutput<'_>,
        producer_identity: &str,
        output_family_identity: &str,
        producer_contacts_in_this_demand: usize,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Arc<Self>, WorthQueryOutputDemandDenial>
    where
        Schema: worth_query_installation::facade::ApplicationSchema,
    {
        let completion = bound.completion();
        let retained_bytes = arc_backing_bytes::<Self>()
            .and_then(|bytes| {
                bytes.checked_add(arc_backing_bytes::<WorthQueryApplicationReadObservation>()?)
            })
            .and_then(|bytes| bytes.checked_add(producer_identity.len()))
            .and_then(|bytes| bytes.checked_add(output_family_identity.len()))
            .ok_or_else(settlement_capacity_denial)?;
        let copy_work = producer_identity
            .len()
            .checked_add(output_family_identity.len())
            .and_then(|work| work.checked_add(10))
            .ok_or_else(settlement_work_denial)?;
        admission
            .charge_external_work(u64::try_from(copy_work).map_err(|_| settlement_work_denial())?)
            .map_err(|_| settlement_work_denial())?;
        let custody = runtime
            .output_demands
            .reserve_settlement_custody_capacity(retained_bytes, admission)?;
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
            producer_identity: producer_identity.to_owned(),
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
                custody,
            ),
        }))
    }
}

fn arc_backing_bytes<T>() -> Option<usize> {
    let header = 2usize.checked_mul(std::mem::size_of::<usize>())?;
    let alignment = std::mem::align_of::<T>();
    let offset = header.checked_add(alignment - 1)? & !(alignment - 1);
    offset.checked_add(std::mem::size_of::<T>())
}

fn settlement_capacity_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
        "current Ready settlement exceeds required custody",
    )
}

fn settlement_work_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
        "current Ready settlement exceeds request work",
    )
}
