//! Real currentness entry rejects corrupted retained provenance and receipts.
use super::*;
impl primary_graph::source_disclosure::OriginalSourceInspectionDisclosure {
    pub(in crate::domain_computation::primary_graph) fn court_public_settlement_rejects_foreign_receipt(
        self: Arc<Self>,
        foreign: Arc<Self>,
    ) {
        let runtime = installation::install();
        let original = journey::complete(&runtime);
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

    pub(in crate::domain_computation::primary_graph) fn court_public_settlement_rejects_disclosure(
        self: Arc<Self>,
    ) {
        let runtime = installation::install();
        let original = journey::complete(&runtime);
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
    let runtime = installation::install();
    let other_runtime = installation::install();
    let original = journey::complete(&runtime);
    let foreign = journey::complete(&other_runtime);
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
