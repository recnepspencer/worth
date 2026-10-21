use bank_domain::reads::EstateLegalAuthorityView;

use super::fixture::{request_scope, warm_locality_world, WarmLocalityAxis, ESTATE};
use crate::{queries, BankReadControls};

const EXPANDED_COUNT: usize = 8;

#[test]
fn every_named_population_axis_stays_out_of_warm_authorization_work() {
    let baseline = observe("warm-locality-baseline", WarmLocalityAxis::Fields, 0);
    for (scenario, axis) in [
        ("warm-locality-grants", WarmLocalityAxis::Grants),
        (
            "warm-locality-relationships",
            WarmLocalityAxis::Relationships,
        ),
        ("warm-locality-fields", WarmLocalityAxis::Fields),
        ("warm-locality-cases", WarmLocalityAxis::Cases),
    ] {
        let expanded = observe(scenario, axis, EXPANDED_COUNT);
        assert_eq!(baseline.ordinary_work_units, expanded.ordinary_work_units);
        assert_eq!(baseline.publication_work, expanded.publication_work);
        assert_eq!(baseline.rows, expanded.rows);
        assert!(expanded.resources_released);
    }

    let result_rows = observe(
        "warm-locality-result-rows",
        WarmLocalityAxis::ResultRows,
        EXPANDED_COUNT,
    );
    assert_eq!(result_rows.rows.len(), baseline.rows.len() + EXPANDED_COUNT);
    assert!(baseline.resources_released);
    assert!(result_rows.resources_released);
}

struct WarmLocalityObservation {
    ordinary_work_units: usize,
    publication_work: (u32, usize, u32),
    resources_released: bool,
    rows: Vec<EstateLegalAuthorityView>,
}

fn observe(scenario: &str, axis: WarmLocalityAxis, count: usize) -> WarmLocalityObservation {
    let fixture = warm_locality_world(scenario, axis, count);
    let principal = fixture.authenticate_reviewer();
    let result = fixture
        .runtime
        .query(queries::estate_legal_compliance(ESTATE))
        .as_principal(&principal)
        .controls(BankReadControls::current(request_scope(), 1).unwrap())
        .execute()
        .expect("the legal-compliance query should remain authorized");
    let receipt = result.receipt();
    let inspection = receipt.inspect();
    WarmLocalityObservation {
        ordinary_work_units: inspection.ordinary_work_units(),
        publication_work: (
            inspection.publication_canonical_entries(),
            inspection.publication_sha256_compression_blocks(),
            inspection.publication_identity_text_materializations(),
        ),
        resources_released: inspection.terminal_resources_released(),
        rows: result.rows()[0].authorities().to_vec(),
    }
}
