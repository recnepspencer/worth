//! Concurrent adoption recovery on one expected World head.

use super::*;
use worth_query_host::facade::runtime::NoEffectCause;
use worth_relational::facade::mvcc::{CompanionPreflightStop, RelationalPublicationDeferred};

#[test]
fn two_same_head_adoptions_report_one_performed_world_transition() {
    let host = publish_on_first_program();
    let branch = host.current_world();
    let activation_entity = host
        .runtime()
        .program_activation_entity_for_test()
        .expect("the activation record is published once");
    let target = *host
        .supported_program::<RetentionProgramP1>()
        .expect("P1 is rostered")
        .owned_revision();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let requirements = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .programs()
        .compare(&target)
        .expect("comparison succeeds");
    let prepare = || {
        host.runtime()
            .request(&principal, &scope)
            .on_branch(branch)
            .programs()
            .adopt(&requirements)
            .prepare(64)
    };
    // A same-head publisher meeting the winner mid-flight is told to retry
    // unchanged. Publishing consumes the prepared attempt, so the retry
    // prepares again from the same requirements once the winner has settled,
    // and that preparation sees the moved product.
    let settled = (std::sync::Mutex::new(false), std::sync::Condvar::new());
    let publish = |mut attempt: WorthQueryPreparedBranchAdoption| {
        for _ in 0..SAME_HEAD_RETRIES {
            let outcome = attempt.publish();
            if let WorthQueryBranchAdoptionPublicationOutcome::NoEffect(no_effect) = &outcome {
                if winner_in_flight(no_effect.cause()) {
                    await_settled(&settled);
                    attempt = prepare()?;
                    continue;
                }
            }
            mark_settled(&settled);
            return Ok(outcome);
        }
        panic!("a same-head loser observes the stale head within {SAME_HEAD_RETRIES} retries")
    };
    let publish = &publish;
    let first = prepare().expect("both attempts prepare against the same head");
    let second = prepare().expect("both attempts prepare against the same head");
    let (first_start, first_ready) = std::sync::mpsc::sync_channel(1);
    let (second_start, second_ready) = std::sync::mpsc::sync_channel(1);

    let (first, second) = std::thread::scope(|threads| {
        let first = threads.spawn(move || {
            first_ready
                .recv_timeout(std::time::Duration::from_secs(10))
                .expect("the first prepared adoption receives its bounded start signal");
            publish(first)
        });
        let second = threads.spawn(move || {
            second_ready
                .recv_timeout(std::time::Duration::from_secs(10))
                .expect("the second prepared adoption receives its bounded start signal");
            publish(second)
        });
        first_start
            .send(())
            .expect("the first prepared adoption remains ready");
        second_start
            .send(())
            .expect("the second prepared adoption remains ready");
        (first.join().unwrap(), second.join().unwrap())
    });

    let mut performed = 0;
    let mut retained = 0;
    for outcome in [first, second] {
        let outcome = match outcome {
            Ok(outcome) => outcome,
            Err(denial) => {
                retained += 1;
                assert!(
                    retry_saw_moved_head(&denial),
                    "a retry prepared after the winner sees the stale head: {denial:?}"
                );
                continue;
            }
        };
        match outcome {
            WorthQueryBranchAdoptionPublicationOutcome::Performed(adoption) => {
                performed += 1;
                assert_eq!(adoption.target(), &target);
            }
            WorthQueryBranchAdoptionPublicationOutcome::ProductUnpublished(unpublished) => {
                retained += 1;
                assert_eq!(
                    unpublished.cause(),
                    worth_query_host::facade::runtime::ProductUnpublishedCause::ProductPublicationLost
                );
                host.runtime()
                    .release_branch_adoption_recovery(unpublished.into_recovery(), 0)
                    .expect("the losing exact owner result must release cleanly");
            }
            WorthQueryBranchAdoptionPublicationOutcome::NoEffect(no_effect) => {
                retained += 1;
                assert_eq!(no_effect.cause(), NoEffectCause::StaleExpectedProductHead);
            }
        }
    }
    assert_eq!(performed, 1);
    assert_eq!(retained, 1);
    assert_eq!(
        host.runtime().program_activation_entity_for_test(),
        Some(activation_entity),
        "the losing attempt cannot replace the once-published activation record"
    );

    let adopted = host.current_world();
    assert_eq!(
        settle(set_retention(&host, adopted, P1_VALUE, 0x9175_2021)),
        RetentionVerdict::Performed(P1_VALUE),
        "the next ordinary call must execute under the performed program"
    );
    assert_eq!(
        read_retention(host.runtime(), host.current_world()),
        P1_VALUE,
        "the post-race World must expose one coherent component combination"
    );
}

const SAME_HEAD_RETRIES: usize = 64;

/// The only deferral a same-head loser may see: the winner holds the
/// companion reservation while the expected head is still unchanged.
fn winner_in_flight(cause: NoEffectCause) -> bool {
    cause
        == NoEffectCause::RelationalDeferred(RelationalPublicationDeferred::CompanionPreflight(
            CompanionPreflightStop::TopologyPending,
        ))
}

type Settled = (std::sync::Mutex<bool>, std::sync::Condvar);

fn mark_settled((done, changed): &Settled) {
    *done.lock().unwrap() = true;
    changed.notify_all();
}

fn await_settled((done, changed): &Settled) {
    let (done, wait) = changed
        .wait_timeout_while(
            done.lock().unwrap(),
            std::time::Duration::from_secs(10),
            |done| !*done,
        )
        .unwrap();
    assert!(
        *done && !wait.timed_out(),
        "the winner settles within its bounded wait"
    );
}

/// A retry prepared after the winner settled reads the moved product.
fn retry_saw_moved_head(denial: &WorthQueryApplicationProgramAdoptionPreparationDenial) -> bool {
    matches!(
        denial,
        WorthQueryApplicationProgramAdoptionPreparationDenial::Adoption(
            WorthQueryBranchAdoptionPreparationDenial::RequirementsChanged
        )
    )
}
