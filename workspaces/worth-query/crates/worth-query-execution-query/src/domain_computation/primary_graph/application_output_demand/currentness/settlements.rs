use super::*;
use crate::domain_computation::primary_graph::OutputDemandSettlementRuntimeExt;
use worth_query_execution_attempt::facade::ports::PrimaryGraphProviderInternalPort as _;
use worth_query_execution_lineage::facade::ports::OutputDemandSettlementInternalPort as _;
use worth_query_execution_lineage::facade::ports::PrimaryGraphIntegrationHandleInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductBranchLeaseInternalPort as _;
impl<Schema: ApplicationSchema> WorthQuerySelectedProductOperation<'_, Schema> {
    pub fn require_current_settlements_in<'s>(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        relational: &worth_relational::facade::runtime::RelationalRuntime,
        settlements: impl IntoIterator<Item = &'s WorthQueryOutputDemandSettlement>,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let mut verification =
            crate::domain_computation::primary_graph::output_reuse::RetainedFactVerification::new(
                &execution,
                relational,
                crate::domain_computation::primary_graph::output_lineage::CertificationBasis::of(
                    self.product().lifecycle_incarnation(),
                    self.product().reference_generation(),
                    self.application_basis().snapshot_handle(),
                ),
                None,

            );
        self.require_current_settlements_in_extent(
            execution,
            relational,
            settlements,
            &mut verification,
        )
    }

    pub fn require_current_settlements_in_extent<'s>(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        relational: &worth_relational::facade::runtime::RelationalRuntime,
        settlements: impl IntoIterator<Item = &'s WorthQueryOutputDemandSettlement>,
        verification: &mut crate::domain_computation::primary_graph::output_reuse::RetainedFactVerification<'_, '_, '_>,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let runtime = self.application();
        for settlement in settlements {
            verification
                .check_live()
                .map_err(crate::domain_computation::primary_graph::output_reuse::read_denial)?;
            if !settlement.belongs_to(runtime) {
                return Err(denial(WorthQueryOutputDemandDenialKind::ForeignSettlement));
            }
            settlement.require_original_provenance(&execution)?;
            let lineage = runtime
                .runtime
                .primary_provider
                .graph_ref()
                .output_lineage_ref()
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let read = lineage_read(|| {
                if let Some(receipt) = settlement.publication_view() {
                    lineage.source_facts_for_publication(
                        runtime.runtime.authority_identity().as_u64(),
                        &runtime.installed_schema.binding_identity(),
                        self.product().observation(),
                        receipt,
                        &execution,
                    )
                } else if let Some(stable) = settlement.stable() {
                    lineage.source_facts_for_stable_output(
                        runtime.runtime.authority_identity().as_u64(),
                        &runtime.installed_schema.binding_identity(),
                        self.product().observation(),
                        stable,
                        &execution,
                    )
                } else {
                    lineage.source_facts_for_restored_output(
                        runtime.runtime.authority_identity().as_u64(),
                        &runtime.installed_schema.binding_identity(),
                        self.product().observation(),
                        settlement,
                        &execution,
                    )
                }
            })?;
            drop(lineage);
            let Some(read) = read else {
                let receipt = settlement.publication_view();
                require_own_publication(receipt, self.product().observation())?;
                continue;
            };
            self.require_current_read_in(
                &execution,
                relational,
                &read,
                settlement.publication_view(),
                verification,
            )?;
        }
        verification
            .check_live()
            .map_err(crate::domain_computation::primary_graph::output_reuse::read_denial)?;
        Ok(())
    }
}
