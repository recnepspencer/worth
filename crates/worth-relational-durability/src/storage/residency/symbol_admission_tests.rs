//! Symbol closure identity follows a logical cell across physical promotion.
use super::*;

#[test]
fn head_symbol_identity_survives_full_promotion_without_covering_history() {
    worth_relational_branch::internal_port::branch::root::capture::with_reconstruction_fixture_for_test(|fixture_lane| {
        let host = worth_harness::facade::execution_fixture::serial_request();
        let execution = worth_execution::ExecutionRequest::serial(&host);
        let fixture = region_with_history();
        let head = head_demand(&fixture.cell, 1 << 24).unwrap();
        let before = head.symbol_admission_identity();
        let full = full_demand(&fixture.cell, 1 << 24).unwrap();
        assert_eq!(before, full.head_symbol_admission_identity());
        assert_ne!(before, full.symbol_admission_identity());
        assert_eq!(
            before,
            full.symbol_admission_identity().covered_head().unwrap()
        );
        assert_eq!(
            worth_relational_storage::internal_port::storage::residency::required_symbols::collect_for_request(
                fixture_lane,
                &head.resident_region().partition,
                false,
                &execution
            )
            .unwrap()
            .ids,
            worth_relational_storage::internal_port::storage::residency::required_symbols::collect_for_request(
                fixture_lane,
                full.full_view().partition(),
                false,
                &execution
            )
            .unwrap()
            .ids,
            "promotion preserves the entire current dictionary closure",
        );
    })
}

#[test]
fn metadata_only_freezes_forward_coverage_without_an_intermediate_cell_chain() {
    worth_relational_branch::internal_port::branch::root::capture::with_reconstruction_fixture_for_test(|fixture_lane| {
        let host = worth_harness::facade::execution_fixture::serial_request();
        let execution = worth_execution::ExecutionRequest::serial(&host);
        let fixture = region_with_history();
        let original = full_demand(&fixture.cell, 1 << 24).unwrap();
        let identity = original.symbol_admission_identity();
        let first = original.into_edit().freeze();
        let second = first.into_edit().freeze();
        assert_eq!(
            second.cell().full_symbol_predecessor_for_test(),
            Some(identity)
        );
        let cold = worth_relational_storage::internal_port::storage::residency::required_symbols::collect_for_request(
            fixture_lane,
            second.full_view().partition(),
            true,
            &execution,
        )
        .unwrap()
        .ids;
        let before = worth_relational_storage::internal_port::storage::residency::required_symbols::collect_for_request(
            fixture_lane,
            &full_demand(&fixture.cell, 1 << 24)
                .unwrap()
                .resident_region()
                .partition,
            true,
            &execution,
        )
        .unwrap()
        .ids;
        assert_eq!(cold, before);
    })
}
