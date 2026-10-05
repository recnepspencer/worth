//! Exact World recovery for a branch adoption whose Relational effect already exists.

use worth_query_host::facade::application_entry::{
    WorthQueryApplicationProgramAdoptionPreparationDenial,
    WorthQueryApplicationProgramAdoptionRecoveryFailure, WorthQueryApplicationRequestExt,
    WorthQueryBranchAdoptionPublicationOutcome, WorthQueryBranchAdoptionRecoveryOutcome,
    WorthQueryPreparedBranchAdoption,
};
use worth_query_host::facade::application_installation::WorthQueryProgramOwner;
use worth_query_host::facade::primary_graph::WorthQueryBranchAdoptionPreparationDenial;
use worth_query_host::facade::primary_graph::WorthQueryBranchAdoptionRecoveryDenial;
use worth_query_host::facade::runtime::NoEffectCause;
use worth_relational::facade::mvcc::{CompanionPreflightStop, RelationalPublicationDeferred};

use crate::document_retention_model::host::publish_on_first_program;
use crate::document_retention_model::operator_identity::{authenticate_operator, request_scope};
use crate::document_retention_model::presented_request::set_retention;
use crate::document_retention_model::programs::RetentionProgramP1;
use crate::document_retention_model::readback::read_retention;
use crate::document_retention_model::settled_verdict::{settle, RetentionVerdict};

const P1_VALUE: u64 = 15;

#[test]
fn unpublished_adoption_settles_and_publishes_without_replaying_relational_work() {
    let host = publish_on_first_program();
    let branch = host.current_world();
    let target = *host
        .supported_program::<RetentionProgramP1>()
        .expect("P1 is rostered")
        .owned_revision();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let programs = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .programs();
    let requirements = programs.compare(&target).expect("comparison succeeds");
    let prepared = programs
        .adopt(&requirements)
        .prepare(64)
        .expect("adoption prepares");

    host.runtime().fail_next_durable_append_for_test();
    let unpublished = match prepared.publish() {
        WorthQueryBranchAdoptionPublicationOutcome::ProductUnpublished(unpublished) => unpublished,
        WorthQueryBranchAdoptionPublicationOutcome::Performed(_) => {
            panic!("the injected durability loss must retain exact custody")
        }
        WorthQueryBranchAdoptionPublicationOutcome::NoEffect(no_effect) => {
            panic!("owner settlement failure is not pre-effect no-effect: {no_effect:?}")
        }
    };
    assert!(unpublished.relational_requires_settlement());
    assert_eq!(unpublished.source(), requirements.source());
    assert_eq!(unpublished.target(), &target);

    let recovery = unpublished.into_recovery();
    let outcome = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .programs()
        .recover(recovery)
        .unwrap_or_else(|_| panic!("the exact branch must recover its adoption"));
    let performed = match outcome {
        WorthQueryBranchAdoptionRecoveryOutcome::Performed { adoption, cleanup } => {
            cleanup.expect("the superseded recovery record and owner cleanup must drain");
            adoption
        }
        WorthQueryBranchAdoptionRecoveryOutcome::NoEffect { no_effect, .. } => {
            panic!("settled adoption unexpectedly had no effect: {no_effect:?}")
        }
        WorthQueryBranchAdoptionRecoveryOutcome::ProductUnpublished { next, .. } => {
            panic!("settled adoption remained unpublished: {next:?}")
        }
    };
    assert_eq!(performed.source(), requirements.source());
    assert_eq!(performed.target(), &target);
    assert_eq!(
        performed.relational_owner_contacts(),
        0,
        "recovery adopts the settled result and never replays the Relational mutation"
    );

    let adopted = host.current_world();
    assert_eq!(
        settle(set_retention(&host, adopted, P1_VALUE, 0x9175_2001)),
        RetentionVerdict::Performed(P1_VALUE)
    );
    assert_eq!(read_retention(host.runtime(), adopted), P1_VALUE);
}

