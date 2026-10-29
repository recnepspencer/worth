//! Actual World protections and Native external leases balance at final owner drop.
pub use super::super::installation::World;
pub use super::{access::*, *};
pub use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::request_scope;
pub use worth_query_host::facade::certification::WorthQueryCertificationCostRuntimeExt;

pub fn probe(
    world: &World<Payload>,
    root: &worth_query_host::facade::application_entry::WorthQueryApplicationReadObservation,
) {
    let runtime = world.application.runtime();
    let baseline = runtime.world_history_snapshot_for_test().costs();
    let cost = runtime
        .with_host_advancement(|phase| {
            runtime
                .capture_certification_cost_scope(&phase, world.application.current_world())
                .unwrap()
        })
        .unwrap();
    {
        let captured = capture::<Entities>(world, 2);
        let mut cursor = open(world, &captured, root);
        let holding = runtime.world_history_snapshot_for_test().costs();
        assert!(
            holding.direct_protection_acquisitions() - holding.direct_protection_releases()
                > baseline.direct_protection_acquisitions() - baseline.direct_protection_releases()
        );
        loop {
            let observed_page = page(world, &captured, &mut cursor, root, &request_scope())
                .unwrap()
                .unwrap();
            if observed_page.is_complete() {
                break;
            }
        }
    }
    let after = runtime.world_history_snapshot_for_test().costs();
    assert_eq!(
        after.direct_protection_acquisitions() - baseline.direct_protection_acquisitions(),
        after.direct_protection_releases() - baseline.direct_protection_releases()
    );
    let observed = runtime
        .with_host_advancement(|phase| runtime.observe_certification_cost(&phase, &cost).unwrap())
        .unwrap();
    let retention = observed.relational().retention_cost_delta();
    assert!(retention.external_pin_acquires > 0);
    assert_eq!(
        retention.external_pin_acquires, retention.external_pin_releases,
        "actual capture/Native cursor external leases end at final drop"
    );
}
