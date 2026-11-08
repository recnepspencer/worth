//! Existing public opening refusal owns the exact retry and close predecessors.
use super::*;
use std::sync::{atomic::AtomicUsize, Arc, Mutex};
use worth_query_host::facade::{
    application_contribution::with_bootstrap_advancement_from_request,
    WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeOpenRefused as Refused,
};
use worth_query_testkit_host::facade::host::{application_home as fixture, checkpoint_capture};
use worth_store_application::{
    ApplicationHomeOpeningEdge as Edge, ApplicationHomeOpeningStage as Stage,
};

pub(super) fn retry(location: &Path, originals: &[ReleasedOriginalProjection], scale: u64) {
    let lease = Arc::new(checkpoint_capture::lease(128 * 1024 * 1024));
    let held = Arc::new(Mutex::new(None));
    let controlled = lease.clone();
    let captured = held.clone();
    let control = host_interruption::opening::control(
        move |arrival| {
            if arrival.stage == Stage::Custody && arrival.edge == Edge::Before {
                *captured.lock().unwrap() = Some(
                    controlled
                        .reserve_memory(controlled.available_memory_bytes().unwrap())
                        .unwrap(),
                );
            }
        },
        None,
        None,
    );
    let request = fixture::request_at_location(
        WorthQueryApplicationHomeLocation::Directory(location.to_owned()),
        opening::build(),
        Arc::new(AtomicUsize::new(0)),
        Arc::new(AtomicUsize::new(0)),
    )
    .with_opening_control_for_test(control);
    let refusal = with_bootstrap_advancement_from_request(
        worth_execution::ExecutionRequest::leased(&lease),
        |phase| request.open_in_advancement(&phase),
    )
    .unwrap()
    .err()
    .expect("held D8 cannot become service");
    let Refused::CustodyReadmission {
        mut query,
        home,
        cause,
    } = refusal
    else {
        panic!("only actual D8 refusal proves this prefix: {refusal:?}");
    };
    drop(held.lock().unwrap().take());
    let mut bytes = scale;
    let ceiling = scale
        .checked_mul((originals.len() as u64).checked_mul(8).unwrap())
        .unwrap();
    let mut retained_trial = None;
    // A bounded runtime sweep uses the owner's measured row-copy footprint,
    // not a newly pinned allocator peak or an edit/build/fit loop.
    while bytes <= ceiling {
        let trial = checkpoint_capture::lease(bytes);
        let before = trial.charged_memory_bytes();
        let refused = match query.readmit_custody(
            &worth_execution::ExecutionRequest::leased(&trial),
            home.query_obligations_medium(),
        ) {
            Ok(_) => panic!(
                "no nonempty proper prefix was observed; bytes={bytes}, source-scale={scale}"
            ),
            Err(refused) => refused,
        };
        let (same, denial) = refused.into_parts();
        query = same;
        let after = trial.charged_memory_bytes();
        if after > before {
            assert!(matches!(denial, worth_query_execution::facade::application_home_integration::QueryHomeCustodyReadmissionDenial::OriginalMemoryAdmission), "{denial:?}");
            println!("original-source prefix retained {} bytes under grant {bytes}, owner-copy-scale {scale}", after - before);
            retained_trial = Some(trial);
            break;
        }
        bytes = bytes.checked_mul(2).unwrap();
    }
    let retained_trial =
        retained_trial.expect("the bounded sweep must expose a genuine admitted prefix");
    let cleanup_lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let cleanup = worth_execution::ExecutionRequest::leased(&cleanup_lease);
    query = query
        .readmit_custody(&cleanup, home.query_obligations_medium())
        .unwrap_or_else(|refused| {
            panic!(
                "same-source prefix must retry: {:?}",
                refused.into_parts().1
            )
        })
        .consume(|query| query);
    let before = cleanup_lease.charged_memory_bytes();
    query = query
        .readmit_custody(&cleanup, home.query_obligations_medium())
        .unwrap_or_else(|refused| {
            panic!(
                "same-source duplicate must retry: {:?}",
                refused.into_parts().1
            )
        })
        .consume(|query| query);
    assert_eq!(
        cleanup_lease.charged_memory_bytes(),
        before,
        "duplicate admission retains no second binding"
    );
    let closed = Refused::CustodyReadmission { query, home, cause }
        .close(&cleanup)
        .unwrap_or_else(|refused| panic!("actual predecessor disposal: {refused:?}"));
    assert!(matches!(
        closed.directory_outcome(),
        Some(worth_store_application::ApplicationHomeCloseOutcome::Closed { .. })
    ));
    closed.reclaim();
    assert_eq!(retained_trial.charged_memory_bytes(), 0);
    let home = host_observation::opening::open(
        WorthQueryApplicationHomeLocation::Directory(location.to_owned()),
        opening::build(),
        Arc::new(AtomicUsize::new(0)),
        Arc::new(AtomicUsize::new(0)),
        None,
        Default::default(),
        &cleanup,
    );
    for (key, expected) in KEYS.into_iter().zip(originals) {
        assert_eq!(
            &super::super::released_original::terminal_projection(&home, key, key, &cleanup),
            expected
        );
    }
    home.close(&cleanup)
        .unwrap_or_else(|cause| panic!("source readmission close: {cause:?}"))
        .reclaim();
}
