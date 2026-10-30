pub(crate) use worth_query_bank_testkit_server::facade::ordinary_reads::canonical_work_scale::*;

#[test]
#[ignore = "scheduled high-operation speed probe; run explicitly with --ignored --nocapture"]
fn high_operation_speed_probe_keeps_publication_work_stable() {
    const QUERY_COUNT: usize = 512;
    const RESULT_COUNT: usize = 64;

    let fixture = over_budget_discovery_world("canonical-work-high-operation", RESULT_COUNT);
    let actor = fixture.authenticate();
    let started = Instant::now();
    let mut expected = None;
    let mut observed_rows = 0usize;

    for _ in 0..QUERY_COUNT {
        let result = fixture
            .world
            .runtime
            .query(queries::accounts())
            .as_principal(&actor)
            .controls(controls(RESULT_COUNT))
            .execute()
            .expect("high-operation account discovery should execute");
        observed_rows = observed_rows
            .checked_add(result.rows().len())
            .expect("observed row count remains bounded");
        let inspection = result.receipt().inspect();
        let phases = (
            inspection.publication_canonical_entries(),
            inspection.publication_sha256_compression_blocks(),
            inspection.publication_identity_text_materializations(),
        );
        if let Some(expected) = expected {
            assert_eq!(phases, expected);
        } else {
            expected = Some(phases);
        }
    }

    let elapsed = started.elapsed();
    let seconds = elapsed.as_secs_f64();
    let queries_per_second = QUERY_COUNT as f64 / seconds;
    let rows_per_second = observed_rows as f64 / seconds;
    assert_eq!(observed_rows, QUERY_COUNT * RESULT_COUNT);
    eprintln!(
        "profile={} queries={QUERY_COUNT} rows={observed_rows} elapsed={elapsed:?} \
         queries_per_second={queries_per_second:.2} rows_per_second={rows_per_second:.2}",
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        }
    );
}
