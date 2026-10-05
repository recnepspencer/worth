use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use worth_query_host::facade::application_entry::{
    WorthQueryApplicationOutputDemandDenial, WorthQueryRequiredOutputPreparationDenial,
};

use super::*;

#[test]
fn producer_domain_denial_preserves_reason_and_releases_claim_for_retry() {
    let _guard = checkpoint_recovery_test_guard();
    let domain_denial = Arc::new(AtomicBool::new(true));
    let application = support::install_with_domain_denial(Arc::clone(&domain_denial));
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let before = request.retain_read().unwrap();

    // The provider asks the real handler to verify an output that cannot yet
    // exist. The handler, not a test substitute, owns CurrentOutputMissing.
    let mut output = request
        .start_program_outputs::<CheckpointProgram, CheckpointRoot>(
            &application,
            PlanarOutputDemand::new("anchor-a"),
            Default::default(),
        )
        .expect("the declared root starts");
    let denied = (0..512)
        .find_map(|_| match output.advance(&request) {
            Err(denied) => Some(denied),
            Ok(WorthQueryApplicationProgramOutputProgress::Pending) => None,
            Ok(WorthQueryApplicationProgramOutputProgress::Settled(_)) => {
                panic!("the missing current output cannot settle")
            }
        })
        .expect("the rejected producer returns a bounded denial");
    drop(output);
    let WorthQueryRequiredOutputPreparationDenial::Demand(
        WorthQueryApplicationOutputDemandDenial::Demand(cause),
    ) = denied
    else {
        panic!("the installed producer must own this refusal")
    };
    assert_eq!(
        cause.kind(),
        WorthQueryOutputDemandDenialKind::ProducerDomainDenied,
        "the real producer refusal must reach the domain boundary: {cause:?}"
    );
    assert_eq!(
        cause.domain_reason(),
        Some("A current planar output is required before this decision.")
    );
    assert_eq!(
        cause.recovery_posture(),
        worth_query_host::facade::primary_graph::WorthQueryOutputDemandRecoveryPosture::Terminal
    );
    assert!(domain_denial.load(Ordering::SeqCst));
    assert_eq!(
        before.selected_commit(),
        request.retain_read().unwrap().selected_commit()
    );

    // The failed demand cannot retain the producer claim or publish a partial
    // result. Repeating the same source with the ordinary provider now settles.
    domain_denial.store(false, Ordering::SeqCst);
    let settled = super::settle(&request, &application);
    assert_eq!(settled.root_producer_contacts_in_this_demand(), 1);
}
