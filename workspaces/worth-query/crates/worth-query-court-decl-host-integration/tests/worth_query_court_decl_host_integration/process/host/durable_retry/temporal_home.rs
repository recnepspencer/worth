//! The temporal Home's public observations shared by its kill courts: the
//! operation's apply contacts, World's installed commits, the intent's
//! authored revision, and a retry parked at World's compare.
use super::*;
use crate::host::application_home::temporal::{observe, Home};
use temporal::TemporalConditional;
use worth_query_testkit_host::facade::host::temporal_conditional_operation::contribution_installation as temporal;

/// Execute the due intent until its Native effect is durable and World's
/// product comparison holds it; the process dies there. `applies` counts the
/// operation's invocations so far, this one included.
pub(super) fn kill_at_world_compare(home: &Home, applies_at_compare: usize) {
    let pause = home
        .runtime()
        .runtime()
        .world_operation_control_for_test()
        .pause_before_product_compare_for_test(std::num::NonZeroUsize::MIN);
    std::thread::scope(|threads| {
        threads.spawn(|| {
            let lease = checkpoint_capture::lease(128 << 20);
            drop(observe(
                home,
                &worth_execution::ExecutionRequest::leased(&lease),
            ));
        });
        assert!(
            pause.wait_until_reached(Duration::from_secs(20)),
            "the retry's Native effect reaches World's compare"
        );
        assert_eq!(applies(home), applies_at_compare);
        child_reexecution::wait_for_kill();
    });
}

pub(super) fn applies(home: &Home) -> usize {
    let installed = home.runtime().conditional::<TemporalConditional>().unwrap();
    installed.as_ref().as_ref().unwrap().contacts.snapshot().3
}

pub(super) fn installed_commits(home: &Home) -> usize {
    home.runtime()
        .runtime()
        .world_history_snapshot_for_test()
        .installed_commits()
}

pub(super) fn revision(home: &Home) -> u64 {
    let application = home.runtime();
    let scope = temporal::request_scope();
    let authentication = temporal::admit_identity_adapter(application.installed_schema());
    let principal = temporal::block_on(authentication.authenticate((), &scope)).unwrap();
    let rows = application
        .request(&principal, &scope)
        .query(worth_query_testkit_host::facade::host::temporal_conditional_operation::schema::current_read::TemporalIntentReadRequest { identity: "intent-1".to_owned() })
        .execute()
        .unwrap();
    rows.rows()[0].revision
}
