//! Two exact-head corrections meet the real World comparison and retain native outcomes.
use super::{authentication::*, edit::Value, host, query::Row};
use std::{num::NonZeroUsize, sync::mpsc, time::Duration};
use worth_query_host::facade::product::WorthQueryApplicationNoEffectCause;
use worth_query_host::facade::{application_entry::*, primary_graph::*, runtime};

pub fn run() {
    let application = host::install();
    let source = host::edit(&application, Value::Primary(11), "race-source");
    host::edit(&application, Value::Secondary(19), "race-unrelated");
    let target = super::required_output_courts::head(&application);
    let pause = application
        .runtime()
        .world_operation_control_for_test()
        .pause_before_product_compare_for_test(NonZeroUsize::MIN);
    let (ready, prepared) = mpsc::sync_channel(2);
    let (first_start, first_run) = mpsc::sync_channel(1);
    let (second_start, second_run) = mpsc::sync_channel(1);
    let (cleanup_start, cleanup_run) = mpsc::sync_channel(1);
    let published = std::thread::scope(|threads| {
        let application = &application;
        let source = &source;
        let first_ready = ready.clone();
        let first = spawn(threads, move || {
            let scope = request_scope();
            let principal = authenticate_operator(application.installed_schema(), &scope);
            let request = application.runtime().request(&principal, &scope);
            let limits = super::required_output_courts::limits();
            let selected = request.corrections().select(source, limits).unwrap();
            let preparation = request
                .corrections()
                .reverse(&selected)
                .idempotency("race-first-correction")
                .limits(limits)
                .prepare()
                .unwrap();
            let WorthQueryApplicationCorrectionPreparation::Prepared(candidate) = preparation
            else {
                panic!("first correction must prepare at the shared target")
            };
            first_ready.send(()).unwrap();
            first_run.recv_timeout(Duration::from_secs(10)).unwrap();
            let outcome = candidate.publish().unwrap();
            let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
                panic!("first correction owns the paused publication: {outcome:?}")
            };
            assert_eq!(
                receipt
                    .aftermath_causality()
                    .unwrap()
                    .recorded_source()
                    .relational(),
                source.commit_reference()
            );
            let publication = receipt.committed_product_publication();
            assert_eq!(
                publication.relational_posture(),
                runtime::CompositeComponentChangePosture::Published
            );
            assert_eq!(
                publication.signal_posture(),
                runtime::CompositeComponentChangePosture::RetainExact
            );
            publication.composite_commit().clone()
        });
        let second = spawn(threads, move || {
            let scope = request_scope();
            let principal = authenticate_operator(application.installed_schema(), &scope);
            let request = application.runtime().request(&principal, &scope);
            let limits = super::required_output_courts::limits();
            let selected = request.corrections().select(source, limits).unwrap();
            let preparation = request
                .corrections()
                .reverse(&selected)
                .idempotency("race-second-correction")
                .limits(limits)
                .prepare()
                .unwrap();
            let WorthQueryApplicationCorrectionPreparation::Prepared(candidate) = preparation
            else {
                panic!("second correction must prepare before first publication")
            };
            ready.send(()).unwrap();
            second_run.recv_timeout(Duration::from_secs(10)).unwrap();
            let outcome = candidate.publish().unwrap();
            // Keep any native recovery on this worker until the winner settles.
            let recovery = match outcome {
                WorthQueryApplicationMutationOutcome::Commit(
                    WorthQueryApplicationUncommitted::Stale(stale),
                ) => {
                    assert!(stale.stale_fact_count() > 0);
                    None
                }
                WorthQueryApplicationMutationOutcome::Commit(
                    WorthQueryApplicationUncommitted::ProductStale(_),
                ) => None,
                WorthQueryApplicationMutationOutcome::Commit(
                    WorthQueryApplicationUncommitted::Denied(denial),
                ) if denial.kind() == WorthQueryApplicationCommitDenialKind::ProductBasisStale => {
                    None
                }
                WorthQueryApplicationMutationOutcome::Commit(
                    WorthQueryApplicationUncommitted::NoEffect(no_effect),
                ) => {
                    assert!(matches!(no_effect.cause(), WorthQueryApplicationNoEffectCause::OwnerDeniedBeforeEffect | WorthQueryApplicationNoEffectCause::RelationalDeferred(worth_relational::facade::mvcc::RelationalPublicationDeferred::PatchPositionReservationContended)), "unrelated no-effect reason: {no_effect:?}");
                    None
                }
                WorthQueryApplicationMutationOutcome::Commit(
                    WorthQueryApplicationUncommitted::Deferred(deferred),
                ) => {
                    assert!(matches!(deferred.kind(), WorthQueryApplicationCommitDeferredKind::PatchPositionReservationContended | WorthQueryApplicationCommitDeferredKind::SourceCurrentnessRaced(_)), "unrelated deferral: {deferred:?}");
                    None
                }
                WorthQueryApplicationMutationOutcome::Commit(
                    WorthQueryApplicationUncommitted::ProductUnpublished(unpublished),
                ) => {
                    assert_eq!(
                        unpublished.cause(),
                        runtime::ProductUnpublishedCause::ProductPublicationLost
                    );
                    assert!(unpublished.owner_effect_count() > 0);
                    Some(unpublished.into_recovery())
                }
                other => {
                    panic!("same-head loser must retain its precise native race outcome: {other:?}")
                }
            };
            cleanup_run.recv_timeout(Duration::from_secs(10)).unwrap();
            if let Some(recovery) = recovery {
                application
                    .runtime()
                    .release_product_publication_recovery(recovery, 0)
                    .unwrap();
            }
        });
        prepared.recv_timeout(Duration::from_secs(10)).unwrap();
        prepared.recv_timeout(Duration::from_secs(10)).unwrap();
        assert_eq!(super::required_output_courts::head(application), target);
        first_start.send(()).unwrap();
        if !pause.wait_until_reached(Duration::from_secs(10)) {
            pause.release();
            panic!("first corrective owner must reach the actual World CAS");
        }
        assert_eq!(super::required_output_courts::head(application), target);
        let contention = application
            .runtime()
            .observe_application_branch_commit_contention_for_test();
        second_start.send(()).unwrap();
        if !contention.wait_until_blocked(Duration::from_secs(10)) {
            pause.release();
            cleanup_start.send(()).unwrap();
            panic!("second correction must reach actual native branch-lane contention");
        }
        assert_eq!(super::required_output_courts::head(application), target);
        pause.release();
        let published = first.join().unwrap();
        cleanup_start.send(()).unwrap();
        second.join().unwrap();
        published
    });
    application
        .with_host_advancement(|phase| {
            let selected = application
                .runtime()
                .on_branch(application.runtime().current_world())
                .select(&phase)
                .unwrap();
            assert_eq!(selected.product().selected_commit(), &published);
        })
        .unwrap();
    assert_eq!(
        host::read(&application),
        [Row {
            primary: 5,
            secondary: 19
        }]
    );
    println!("same-head corrective publishers serialize at the native branch lane and retain one coherent performed World");
}

/// Each racing request gets the court thread's stack: an ordinary correction
/// runs deeper than a default thread allows in an unoptimized build.
fn spawn<'scope, T: Send + 'scope>(
    threads: &'scope std::thread::Scope<'scope, '_>,
    run: impl FnOnce() -> T + Send + 'scope,
) -> std::thread::ScopedJoinHandle<'scope, T> {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn_scoped(threads, run)
        .expect("start a racing request")
}
