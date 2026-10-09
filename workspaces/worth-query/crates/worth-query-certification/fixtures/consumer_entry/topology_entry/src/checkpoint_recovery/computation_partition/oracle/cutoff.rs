//! A real input-reuse producer declines cutoff after managed execution.

use super::super::entry_edit::EntryFact;
use super::*;
use worth_query_host::facade::application_contribution::WorthQueryPartitionedComputationRun as Run;
use worth_query_host::facade::application_entry::WorthQueryOutputSettlementPosture;

fn settle(
    request: &Request<'_, '_, '_>,
    application: &Application<true>,
) -> (WorthQueryOutputSettlementPosture, Vec<OracleRun>) {
    room().clear();
    let mut demand = request
        .demand(RegionOutputDemand("anchor-a".to_owned()))
        .controls(input_cutoff::controls())
        .start_dependent_in_program::<OracleProgram<true>, RegionConnection>(application)
        .unwrap();
    let WorthQueryApplicationOutputDemandProgress::Settled(settled) =
        demand.advance(request).unwrap()
    else {
        panic!("one advance settles managed input cutoff")
    };
    (settled.posture(), take_runs(None))
}

#[test]
fn managed_partitioned_execution_prevents_cutoff_and_keeps_incremental_reuse() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install_with_reuse::<true>(|graph| {
        for set in ["even", "odd"] {
            facts::seed_set(graph, set, -0.0);
        }
        seed_entry(
            graph,
            &["even", "odd"],
            0,
            RegionEntry {
                id: 0,
                region: 1,
                value: 1.0,
                work: 1,
                fault: None,
            },
        );
    });
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    assert_eq!(settle(&request, &application).1.len(), 1);
    let source = request
        .query(PlanarRead {
            body_key: "anchor-b".to_owned(),
        })
        .execute()
        .unwrap();
    request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-b".to_owned(),
            replacement_y: length(2),
        })
        .expect_source(source.observed_sources()[0].clone())
        .idempotency(&9001_u64)
        .execute_performed::<OracleProgram<true>, OracleRoot>(&application)
        .unwrap();
    let (posture, runs) = settle(&request, &application);
    assert_eq!(posture, WorthQueryOutputSettlementPosture::Performed);
    assert_eq!(
        runs.len(),
        1,
        "managed execution has untracked context and cannot publish an alias"
    );
    assert_eq!(runs[0].runs, [Run::Incremental]);
    assert_eq!(
        runs[0].calls,
        OwnerCalls {
            plans: 0,
            keys: 0,
            gathers: 0,
            kernels: 0
        },
        "byte-equal gathered facts reuse every managed partition"
    );
    request
        .mutate(EntryEdit::new("odd", 0, EntryFact::Value, 2.5_f64.to_bits()).commanded(9002))
        .without_source()
        .idempotency(&9002_u64)
        .execute_in_program::<OracleProgram<true>>(&application)
        .unwrap();
    let (_, runs) = settle(&request, &application);
    assert_eq!(runs.len(), 1);
    assert_eq!(
        runs[0].calls,
        OwnerCalls {
            plans: 0,
            keys: 0,
            gathers: 1,
            kernels: 1
        }
    );
    assert_eq!(
        runs[0].runs,
        [Run::Incremental],
        "edit, source no-op, edit never loses the prior"
    );
}
