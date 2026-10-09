//! The caller pays the read and keeps a committed checkpoint through exhaustion.
use super::super::differential::alphabet::Lcg;
use super::*;

#[test]
fn exhaustion_at_the_performed_commit_boundary_keeps_a_retryable_checkpoint() {
    let _guard = checkpoint_recovery_test_guard();
    let model = Model::new(&mut Lcg(SEEDS[0]));
    let app = installation::install_variant::<false, TOTALS_WORK, 1, 2>(
        None,
        Default::default(),
        |graph| model.seed(graph),
    );
    let (scope, principal) = authenticate(&app);
    let request = app.request(&principal, &scope);
    let mut output = request
        .demand(RegionOutputDemand(SCOPE.to_owned()))
        .start_dependent_in_program::<OracleProgram<false, TOTALS_WORK, 1, 2>, RegionConnection>(
            &app,
        )
        .unwrap();
    room().clear();
    use super::super::super::region_output::{arm_own_write, OwnWrite, OwnWriteRead};
    arm_own_write(Some(OwnWrite {
        number: 1,
        bits: 10_f64.to_bits(),
        read: OwnWriteRead::Observed,
    }));
    app.exhaust_request_at_performed_commit_for_test();
    let stopped = output.advance(&request);
    arm_own_write(None);
    assert!(
        matches!(&stopped,
        Err(worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandDenial::Demand(denial))
        if denial.kind() == primary_graph::WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
            && denial.recovery_posture() == primary_graph::WorthQueryOutputDemandRecoveryPosture::Retryable),
        "the next charge stops this request retryable: {:?}",
        stopped.as_ref().err()
    );
    assert_eq!(room().len(), 1, "the performed producer really committed");
    let committed = *room()[0].outcome.as_ref().unwrap();
    let WorthQueryApplicationOutputDemandProgress::Settled(settled) =
        output.advance(&request).unwrap()
    else {
        panic!("the checkpoint settles on the next request")
    };
    assert_eq!(
        settled.producer_contacts_in_this_demand(),
        2,
        "retry executes once for the input changed by the own-write"
    );
    assert_eq!(
        room().len(),
        2,
        "the retry consumes the checkpoint and then the changed input"
    );
    assert_eq!(*room()[0].outcome.as_ref().unwrap(), committed);
}

#[test]
fn a_retained_one_shot_read_debits_its_executed_work_from_the_callers_request() {
    let _guard = checkpoint_recovery_test_guard();
    let model = Model::new(&mut Lcg(SEEDS[0]));
    let app = install(|graph| model.seed(graph));
    let (scope, principal) = authenticate(&app);
    let request = app.request(&principal, &scope);
    let mut output = request
        .demand(RegionOutputDemand(SCOPE.to_owned()))
        .start_dependent_in_program::<OracleProgram, RegionConnection>(&app)
        .unwrap();
    app.retained_read_request_debits_for_test();
    assert!(matches!(
        output.advance(&request).unwrap(),
        WorthQueryApplicationOutputDemandProgress::Settled(_)
    ));
    let reads = app.retained_read_request_debits_for_test();
    assert_eq!(
        reads.len(),
        1,
        "the cold advance discloses its retained source once"
    );
    for (executed, debited) in reads {
        assert!(executed > 0, "the query executed real work");
        assert!(
            debited >= executed,
            "executed work and preparation debit the original request"
        );
    }
}

mod settled_rows;

mod first_admission;
mod population;
