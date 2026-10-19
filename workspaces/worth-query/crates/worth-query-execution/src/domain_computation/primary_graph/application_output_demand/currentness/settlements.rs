use super::*;
impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema> {
    pub(in crate::domain_computation::primary_graph) fn require_current_settlements_in<'s>(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        relational: &worth_relational::facade::runtime::RelationalRuntime,
        settlements: impl IntoIterator<Item = &'s WorthQueryOutputDemandSettlement>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let runtime = self.application();
        for settlement in settlements {
            if !settlement.belongs_to(runtime) {
                return Err(denial(WorthQueryOutputDemandDenialKind::ForeignSettlement));
            }
            let lineage = runtime
                .primary_provider
                .graph
                .output_lineage
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let read = lineage_read(admission, |admission| {
                if let Some(receipt) = settlement.application_commit_receipt() {
                    lineage.source_facts_for_receipt(
                        runtime.runtime.authority_identity().as_u64(),
                        &runtime.installed_schema.binding_identity(),
                        self.product().observation(),
                        receipt,
                        &execution,
                        admission,
                    )
                } else if let Some(stable) = settlement.stable.as_ref() {
                    lineage.source_facts_for_stable_output(
                        runtime.runtime.authority_identity().as_u64(),
                        &runtime.installed_schema.binding_identity(),
                        self.product().observation(),
                        stable,
                        &execution,
                        admission,
                    )
                } else {
                    lineage.source_facts_for_restored_output(
                        runtime.runtime.authority_identity().as_u64(),
                        &runtime.installed_schema.binding_identity(),
                        self.product().observation(),
                        settlement,
                        &execution,
                        admission,
                    )
                }
            })?;
            drop(lineage);
            let Some(read) = read else {
                let receipt = settlement.application_commit_receipt();
                require_own_publication(receipt, self.product().observation())?;
                continue;
            };
            self.require_current_read(
                &execution,
                relational,
                &read,
                settlement.application_commit_receipt(),
                admission,
            )?;
        }
        Ok(())
    }
}
