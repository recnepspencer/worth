//! Real currentness entry rejects corrupted retained provenance and receipts.

use worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::application_output_demand::settlement::observation::court::CourtWorthQueryOutputDemandSettlementForTest;
use crate::domain_computation::primary_graph::output_lineage::ownership_oracle::producer_observation::*;
/// Test doors on a library type, owned by this court.
pub trait ProvenanceOriginalSourceInspectionDisclosureForTest {
    fn court_public_settlement_rejects_foreign_receipt(self: Arc<Self>, foreign: Arc<Self>);
    fn court_public_settlement_rejects_disclosure(self: Arc<Self>);
}

impl ProvenanceOriginalSourceInspectionDisclosureForTest for worth_query_execution::court_doors_for_test::domain_computation::primary_graph::source_disclosure::inspection::OriginalSourceInspectionDisclosure {
    fn court_public_settlement_rejects_foreign_receipt(
        self: Arc<Self>,
        foreign: Arc<Self>,
    ) {
        let runtime = crate::domain_computation::primary_graph::output_lineage::ownership_oracle::installation::install();
        let original = crate::domain_computation::primary_graph::output_lineage::ownership_oracle::journey::complete(&runtime);
        runtime
            .with_host_advancement(|phase| {
                let selected = runtime
                    .on_branch(runtime.current_world())
                    .select(&phase)
                    .unwrap();
                original.assert_receipt_disclosure_provenance(self, foreign, |settled| {
                    selected.require_current_output_settlements(&phase, [settled])
                });
            })
            .unwrap();
    }

    fn court_public_settlement_rejects_disclosure(
        self: Arc<Self>,
    ) {
        let runtime = crate::domain_computation::primary_graph::output_lineage::ownership_oracle::installation::install();
        let original = crate::domain_computation::primary_graph::output_lineage::ownership_oracle::journey::complete(&runtime);
        runtime
            .with_host_advancement(|phase| {
                let selected = runtime
                    .on_branch(runtime.current_world())
                    .select(&phase)
                    .unwrap();
                original.assert_original_provenance(None, Some(self), |settled| {
                    selected.require_current_output_settlements(&phase, [settled])
                });
            })
            .unwrap();
    }
}

#[test]
fn committed_provenance_rejects_present_foreign_settlement_owner() {
    let runtime = crate::domain_computation::primary_graph::output_lineage::ownership_oracle::installation::install();
    let other_runtime = crate::domain_computation::primary_graph::output_lineage::ownership_oracle::installation::install();
    let original = crate::domain_computation::primary_graph::output_lineage::ownership_oracle::journey::complete(&runtime);
    let foreign = crate::domain_computation::primary_graph::output_lineage::ownership_oracle::journey::complete(&other_runtime);
    runtime
        .with_host_advancement(|phase| {
            let selected = runtime
                .on_branch(runtime.current_world())
                .select(&phase)
                .unwrap();
            original.assert_original_provenance(Some(&foreign), None, |settled| {
                selected.require_current_output_settlements(&phase, [settled])
            });
        })
        .unwrap();
}