#[test]
fn sibling_branch_cannot_consume_an_unpublished_adoption_recovery() {
    let host = publish_on_first_program();
    let branch = host.current_world();
    let sibling = host
        .runtime()
        .branches()
        .fork(branch)
        .components(|components| components.fork_relational().reuse_exact_signal_basis())
        .create()
        .expect("sibling branch publishes");
    let target = *host
        .supported_program::<RetentionProgramP1>()
        .expect("P1 is rostered")
        .owned_revision();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let programs = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .programs();
    let requirements = programs.compare(&target).expect("comparison succeeds");
    let prepared = programs
        .adopt(&requirements)
        .prepare(64)
        .expect("adoption prepares");
    host.runtime().fail_next_durable_append_for_test();
    let recovery = match prepared.publish() {
        WorthQueryBranchAdoptionPublicationOutcome::ProductUnpublished(unpublished) => {
            unpublished.into_recovery()
        }
        _ => panic!("the injected durability loss must retain exact custody"),
    };

    let failure = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(sibling)
        .programs()
        .recover(recovery)
        .err()
        .expect("a sibling cannot consume foreign adoption custody");
    let WorthQueryApplicationProgramAdoptionRecoveryFailure::Recovery(failure) = failure else {
        panic!("the sibling remains selectable, so execution must reject affinity")
    };
    assert!(matches!(
        failure.denial(),
        WorthQueryBranchAdoptionRecoveryDenial::ProductAffinityMismatch
    ));
    let recovery = failure.into_recovery();

    let outcome = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .programs()
        .recover(recovery)
        .unwrap_or_else(|_| panic!("the owning branch must retain recovery authority"));
    match outcome {
        WorthQueryBranchAdoptionRecoveryOutcome::Performed { adoption, cleanup } => {
            cleanup.expect("recovery cleanup drains");
            assert_eq!(adoption.target(), &target);
            assert_eq!(adoption.relational_owner_contacts(), 0);
        }
        _ => panic!("the exact owning branch must complete recovery"),
    }

    assert_eq!(
        settle(set_retention(&host, sibling, 3, 0x9175_2011)),
        RetentionVerdict::Performed(3),
        "the sibling remains on P0"
    );
}

#[test]
fn performed_recovery_retains_cleanup_failure_after_the_effect() {
    let host = publish_on_first_program();
    let branch = host.current_world();
    let target = *host
        .supported_program::<RetentionProgramP1>()
        .expect("P1 is rostered")
        .owned_revision();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let programs = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .programs();
    let requirements = programs.compare(&target).expect("comparison succeeds");
    let prepared = programs
        .adopt(&requirements)
        .prepare(64)
        .expect("adoption prepares");
    host.runtime().fail_next_durable_append_for_test();
    let recovery = match prepared.publish() {
        WorthQueryBranchAdoptionPublicationOutcome::ProductUnpublished(unpublished) => {
            unpublished.into_recovery()
        }
        _ => panic!("the injected durability loss must retain exact custody"),
    };

    let outcome = host
        .runtime()
        .with_owner_cleanup_capacity_exhausted_for_test(|| {
            host.runtime()
                .request(&principal, &scope)
                .on_branch(branch)
                .programs()
                .recover(recovery)
                .unwrap_or_else(|_| panic!("cleanup capacity cannot erase the performed effect"))
        });
    let (adoption, cleanup_failure) = match outcome {
        WorthQueryBranchAdoptionRecoveryOutcome::Performed { adoption, cleanup } => (
            adoption,
            cleanup.expect_err("the performed terminal must retain cleanup-capacity failure"),
        ),
        _ => panic!("cleanup failure occurs after the adoption has performed"),
    };
    assert_eq!(adoption.target(), &target);
    assert_eq!(adoption.relational_owner_contacts(), 0);
    let retained = cleanup_failure
        .into_recovery()
        .expect("capacity denial retains the exact prior recovery record");
    host.runtime()
        .release_product_publication_recovery(retained, 0)
        .expect("released cleanup capacity permits exact retry");
}

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
