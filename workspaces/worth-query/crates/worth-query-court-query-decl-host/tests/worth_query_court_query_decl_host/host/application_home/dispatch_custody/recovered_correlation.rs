//! A reopened correlation index selects actual source rows without contacting a rail.
use super::*;
use worth_query_host::facade::primary_graph::{
    ExternalEffectCorrelationIdentity, WorthQueryCommittedDispatchOutboxObservation,
    WorthQueryDispatchOutboxRecord,
};

fn record_notice(home: &Home, key: u64, balance: u64) -> WorthQueryDispatchOutboxRecord {
    let (_, scope) = pending_support::scope();
    let runtime = home.runtime();
    let principal = identity::inspection_authentication::authenticate(
        runtime.runtime().installed_schema(),
        &scope,
    );
    let outcome = runtime
        .request(&principal, &scope)
        .advanced()
        .mutate(NotifyAccountBalance { balance })
        .without_source()
        .idempotency(&key)
        .execute_in_program(
            runtime,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("the genuine keyed mutation commits");
    let WorthQueryApplicationMutationOutcome::Committed { receipt, result } = outcome else {
        panic!("the original notification commits: {outcome:?}");
    };
    assert_eq!(result, balance);
    receipt.dispatch_outbox().unwrap().clone()
}

fn observe(
    home: &Home,
    correlation: &ExternalEffectCorrelationIdentity,
) -> WorthQueryCommittedDispatchOutboxObservation {
    let (_, scope) = pending_support::scope();
    home.runtime()
        .runtime()
        .with_application_advancement(&scope, |phase| {
            home.runtime()
                .runtime()
                .observe_committed_dispatch_outbox_by_correlation(&phase, correlation)
        })
        .unwrap()
        .expect("the original exact source is readable by correlation")
}

#[test]
fn recovered_correlations_keep_their_original_rows_and_world_publications_without_contact() {
    let location = dispatch::location();
    let contacts = Arc::new(AtomicUsize::new(0));
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let rail = ScriptedRail::answering(&[Outcome::LostResponse, Outcome::TimedOut]);
    let home = open(&location, execution, &contacts, true, &rail);
    let records = [
        record_notice(&home, 0x8b01, 31),
        record_notice(&home, 0x8b02, 32),
    ];
    let originals = records
        .iter()
        .map(|record| {
            let observed = observe(&home, record.correlation());
            assert_eq!(observed.record(), record);
            (
                observed.commit_reference().clone(),
                observed.record_ref().clone(),
                observed
                    .committed_product_publication()
                    .composite_commit()
                    .clone(),
                observed
                    .committed_product_publication()
                    .publication_attempt()
                    .clone(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(rail.contacts().len(), 2);
    Arc::into_inner(home)
        .expect("exclusive original Home")
        .close(&execution)
        .unwrap_or_else(|cause| panic!("the original Home closes: {cause:?}"))
        .reclaim();
    // Rebuild every process-local selector twice, from two fresh Home runtimes.
    for _opening in 0..2 {
        let rail = ScriptedRail::answering(&[]);
        let home = open(&location, execution, &contacts, false, &rail);
        for (record, original) in records.iter().zip(&originals) {
            let observed = observe(&home, record.correlation());
            assert_eq!(observed.record(), record);
            assert_eq!(observed.commit_reference(), &original.0);
            assert_eq!(observed.record_ref(), &original.1);
            let publication = observed.committed_product_publication();
            let commit = publication.composite_commit();
            let attempt = publication.publication_attempt();
            // Durable coordinates survive; admission identity belongs to this opening.
            assert_eq!(
                commit.owner_identity().durable(),
                original.2.owner_identity().durable()
            );
            assert_eq!(commit.ordinal(), original.2.ordinal());
            assert_eq!(
                attempt.owner_identity().durable(),
                original.3.owner_identity().durable()
            );
            assert_eq!(attempt.ordinal(), original.3.ordinal());
            assert_eq!(attempt.owner_identity(), commit.owner_identity());
            assert_ne!(
                commit.owner_identity(),
                original.2.owner_identity(),
                "a recovered description uses fresh admission, never the former live owner"
            );
        }
        assert!(
            rail.contacts().is_empty(),
            "source readmission and lookup never contact the rail"
        );
        assert_eq!(contacts.load(Ordering::SeqCst), 2, "no handler reexecution");
        Arc::into_inner(home)
            .expect("exclusive reopened Home")
            .close(&execution)
            .unwrap_or_else(|cause| panic!("the reconstructed index retires: {cause:?}"))
            .reclaim();
    }
}
