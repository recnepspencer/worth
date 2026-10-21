//! End-to-end ordinary/native equivalence across two independent finite growth axes.
use super::*;
#[derive(Debug)]
pub(super) struct Sample {
    pub native: WorthQueryApplicationProgramWork,
    pub work: measurement::Stages,
    pub population: measurement::Retained,
    pub retained: measurement::Retained,
    pub values: Vec<u64>,
    pub query_work: usize,
}
pub(super) fn values(request: &Request<'_>, roots: usize) -> Vec<u64> {
    (0..roots)
        .map(|i| {
            let read = request
                .query(PlanarOutputRead {
                    body_key: population::target("focus-a", i),
                })
                .execute()
                .unwrap();
            assert_eq!(read.rows().len(), 1);
            assert_eq!(
                read.rows()[0].value,
                length(2),
                "each independently seeded target has original y=1"
            );
            worth_query_consumer_values::PositiveLength::get(&read.rows()[0].value)
        })
        .collect()
}
fn ordinary(fanout: usize, unrelated: usize) -> Sample {
    let runtime = installation::install(fanout, unrelated);
    let adapter = authentication::admit(runtime.installed_schema());
    let scope = authentication::request_scope();
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let request = runtime.request(&principal, &scope);
    let population = population::retain_unrelated(&runtime, &request, unrelated);
    let before = measurement::retained(&runtime);
    assert!(
        unrelated == 0 || before.rows > 0,
        "population is actual retained native rows"
    );
    reset_planar_source_adjustment_contacts(&input("focus-a"));
    let measuring = measurement::Measure::begin(&runtime);
    let settled = settle(retain(&runtime, &request, "focus-a", 221_001), &request);
    assert_eq!(settled.settlement().root_outputs().count(), fanout);
    assert_eq!(settled.settlement().output_count(), fanout * 2);
    let native = settled.settlement().work();
    let traversal = measuring.finish(&runtime);
    let held = measurement::retained(&runtime);
    let quiet_head = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let delivering = measurement::Measure::begin(&runtime);
    let delivered = settled.read_current(&request).unwrap();
    let delivery = delivering.finish(&runtime);
    let query_work = delivered.receipt().inspect().read_work().total_work_units();
    let historical = settled.read_historical(&request).unwrap();
    let first_receipt = delivered.receipt().inspect();
    let second_receipt = historical.receipt().inspect();
    let first_basis = first_receipt.basis();
    let second_basis = second_receipt.basis();
    assert_eq!(
        (
            first_basis.runtime_instance(),
            first_basis.branch(),
            first_basis.version(),
            first_basis.posture()
        ),
        (
            second_basis.runtime_instance(),
            second_basis.branch(),
            second_basis.version(),
            second_basis.posture()
        )
    );
    assert_eq!(delivered.rows(), historical.rows());
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &quiet_head,
        "quiet reads and refused close cannot change the World commit"
    );

    let observed = values(&request, fanout);
    assert_eq!(
        worth_query_consumer_values::PositiveLength::get(&delivered.rows()[0].value),
        observed[0]
    );
    assert_eq!(planar_source_adjustment_contacts(&input("focus-a")), (1, 1));
    let head_before_close = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    drop((delivered, historical));
    let closing = measurement::Measure::begin(&runtime);
    let closed = settled
        .close(&request)
        .unwrap_or_else(|(_, d)| panic!("ordinary lawful close: {d:?}"));
    let cleanup = closing.finish(&runtime);
    assert!(
        closed.managed_cleanup_receipt().is_none(),
        "unarmed fresh path has no failed physical cleanup"
    );
    assert_eq!(closed.retired_component_count(), 0);
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &head_before_close
    );
    let work = measurement::Stages {
        traversal,
        delivery,
        cleanup,
    };
    for owner in population {
        owner
            .close(&request)
            .unwrap_or_else(|(_, d)| panic!("population close: {d:?}"));
    }
    Sample {
        native,
        work,
        population: before,
        retained: held,
        values: observed,
        query_work,
    }
}
#[test]
fn discovered_ordinary_cost_matches_native_without_unrelated_registry_scan() {
    let mut previous = None;
    let mut sparse_execution = None;
    for fanout in [1, 3] {
        let mut sparse = None;
        for unrelated in [0, 4] {
            let ordinary = ordinary(fanout, unrelated);
            let native = native::run(fanout, unrelated);
            assert_eq!(
                ordinary.native, native.native,
                "same real native traversal bill"
            );
            assert_eq!(
                ordinary.values, native.values,
                "independent product readback agrees"
            );
            assert_eq!(
                ordinary.query_work, native.query_work,
                "same declared native selector bill"
            );
            assert_eq!(
                ordinary.work.whole_rows(),
                0,
                "ordinary full path has no whole-map traversal"
            );
            assert_eq!(
                native.work.whole_rows(),
                0,
                "native full path has no whole-map traversal"
            );
            // Meaningful work-growth headroom relative to the actual native journey,
            // not a new admitted quota or an exact incidental dispatch-unit count.
            assert!(ordinary.work.execution_work() > 0 && native.work.execution_work() > 0);
            assert!(
                ordinary.work.execution_work()
                    <= native.work.execution_work().checked_mul(2).unwrap(),
                "facade work stays within one additional native journey's measured work"
            );
            assert!(ordinary.work.span() <= ordinary.work.execution_work());
            assert!(native.work.span() <= native.work.execution_work());
            assert_eq!(
                ordinary.work.delivery.phases, 1,
                "ordinary guard+query are one phase"
            );
            assert_eq!(
                native.work.delivery.phases, 3,
                "native selection + guard + query use their actual public phases"
            );
            assert_eq!(
                ordinary.work.cleanup.phases, native.work.cleanup.phases,
                "both explicitly release in one native phase"
            );
            assert!(ordinary.retained.rows >= ordinary.population.rows);
            if let Some((rows, work, small_ordinary, small_native)) = sparse {
                assert!(ordinary.population.rows > rows);
                assert_eq!(
                    ordinary.native, work,
                    "unrelated population adds no producer/traversal work"
                );
                assert_growth(
                    small_ordinary,
                    ordinary.work.execution_work(),
                    small_native,
                    native.work.execution_work(),
                );
            } else {
                sparse = Some((
                    ordinary.population.rows,
                    ordinary.native,
                    ordinary.work.execution_work(),
                    native.work.execution_work(),
                ));
            }
            if unrelated == 0 {
                if let Some(smaller) = previous {
                    assert!(ordinary.native.gathered_demand_count() > smaller);
                }
                if let Some((small_ordinary, small_native)) = sparse_execution {
                    assert_growth(
                        small_ordinary,
                        ordinary.work.execution_work(),
                        small_native,
                        native.work.execution_work(),
                    );
                }
                previous = Some(ordinary.native.gathered_demand_count());
                sparse_execution =
                    Some((ordinary.work.execution_work(), native.work.execution_work()));
            }
            eprintln!(
                "separate Signal checkpoint charges ordinary={}/{}/{} native={}/{}/{}",
                ordinary.work.traversal.signal_checkpoint_work,
                ordinary.work.delivery.signal_checkpoint_work,
                ordinary.work.cleanup.signal_checkpoint_work,
                native.work.traversal.signal_checkpoint_work,
                native.work.delivery.signal_checkpoint_work,
                native.work.cleanup.signal_checkpoint_work
            );
            eprintln!("finite serial fresh-runtime fanout={fanout} unrelated={unrelated} ordinary={ordinary:?} native={native:?}");
        }
    }
}
fn assert_growth(small_ordinary: u64, large_ordinary: u64, small_native: u64, large_native: u64) {
    let ordinary_growth = large_ordinary.saturating_sub(small_ordinary);
    let native_growth = large_native.abs_diff(small_native);
    // One measured small-native journey pays fixed phase/dispatch granules;
    // variable facade growth may not exceed twice the actual native growth.
    assert!(
        ordinary_growth
            <= small_native
                .checked_add(native_growth.checked_mul(2).unwrap())
                .unwrap(),
        "ordinary growth exceeds fixed dispatch headroom plus native growth"
    );
}
