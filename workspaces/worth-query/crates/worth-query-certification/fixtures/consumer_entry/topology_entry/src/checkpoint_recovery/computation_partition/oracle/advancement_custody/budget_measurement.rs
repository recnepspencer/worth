//! A fresh successful settlement measures the allowance before the refused run.
use super::*;
use worth_query_host::facade::primary_graph::signal_request_charges_on_this_thread_for_test as signal_charges;

pub(super) fn application() -> installation::Application<false, PRODUCER_WORK, RUNS> {
    installation::install_configured::<false, PRODUCER_WORK, RUNS>(
        None,
        Default::default(),
        |graph| {
            facts::seed_set(graph, "even", -0.0);
            seed_entry(
                graph,
                &["even"],
                0,
                RegionEntry {
                    id: 0,
                    region: 0,
                    value: 1.0,
                    work: ENTRY_WORK,
                    fault: None,
                },
            );
        },
    )
}

pub(super) fn ceiling() -> u64 {
    bound(None);
    let application = application();
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let mut interest = request
        .demand(RegionOutputDemand(SCOPE.to_owned()))
        .start_dependent_in_program::<OracleProgram<false, PRODUCER_WORK, RUNS>, RegionConnection>(
            &application,
        )
        .unwrap();
    arm_own_write(Some(OwnWrite {
        number: 0,
        bits: 2.5_f64.to_bits(),
        read: OwnWriteRead::Blind,
    }));
    let policy = support::CHECKPOINT_EXECUTION_POLICY.budget();
    bound(Some(worth_foundational::ExecutionBudget::new(
        NonZeroUsize::MIN,
        policy.charged_memory_bytes(),
        u64::MAX,
    )));
    reports();
    passes();
    signal_charges();
    kernel_charges();
    room().clear();
    interest
        .settle(&request)
        .expect("the measuring run funds the complete real settlement");
    assert_eq!(
        room().len(),
        REQUIRED_PRODUCER_PASSES as usize,
        "measurement includes both producer passes"
    );
    assert!(room().iter().all(|run| run.outcome.is_ok()));
    assert_eq!(
        kernel_charges(),
        vec![ENTRY_WORK as u64 + EVEN_WEIGHT_WORK; REQUIRED_PRODUCER_PASSES as usize]
    );
    assert!(
        signal_charges().iter().sum::<u64>() > 0,
        "measurement includes delivery"
    );
    let requests = reports();
    assert_eq!(
        requests.len(),
        1,
        "measurement retains one request across the retry"
    );
    let complete = requests[0].as_ref().unwrap().charged_work();
    // Refuse one work unit of the completed settlement, after both handlers.
    complete
        .checked_sub(1)
        .expect("real settlement charges positive work")
}
