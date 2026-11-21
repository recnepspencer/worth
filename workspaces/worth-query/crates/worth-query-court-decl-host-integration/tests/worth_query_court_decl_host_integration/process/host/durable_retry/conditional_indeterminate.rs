//! C3.3: a conditional operation whose commit answer was lost (R9
//! `OperationIndeterminate`) is killed. Reopen classifies its durable row by
//! World's state on that exact durable product branch before service, and the
//! retry of the original intent yields the exact original occurrence:
//! published, recovered unpublished (on root and non-root fork), or no effect.
use super::temporal_home::{
    applies, find_fork_branch, fork_branch, head_commit_projection, installed_commits,
    kill_at_world_compare, kill_at_world_compare_on_branch, published_occurrence_projection,
    revision, revision_on_branch,
};
use super::*;
use crate::host::application_home::temporal::{
    observe, observe_on_branch, open, retry_original_key_on_branch,
};
use worth_query_testkit_host::facade::host::temporal_conditional_operation::contribution_installation::primary_graph;

/// World's state for the lost answer at the kill.
#[derive(Clone, Copy)]
enum Cut {
    /// The commit published on root; only its answer was lost.
    Published,
    /// Nothing committed; a retry on root reached World's compare with its
    /// Native effect durable and the process died there.
    RecoveredUnpublished,
    /// Same as `RecoveredUnpublished`, executed on a non-root fork branch so
    /// cold classification must resolve against that exact branch incarnation.
    ForkRecoveredUnpublished,
    /// Nothing committed.
    NoEffect,
}

#[test]
fn indeterminate_after_world_published_finishes_with_the_original_after_kill() {
    journey(
        Cut::Published,
        "indeterminate_after_world_published_finishes_with_the_original_after_kill",
    );
}
#[test]
fn indeterminate_with_its_original_unpublished_continues_that_original_after_kill() {
    journey(
        Cut::RecoveredUnpublished,
        "indeterminate_with_its_original_unpublished_continues_that_original_after_kill",
    );
}
#[test]
fn indeterminate_on_non_root_branch_joins_that_branch_original_after_kill() {
    journey(
        Cut::ForkRecoveredUnpublished,
        "indeterminate_on_non_root_branch_joins_that_branch_original_after_kill",
    );
}
#[test]
fn indeterminate_without_a_committed_open_executes_fresh_once_after_kill() {
    journey(
        Cut::NoEffect,
        "indeterminate_without_a_committed_open_executes_fresh_once_after_kill",
    );
}

fn journey(cut: Cut, behavior: &str) {
    child_reexecution::drive(
        &format!("conditional_indeterminate::{behavior}"),
        || writer(cut),
        || reader(cut),
    );
}

