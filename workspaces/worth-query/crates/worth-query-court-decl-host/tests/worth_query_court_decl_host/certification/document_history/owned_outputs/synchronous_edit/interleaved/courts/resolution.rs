//! Whole-intent confirmation never promotes an unseen or pending request.
use super::*;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationRequestMutationDenial, WorthQueryEditEnvelopeExecutionDenial,
    WorthQueryPerformedMutationExecutionDenial,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationIdempotencyResolutionDenialKind, WorthQueryApplicationUncommitted,
    WorthQueryExecutionObservation,
};

#[test]
fn unseen_resolution_precedes_supplier_planning_and_does_not_execute() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = host(None, None);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    for value in [1001, 3] {
        let make = || {
            let mut edit = request
                .edit_envelope()
                .idempotency(format!("resolve-unseen-{value}"))
                .runtime_stages(&host);
            edit.push_stage::<ProducedTree, _>(dependent::Intent { add: 7 })
                .unwrap();
            edit.push_stage::<ModelTree, _>(fixture::model::Intent {
                key: "a".into(),
                value,
                key_derived_target: false,
            })
            .unwrap();
            edit.push_stage::<ModelTree, _>(fixture::model::Intent {
                key: "b".into(),
                value: 4,
                key_derived_target: false,
            })
            .unwrap();
            edit
        };
        let before = super::super::super::courts::native_head(&host);
        let contacts = host.producer_contacts_on_this_thread_for_test();
        let (outcome, writes) = observe_maintenance_work(16, || {
            make()
                .resolve_idempotency(ExecutionAllocationPolicy::SystemAllocation)
                .unwrap()
        });
        assert!(matches!(outcome, Outcome::Unseen), "{outcome:?}");
        assert_eq!(super::super::super::courts::native_head(&host), before);
        assert_eq!(host.producer_contacts_on_this_thread_for_test(), contacts);
        assert_eq!(writes.published_rows.len() + writes.omitted_publications, 0);
        // 1001 would refuse the real supplier; 3 remains available to execute.
        if value == 3 {
            let outcome = make()
                .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
                .unwrap();
            let Outcome::Committed(receipt) = outcome else {
                panic!("{outcome:?}")
            };
            assert_eq!(receipt.stage_count(), 3);
            assert_eq!(
                super::super::super::courts::native_head(&host).1,
                [before.0]
            );
        }
    }
}

#[test]
fn resolution_confirms_the_original_whole_intent_after_a_later_commit() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = host(None, None);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let make = |second, ordered, reverse| {
        let edit = request
            .edit_envelope()
            .idempotency("resolve-original".to_owned())
            .runtime_stages(&host);
        let mut edit = if ordered {
            edit.ordered_authored().unwrap()
        } else {
            edit
        };
        let mut inputs = vec![
            created::Intent {
                key: "resolve:first".into(),
                previous: None,
                value: 3,
            },
            created::Intent {
                key: "resolve:second".into(),
                previous: Some("resolve:first".into()),
                value: second,
            },
        ];
        if reverse {
            inputs.reverse();
        }
        for input in inputs {
            edit.push_stage::<ModelTree, _>(input).unwrap();
        }
        edit
    };
    let outcome = make(4, true, false)
        .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let Outcome::Committed(original) = outcome else {
        panic!("{outcome:?}")
    };
    let first_head = super::super::super::courts::native_head(&host);
    let mut later = request
        .edit_envelope()
        .idempotency("resolve-later".to_owned())
        .runtime_stages(&host);
    later
        .push_stage::<ModelTree, _>(created::Intent {
            key: "resolve:later".into(),
            previous: None,
            value: 1,
        })
        .unwrap();
    let outcome = later
        .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(outcome, Outcome::Committed(_)), "{outcome:?}");
    let current = super::super::super::courts::native_head(&host);
    assert_ne!(current, first_head);
    for (second, ordered, reverse, same) in [
        (4, true, false, true),
        (5, true, false, false),
        (4, false, false, false),
        (4, true, true, false),
    ] {
        let contacts = host.producer_contacts_on_this_thread_for_test();
        let (outcome, writes) = observe_maintenance_work(16, || {
            make(second, ordered, reverse)
                .resolve_idempotency(ExecutionAllocationPolicy::SystemAllocation)
                .unwrap()
        });
        if same {
            let Outcome::RecoveredCommitted(recorded) = outcome else {
                panic!("{outcome:?}")
            };
            assert!(original.receipt().matches_recorded_original(&recorded));
            assert_eq!(
                original.receipt().commit_reference(),
                recorded.commit_reference()
            );
        } else {
            assert!(
                matches!(outcome, Outcome::IdempotencyIntentDrift),
                "{outcome:?}"
            );
        }
        assert_eq!(super::super::super::courts::native_head(&host), current);
        assert_eq!(host.producer_contacts_on_this_thread_for_test(), contacts);
        assert_eq!(writes.published_rows.len() + writes.omitted_publications, 0);
    }
}

#[test]
fn resolution_preserves_pending_publication_until_execution_resumes_it() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = host(None, None);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let make = |key: &str| {
        let mut edit = request
            .edit_envelope()
            .idempotency(key.to_owned())
            .runtime_stages(&host);
        edit.push_stage::<ModelTree, _>(fixture::model::Intent {
            key: "a".into(),
            value: 3,
            key_derived_target: false,
        })
        .unwrap();
        edit
    };
    let guard = host.hold_post_commit_snapshot_failure_for_test().unwrap();
    let pending = make("resolve-pending")
        .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(&pending, Outcome::Uncommitted(_)), "{pending:?}");
    let held = super::super::super::courts::native_head(&host);
    // Recovery must now be able to run: the observation cannot pass because
    // the injected failure happened to suppress an accidental resume.
    drop(guard);
    for key in ["resolve-pending", "resolve-other"] {
        let contacts = host.producer_contacts_on_this_thread_for_test();
        let (outcome, writes) = observe_maintenance_work(16, || {
            make(key).resolve_idempotency(ExecutionAllocationPolicy::SystemAllocation)
        });
        unavailable(outcome);
        assert_eq!(super::super::super::courts::native_head(&host), held);
        assert_eq!(host.producer_contacts_on_this_thread_for_test(), contacts);
        assert_eq!(writes.published_rows.len() + writes.omitted_publications, 0);
    }
    let outcome = make("resolve-pending")
        .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(outcome, Outcome::RecoveredCommitted(_)),
        "{outcome:?}"
    );
    assert_ne!(super::super::super::courts::native_head(&host), held);
    drop(pending);
}

fn unavailable(outcome: Result<Outcome, WorthQueryEditEnvelopeExecutionDenial>) {
    match outcome {
        Ok(Outcome::Uncommitted(
            WorthQueryApplicationUncommitted::ProductUnpublished(_)
            | WorthQueryApplicationUncommitted::ProductPreparationPending(_),
        )) => {}
        Err(denial) => match denial.cause() {
            WorthQueryPerformedMutationExecutionDenial::Mutation(
                WorthQueryApplicationRequestMutationDenial::Idempotency(cause),
            ) => {
                assert_eq!(
                    cause.kind(),
                    WorthQueryApplicationIdempotencyResolutionDenialKind::ProviderUnavailable
                );
            }
            other => panic!("unexpected pending refusal: {other:?}"),
        },
        other => panic!("pending publication was not retained: {other:?}"),
    }
}
