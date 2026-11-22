//! The temporal Home's public observations shared by its kill courts: the
//! operation's apply contacts, World's installed commits, the intent's
//! authored revision, branch forking, and a retry parked at World's compare.
use super::*;
use crate::host::application_home::temporal::{observe_on_branch, Home};
use temporal::TemporalConditional;
use worth_query_host::facade::product::WorthQueryProductBranch;
use worth_query_testkit_host::facade::host::temporal_conditional_operation::contribution_installation as temporal;

/// Execute the due intent on root until its Native effect is durable and
/// World's product comparison holds it; the process dies there.
pub(super) fn kill_at_world_compare(home: &Home, applies_at_compare: usize) {
    kill_at_world_compare_on_branch(home, home.runtime().current_world(), applies_at_compare);
}

/// Execute the due intent on `branch` until its Native effect is durable and
/// World's product comparison holds it; the process dies there.
pub(super) fn kill_at_world_compare_on_branch(
    home: &Home,
    branch: WorthQueryProductBranch,
    applies_at_compare: usize,
) {
    let pause = home
        .runtime()
        .runtime()
        .world_operation_control_for_test()
        .pause_before_product_compare_for_test(std::num::NonZeroUsize::MIN);
    std::thread::scope(|threads| {
        threads.spawn(|| {
            let lease = checkpoint_capture::lease(128 << 20);
            drop(observe_on_branch(
                home,
                branch,
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

/// Fork a durable non-root product branch from the Home's current root branch.
pub(super) fn fork_branch(home: &Home) -> WorthQueryProductBranch {
    use worth_query_host::facade::product::{
        WorthQueryProductBranchCreateError as Create,
        WorthQueryProductBranchCreationContinuation as Continued,
    };
    let application = home.runtime().runtime();
    let scope = temporal::request_scope();
    let created = application
        .with_application_advancement(&scope, |phase| {
            application
                .branches()
                .fork(application.current_world())
                .components(|parts| parts.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
        })
        .unwrap();
    let mut original = match created {
        Ok(branch) => return branch,
        Err(Create::ProductUnpublished(original)) => original,
        Err(cause) => panic!("durable non-root branch creation: {cause:?}"),
    };
    let deadline = std::time::Instant::now() + Duration::from_secs(60);
    loop {
        assert!(std::time::Instant::now() < deadline, "fork settles");
        let scope = temporal::request_scope();
        let answer = application
            .with_application_advancement(&scope, |phase| original.continue_in_advancement(&phase))
            .unwrap();
        match answer {
            Ok(Continued::Performed(branch)) => return branch,
            Ok(other) => panic!("unexpected fork continuation: {other:?}"),
            Err(failure) => {
                original = failure.into_recovery();
                std::thread::yield_now();
            }
        }
    }
}

/// Find the unique non-root branch in the reopened Home's branch inventory.
pub(super) fn find_fork_branch(home: &Home) -> WorthQueryProductBranch {
    let application = home.runtime().runtime();
    let root = application.current_world();
    let scope = temporal::request_scope();
    let inventory = application
        .with_application_advancement(&scope, |phase| {
            application.branches().inventory(&phase).unwrap()
        })
        .unwrap();
    let forks: Vec<_> = inventory
        .iter()
        .filter(|branch| *branch != root)
        .collect();
    let [fork] = forks.as_slice() else {
        panic!("expected one non-root fork branch, found {}", forks.len());
    };
    *fork
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
    revision_on_branch(home, home.runtime().current_world())
}

pub(super) fn revision_on_branch(home: &Home, branch: WorthQueryProductBranch) -> u64 {
    let application = home.runtime();
    let scope = temporal::request_scope();
    let authentication = temporal::admit_identity_adapter(application.installed_schema());
    let principal = temporal::block_on(authentication.authenticate((), &scope)).unwrap();
    let rows = application
        .request(&principal, &scope)
        .on_branch(branch)
        .query(worth_query_testkit_host::facade::host::temporal_conditional_operation::schema::current_read::TemporalIntentReadRequest { identity: "intent-1".to_owned() })
        .execute()
        .unwrap();
    rows.rows()[0].revision
}

/// Format the currently published World head commit on `branch` without
/// touching conditional operation locks or `query.obligations`.
pub(super) fn head_commit_projection(home: &Home, branch: WorthQueryProductBranch) -> String {
    let application = home.runtime().runtime();
    let selected = application
        .with_host_advancement(|phase| application.on_branch(branch).select(&phase).unwrap())
        .unwrap();
    let commit = selected.product().selected_commit();
    let durable = worth_foundational::facade::DurableCompositeCommitIdentity::from_recorded(
        commit
            .owner_identity()
            .durable()
            .expect("durable Home publication"),
        commit.ordinal(),
    );
    format!("{durable:?}")
}

/// Format a published original receipt's durable World commit identity so
/// writer and reader processes compare the exact same occurrence across a kill.
pub(super) fn published_occurrence_projection(
    receipt: &worth_query_host::facade::primary_graph::WorthQueryRecoveredApplicationCommitReceipt,
) -> String {
    let commit = receipt
        .new_product_head()
        .durable()
        .expect("durable Home publication")
        .selected_commit();
    format!("{commit:?}")
}