fn encode_hex(bytes: &[u8; 32]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn decode_hex(text: &str) -> [u8; 32] {
    let mut out = [0u8; 32];
    for (i, slot) in out.iter_mut().enumerate() {
        *slot = u8::from_str_radix(&text[i * 2..i * 2 + 2], 16).unwrap();
    }
    out
}

fn writer(cut: Cut) {
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = open(&child_reexecution::location(), &execution);
    let branch = match cut {
        Cut::ForkRecoveredUnpublished => fork_branch(&home),
        Cut::Published | Cut::RecoveredUnpublished | Cut::NoEffect => {
            home.runtime().current_world()
        }
    };
    let commits = installed_commits(&home);
    let runtime = home.runtime().runtime();
    let rejection = match cut {
        Cut::Published => None,
        Cut::RecoveredUnpublished | Cut::ForkRecoveredUnpublished | Cut::NoEffect => Some(
            runtime
                .reject_next_commit_before_transaction_for_test()
                .unwrap(),
        ),
    };
    runtime.lose_next_commit_answer_for_test();
    let primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(receipt) =
        observe_on_branch(&home, branch, &execution)
    else {
        panic!("the due intent executes");
    };
    assert_eq!(receipt.indeterminate_operation_count(), 1);
    drop((receipt, rejection));
    let [row] = home
        .runtime()
        .conditional_decision_obligations()
        .try_into()
        .unwrap();
    assert_eq!(
        row.posture,
        primary_graph::QueryConditionalPosture::Indeterminate(
            primary_graph::QueryConditionalOriginal::Unclassified
        ),
        "the live owner records the lost answer before custody leaves it"
    );
    assert_eq!(applies(&home), 1);
    let published = matches!(cut, Cut::Published);
    assert_eq!(revision_on_branch(&home, branch), if published { 2 } else { 1 });
    assert_eq!(installed_commits(&home), commits + usize::from(published));

    // Persist the original key, intent, durable branch ordinal, and (for the
    // Published cut) World's exact original published occurrence identity.
    let mut payload = format!(
        "{}\n{}\n{}",
        encode_hex(&row.key),
        encode_hex(&row.intent),
        row.product.ordinal(),
    );
    if published {
        payload.push('\n');
        payload.push_str(&head_commit_projection(&home, branch));
        assert_eq!(
            home.runtime().conditional_decision_obligations().len(),
            1,
            "the unclassified indeterminate row remains durable until kill"
        );
    }
    std::fs::write(child_reexecution::expected(), payload).unwrap();

    match cut {
        Cut::RecoveredUnpublished => kill_at_world_compare(&home, 2),
        Cut::ForkRecoveredUnpublished => kill_at_world_compare_on_branch(&home, branch, 2),
        Cut::Published | Cut::NoEffect => child_reexecution::wait_for_kill(),
    }
}

fn reader(cut: Cut) {
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = open(&child_reexecution::location(), &execution);
    let expected_raw = std::fs::read_to_string(child_reexecution::expected()).unwrap();
    let mut lines = expected_raw.lines();
    let original_key = decode_hex(lines.next().unwrap());
    let original_intent = decode_hex(lines.next().unwrap());
    let expected_product_ordinal: u64 = lines.next().unwrap().parse().unwrap();
    let expected_published = lines.collect::<Vec<_>>().join("\n");

    let branch = match cut {
        Cut::ForkRecoveredUnpublished => find_fork_branch(&home),
        Cut::Published | Cut::RecoveredUnpublished | Cut::NoEffect => {
            home.runtime().current_world()
        }
    };
    let rows = home.runtime().conditional_decision_obligations();
    let joined_attempt = match cut {
        Cut::Published | Cut::NoEffect => {
            assert!(
                rows.is_empty(),
                "before service the row finished with World's answer: {rows:?}"
            );
            None
        }
        Cut::RecoveredUnpublished | Cut::ForkRecoveredUnpublished => {
            let [row] = rows.as_slice() else {
                panic!("the obligation is retained on its branch: {rows:?}")
            };
            assert_eq!(row.key, original_key);
            assert_eq!(row.intent, original_intent);
            assert_eq!(
                row.product.ordinal(),
                expected_product_ordinal,
                "row kept its exact durable product branch incarnation"
            );
            let primary_graph::QueryConditionalPosture::Indeterminate(
                primary_graph::QueryConditionalOriginal::Recovered(joined),
            ) = row.posture
            else {
                panic!("joined to its recovered original: {row:?}");
            };
            // Verify World's actual recovered original matches the joined attempt.
            let page = home
                .runtime()
                .runtime()
                .product_publication_recovery_page(None, std::num::NonZeroUsize::new(32).unwrap())
                .unwrap();
            let [world_row] = page.rows() else {
                panic!("one recovered World original: {}", page.rows().len());
            };
            let claim = home
                .runtime()
                .runtime()
                .readmit_product_publication_recovery(world_row.handle())
                .unwrap();
            let effects = claim.inspect().unwrap();
            let actual = effects.attempt_identity();
            assert_eq!(actual.owner_identity().durable(), Some(joined.owner()));
            assert_eq!(actual.ordinal(), joined.ordinal());
            drop(effects);
            drop(claim);
            Some(joined)
        }
    };

    let commits = installed_commits(&home);
    let published = matches!(cut, Cut::Published);
    assert_eq!(
        revision_on_branch(&home, branch),
        if published { 2 } else { 1 }
    );

    if matches!(cut, Cut::NoEffect) {
        assert!(
            matches!(
                retry_original_key_on_branch(&home, branch, original_key, original_intent, &execution),
                primary_graph::WorthQueryConditionalOriginalResolution::Absent
            ),
            "before fresh execution, World confirms no original exists"
        );
    }

    let primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(receipt) =
        observe_on_branch(&home, branch, &execution)
    else {
        panic!("the reopened clock observes");
    };
    let (fresh, original) = match cut {
        Cut::Published => (0, 0),
        Cut::RecoveredUnpublished | Cut::ForkRecoveredUnpublished => (0, 1),
        Cut::NoEffect => (1, 0),
    };
    assert_eq!(receipt.committed_operation_count(), fresh);
    assert_eq!(receipt.already_committed_operation_count(), original);
    let observed_commit = receipt
        .execution_provenance()
        .first()
        .and_then(|p| p.published_commit().cloned());
    drop(receipt);

    // Retrying the original intent/key through the conditional/temporal port
    // returns the exact published original occurrence with zero extra applies.
    let primary_graph::WorthQueryConditionalOriginalResolution::Published(recovered) =
        retry_original_key_on_branch(&home, branch, original_key, original_intent, &execution)
    else {
        panic!("original key retry must return the exact published original occurrence");
    };
    let recovered_commit = recovered
        .new_product_head()
        .durable()
        .unwrap()
        .selected_commit();
    match cut {
        Cut::Published => {
            assert_eq!(
                published_occurrence_projection(&recovered),
                expected_published,
                "published retry returns the exact original occurrence from before the kill"
            );
        }
        Cut::RecoveredUnpublished | Cut::ForkRecoveredUnpublished => {
            assert_eq!(
                observed_commit.as_ref(),
                Some(&recovered_commit),
                "continued original reports its exact published commit"
            );
            assert_eq!(
                recovered.attempt_identity().durable(),
                joined_attempt,
                "published occurrence is the exact joined original attempt"
            );
        }
        Cut::NoEffect => {
            assert_eq!(
                observed_commit.as_ref(),
                Some(&recovered_commit),
                "fresh commit and subsequent original-key retry agree on occurrence identity"
            );
        }
    }
    drop(recovered);

    assert!(home.runtime().conditional_decision_obligations().is_empty());
    assert_eq!(
        applies(&home),
        fresh,
        "only a proven no-effect key runs the operation again"
    );
    assert_eq!(revision_on_branch(&home, branch), 2);
    if matches!(cut, Cut::ForkRecoveredUnpublished) {
        assert_eq!(
            revision(&home),
            1,
            "root branch remains untouched by the non-root fork's operation"
        );
    }
    assert_eq!(
        installed_commits(&home),
        commits + usize::from(!published),
        "exactly one original occurrence"
    );
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("the settled Home closes: {refused:?}"))
        .reclaim();

    let home = open(&child_reexecution::location(), &execution);
    let branch = match cut {
        Cut::ForkRecoveredUnpublished => find_fork_branch(&home),
        Cut::Published | Cut::RecoveredUnpublished | Cut::NoEffect => {
            home.runtime().current_world()
        }
    };
    assert!(home.runtime().conditional_decision_obligations().is_empty());
    assert_eq!(revision_on_branch(&home, branch), 2);
    let primary_graph::WorthQueryConditionalClockObservationOutcome::Accepted(receipt) =
        observe_on_branch(&home, branch, &execution)
    else {
        panic!("the reopened clock observes");
    };
    assert_eq!(receipt.committed_operation_count(), 0);
    drop(receipt);
    assert_eq!(applies(&home), 0);
    let _ = observe;
    home.close(&execution)
        .unwrap_or_else(|refused| panic!("the reopened Home closes: {refused:?}"))
        .reclaim();
}
