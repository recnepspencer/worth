#[allow(unused_imports)] use crate::physical_record_journeys::maintenance_interference::scale::*;
use {worth_store_testkit_store::physical_record_journeys::maintenance_interference::append_copies, worth_store_testkit_store::physical_record_journeys::maintenance_interference::checkpoint, worth_store_testkit_store::physical_record_journeys::maintenance_interference::initialize, worth_store_testkit_store::physical_record_journeys::maintenance_interference::placement};

#[test]
fn persisted_payload_reaches_eight_resident_budgets() {
    persist(8, false);
}

#[test]
fn persisted_payload_reaches_thirty_two_resident_budgets() {
    persist(32, false);
}

// The 128x world takes close to an hour in a debug build, so it runs in the
// scheduled lane: `cargo test --release ... -- --ignored scale::`.
#[test]
#[ignore = "scheduled release-scale Phase 6 lane"]
fn persisted_payload_reaches_128x_resident_budget() {
    persist(128, true);
}
