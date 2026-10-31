//! Explicit Native retained-owner inventory, never a sum of current-head samples.
use super::operator_identity::request_scope;
use super::{
    model::{host::Host, live_at},
    values,
};
use worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome;
use worth_query_replay::facade::WorthQueryCertificationCostRuntimeExt;
use worth_relational::facade::inspection::RelationalRetainedStorageObservation;

pub fn observe(host: &Host) -> RelationalRetainedStorageObservation {
    let scope = request_scope();
    let start = std::time::Instant::now();
    let inventory = host
        .runtime()
        .with_application_advancement(&scope, |phase| {
            host.runtime()
                .observe_certification_retained_storage(&phase)
                .expect("the actual settled cell owners are fully resident and linked")
        })
        .unwrap();
    println!("cell diagnostic_inventory_us={} scope=live-and-retired-root-allocation-union-plus-resident-catalog-owner-footprint;catalog-nested-optional-authorities-use-canonical-encoding-size;excludes-optional-index-estimates-allocator-private-headers-diagnostics-observation-backing-unpublished-candidates-and-transaction-working-sets;not-total-heap {inventory:?}", start.elapsed().as_micros());
    assert_eq!(
        inventory.authoritative_owner_footprint_bytes,
        inventory.current_root_authoritative_bytes
            + inventory.held_history_only_authoritative_bytes
            + inventory.reclaimable_only_authoritative_bytes
            + inventory.catalog_owner_footprint_bytes
    );
    inventory
}

#[test]
fn retained_cell_storage_includes_real_history_beyond_the_current_root() {
    let host = super::model::host::install();
    let initial = live_at::observe(&host);
    for (value, key) in [(31, 601), (43, 602)] {
        assert!(matches!(
            live_at::edit(&host, 1, 10, value, key),
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ));
    }
    let inventory = observe(&host);
    assert!(inventory.current_roots > 0);
    assert!(inventory.held_roots > 0);
    assert!(inventory.held_history_only_authoritative_bytes > 0);
    assert!(inventory.catalog_owner_footprint_bytes > 0);
    assert!(
        inventory.authoritative_owner_footprint_bytes > inventory.current_root_authoritative_bytes
    );
    assert_eq!(
        values(&live_at::at(&host, 1, &initial)),
        [(10, 7), (11, 13)]
    );
    assert_eq!(values(&live_at::current(&host, 1)), [(10, 43), (11, 13)]);
    assert_eq!(values(&live_at::current(&host, 2)), [(20, 29)]);
}
