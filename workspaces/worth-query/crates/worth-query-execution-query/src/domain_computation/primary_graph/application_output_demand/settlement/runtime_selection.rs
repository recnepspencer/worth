//! Runtime settlement selection and current source-fact lookup.

use super::runtime::OutputDemandSettlementRuntimeExt;
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
    WorthQueryPrimaryGraphApplicationRuntime, WorthQuerySelectedProductOperation,
};
use worth_query_execution_attempt::facade::ports::PrimaryGraphProviderInternalPort as _;
use worth_query_execution_lineage::facade::ports::OutputDemandSettlementInternalPort as _;
use worth_query_execution_lineage::facade::ports::PrimaryGraphIntegrationHandleInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductBranchLeaseInternalPort as _;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: worth_query_installation::facade::ApplicationSchema,
{
    pub fn select_output_demand_settlement(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        settlement: &WorthQueryOutputDemandSettlement,
    ) -> Result<WorthQuerySelectedProductOperation<'_, Schema>, WorthQueryOutputDemandDenial> {
        if !settlement.belongs_to(self) {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ForeignSettlement,
                "output settlement belongs to another application runtime",
            ));
        }
        self.select_application_read_observation_with_request(execution, settlement.observation())
            .map_err(|error| match error {
                stopped @ crate::basis::WorthQueryProductBranchAdmissionDenial::Stopped(_) => {
                    WorthQueryOutputDemandDenial::product_selection(
                        stopped,
                        "retained output basis",
                    )
                }
                error @ crate::basis::WorthQueryProductBranchAdmissionDenial::Refused(_) => {
                    WorthQueryOutputDemandDenial::new(
                        WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                        format!("retained output basis unavailable: {error:?}"),
                    )
                }
            })
    }

    pub fn current_output_source_facts(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        settlement: &WorthQueryOutputDemandSettlement,
    ) -> Result<
        crate::domain_computation::primary_graph::output_lineage::ComparableSourceFacts,
        WorthQueryOutputDemandDenial,
    > {
        if !settlement.belongs_to(self) {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ForeignSettlement,
                "output settlement belongs to another application runtime",
            ));
        }
        if settlement.posture() == WorthQueryOutputSettlementPosture::RecoveredStableReused {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                "restored stable output requires native re-evidence",
            ));
        }
        let selected = match settlement.publication_view() {
            Some(receipt) => self
                .on_branch(receipt.product_branch())
                .select_with_request(execution)
                .map_err(|error| {
                    WorthQueryOutputDemandDenial::product_selection(
                        error,
                        "settled output currentness could not select its product occurrence",
                    )
                })?,
            None => self.select_output_demand_settlement(execution, settlement)?,
        };
        let lineage = self
            .runtime
            .primary_provider
            .graph_ref()
            .output_lineage_ref()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let facts = match settlement.publication_view() {
            Some(receipt) => lineage.source_facts_for_publication(
                self.runtime.authority_identity().as_u64(),
                &self.installed_schema.binding_identity(),
                selected.product().observation(),
                receipt,
                execution,
            ),
            None => match settlement.stable() {
                Some(stable) => lineage.source_facts_for_stable_output(
                    self.runtime.authority_identity().as_u64(),
                    &self.installed_schema.binding_identity(),
                    selected.product().observation(),
                    stable,
                    execution,
                ),
                None => lineage.source_facts_for_restored_output(
                    self.runtime.authority_identity().as_u64(),
                    &self.installed_schema.binding_identity(),
                    selected.product().observation(),
                    settlement,
                    execution,
                ),
            },
        };
        facts
            .map_err(|stop| {
                crate::domain_computation::primary_graph::output_reuse::retained_fact_denial(
                    crate::domain_computation::primary_graph::output_reuse::preflight_stop(stop),
                    "settled output lineage read unavailable",
                )
            })?
            .and_then(|read| read.facts)
            .filter(|facts| !facts.is_empty())
            .ok_or_else(|| {
                WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::Superseded,
                    "settled output no longer names the current output lineage",
                )
            })
    }
}
