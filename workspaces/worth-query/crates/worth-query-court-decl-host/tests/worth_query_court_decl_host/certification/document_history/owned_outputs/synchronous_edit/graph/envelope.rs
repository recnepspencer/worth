//! The public envelope keeps one commit and a request-branded typed result.
use super::*;
use worth_query_host::facade::application_entry::WorthQueryEditEnvelopeOutcome as Outcome;
use worth_relational::facade::inspection::observe_maintenance_work;
fn exercise(worker: bool) {
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let host: WorthQueryProgramApplicationRuntime<Schema, program::Program> =
        world::install_program_with_value(limits, 2);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let build = || {
        request
            .edit_envelope()
            .stage::<program::Tree, _>(edit::Intent {
                key: "a".into(),
                value: 3,
            })
            .idempotency("one-envelope".to_owned())
    };
    let envelope = build();
    let token = envelope.stage_token::<edit::Binding>(0).unwrap();
    let foreign = build().stage_token::<edit::Binding>(0).unwrap();
    let before = courts::native_head(&host).0;
    let (outcome, published) = observe_maintenance_work(8, || {
        envelope
            .execute_performed::<program::Program>(
                &host,
                ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap()
    });
    let Outcome::Committed(receipt) = outcome else {
        panic!("envelope did not commit")
    };
    assert_eq!(receipt.stage_count(), 1);
    let _: &edit::Edited = receipt.result(&token).unwrap();
    assert!(receipt.result(&foreign).is_err());
    assert_eq!(
        published.published_rows.len() + published.omitted_publications,
        1
    );
    assert_eq!(courts::native_head(&host).1, [before]);
    assert_eq!(
        request
            .query(query::model::Read("a".to_owned()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        3
    );
    let head = courts::native_head(&host);
    let (replay, published) = observe_maintenance_work(8, || {
        build()
            .execute_performed::<program::Program>(
                &host,
                ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap()
    });
    let Outcome::RecoveredCommitted(replayed) = replay else {
        panic!("envelope replay was not RecoveredCommitted")
    };
    // D1675/D1676: canonical owner readmission carries durable meaning,
    // never reconstructed live session custody or per-stage results.
    assert_eq!(
        replayed.commit_reference(),
        receipt.receipt().commit_reference()
    );
    assert_eq!(
        replayed.outcome_identity(),
        receipt.receipt().outcome_identity()
    );
    assert_eq!(
        replayed.changed_record_count(),
        receipt.receipt().changed_record_count()
    );
    assert_eq!(
        replayed.emitted_effect_count(),
        receipt.receipt().emitted_effect_count()
    );
    let recorded_generation = match replayed.new_product_head() {
        worth_query_host::facade::primary_graph::RecordedProductBranchReference::Home(head) => {
            head.generation().get()
        }
        worth_query_host::facade::primary_graph::RecordedProductBranchReference::Memory(head) => {
            head.generation()
        }
    };
    assert_eq!(
        recorded_generation,
        receipt
            .receipt()
            .committed_product_publication()
            .product_generation()
            .get()
    );
    eprintln!(
        "recovered commit={:?}; recorded head={:?}",
        replayed.commit_reference().commit_id,
        replayed.new_product_head()
    );
    assert_eq!(
        published.published_rows.len() + published.omitted_publications,
        0
    );
    assert_eq!(courts::native_head(&host), head);
}
#[test]
fn one_stage_envelope_commits_once_and_recovers_committed_meaning() {
    exercise(false);
}
#[test]
fn one_stage_envelope_commits_once_and_recovers_committed_meaning_worker() {
    exercise(true);
}
#[test]
fn repeated_binding_refuses_a_named_declared_scope_cycle() {
    repeated_scope(false);
}
#[test]
fn repeated_binding_refuses_a_named_declared_scope_cycle_worker() {
    repeated_scope(true);
}
fn repeated_scope(worker: bool) {
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let host: WorthQueryProgramApplicationRuntime<Schema, program::Program> =
        world::install_program_with_other_model(limits, 2, Some(9));
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = courts::native_head(&host);
    let outcome = request
        .edit_envelope()
        .stage::<program::Tree, _>(edit::Intent {
            key: "a".into(),
            value: 3,
        })
        .stage::<program::Tree, _>(edit::Intent {
            key: "b".into(),
            value: 4,
        })
        .idempotency("declared-cycle".to_owned())
        .execute_performed::<program::Program>(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let Outcome::CyclicStages(cycle) = outcome else {
        panic!("declared overlap was not refused")
    };
    assert_eq!(
        cycle
            .stages
            .iter()
            .map(|stage| stage.identity.ordinal())
            .collect::<Vec<_>>(),
        [0, 1]
    );
    assert!(cycle
        .stages
        .iter()
        .all(|stage| stage.binding == edit::Binding::IDENTITY));
    assert_eq!(cycle.overlaps.len(), 2);
    assert_eq!(courts::native_head(&host), before);
    assert_eq!(
        request
            .query(query::model::Read("a".to_owned()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        2
    );
}
