//! Output settlement funds delivery from the same request that ran its producer.
use super::*;
use worth_query_host::facade::{
    application_contribution::WorthQueryAdvancementDenial as Denial,
    application_entry::WorthQueryApplicationOutputDemandDenial as OutputDenial,
};

#[test]
fn producer_completes_but_delivery_cannot_reset_its_remaining_budget() {
    let _guard = checkpoint_recovery_test_guard();
    let _restore = Restore(place(Placement::Serial), bound(None));
    for placement in [Placement::Serial, Placement::Leased(NonZeroUsize::MIN)] {
        refused_delivery(placement);
    }
}

worth_execution::needs_os_threads! {
#[test]
fn a_wider_lease_funds_delivery_from_the_same_request() {
    let _guard = checkpoint_recovery_test_guard();
    let _restore = Restore(place(Placement::Serial), bound(None));
    refused_delivery(Placement::Leased(NonZeroUsize::new(4).unwrap()));
}
}

fn refused_delivery(placement: Placement) {
    place(placement);
    let ceiling = CALLER_WORK;
    bound(None);
    let application = application_fixture::application();
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let mut interest = request
        .demand(RegionOutputDemand(SCOPE.to_owned()))
        .start_dependent_in_program::<OracleProgram<false, PRODUCER_WORK, RUNS>, RegionConnection>(
            &application,
        )
        .unwrap();
    // The producer's real own-write invalidates the gathered readiness facts,
    // so settlement must evaluate Signal again after that producer completed.
    arm_own_write(Some(OwnWrite {
        number: 0,
        bits: 2.5_f64.to_bits(),
        read: OwnWriteRead::Blind,
    }));
    let policy = support::observation::CHECKPOINT_EXECUTION_POLICY.budget();
    bound(Some(worth_foundational::ExecutionBudget::new(
        NonZeroUsize::MIN,
        policy.charged_memory_bytes(),
        ceiling,
    )));
    reports();
    room().clear();
    let stopped = interest.settle(&request);
    let OutputDenial::Demand(denial) = stopped.err().expect("delivery must refuse") else {
        panic!("the output consumer retains the delivery cause");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryOutputDemandDenialKind::ExecutionRequest(Denial::NestedStopped,)
    );
    assert!(
        room().first().unwrap().outcome.is_ok(),
        "the first producer completed"
    );
    assert_eq!(
        room().len(),
        1,
        "the spent request cannot admit another producer"
    );
    let requests = reports();
    assert_eq!(requests.len(), 1);
    let charged = requests[0].as_ref().unwrap().charged_work();
    assert!(
        charged <= ceiling,
        "the refused reservation cannot overdraw its request"
    );
    // Err delivers no settlement. A request stop retains its owner's
    // terminal posture; this probe does not promise a fresh retry of it.
}
