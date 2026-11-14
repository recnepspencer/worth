//! Runtime collection and semantic ordered Creates use actual candidate data.
use super::*;
#[test]
fn positioned_creates_keep_earlier_data_and_one_commit_or_discard_everything() {
    let _execution = worth_query_host::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
    let host = host(None, None);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = super::super::super::courts::native_head(&host);
    let mut edit = request
        .edit_envelope()
        .idempotency("ordered-creates".to_owned())
        .runtime_stages(&host)
        .ordered_authored()
        .unwrap();
    let first = edit
        .push_stage::<ModelTree, _>(created::Intent {
            key: "created:first".into(),
            previous: None,
            value: 3,
        })
        .unwrap();
    let second = edit
        .push_stage::<ModelTree, _>(created::Intent {
            key: "created:second".into(),
            previous: Some("created:first".into()),
            value: 4,
        })
        .unwrap();
    assert_eq!((first.ordinal(), second.ordinal()), (0, 1));
    let (outcome, writes) = observe_maintenance_work(16, || {
        edit.execute_performed(ExecutionAllocationPolicy::SystemAllocation)
            .unwrap()
    });
    let Outcome::Committed(receipt) = outcome else {
        panic!("{outcome:?}")
    };
    assert_eq!(receipt.result(&first).unwrap().value, 3);
    let mut foreign = request
        .edit_envelope()
        .idempotency("other-envelope".to_owned())
        .runtime_stages(&host);
    let foreign_token = foreign
        .push_stage::<ModelTree, _>(created::Intent {
            key: "never-executed".into(),
            previous: None,
            value: 1,
        })
        .unwrap();
    assert!(matches!(
        receipt.result(&foreign_token),
        Err(WorthQueryEditStageResultDenial::ForeignEnvelope)
    ));
    drop(foreign);

    assert_eq!(receipt.result(&second).unwrap().value, 7);
    assert_eq!(receipt.result(&second).unwrap().ordinal, 1);
    assert_eq!(receipt.result(&second).unwrap().key, "ordered-creates");
    assert_eq!(writes.published_rows.len() + writes.omitted_publications, 1);
    assert_eq!(
        super::super::super::courts::native_head(&host).1,
        [before.0]
    );
    let committed = super::super::super::courts::native_head(&host);
    for (ordered, reverse, expected_replay) in [
        (true, false, true),
        (false, false, false),
        (true, true, false),
    ] {
        let edit = request
            .edit_envelope()
            .idempotency("ordered-creates".to_owned())
            .runtime_stages(&host);
        let mut edit = if ordered {
            edit.ordered_authored().unwrap()
        } else {
            edit
        };
        let mut inputs = vec![
            created::Intent {
                key: "created:first".into(),
                previous: None,
                value: 3,
            },
            created::Intent {
                key: "created:second".into(),
                previous: Some("created:first".into()),
                value: 4,
            },
        ];
        if reverse {
            inputs.reverse();
        }
        for input in inputs {
            edit.push_stage::<ModelTree, _>(input).unwrap();
        }
        let ((outcome, contacts), writes) = observe_maintenance_work(16, || {
            let before = host.producer_contacts_on_this_thread_for_test();
            let outcome = edit
                .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
                .unwrap();
            (
                outcome,
                host.producer_contacts_on_this_thread_for_test() - before,
            )
        });
        if expected_replay {
            let Outcome::RecoveredCommitted(original) = outcome else {
                panic!("{outcome:?}")
            };
            assert!(receipt.receipt().matches_recorded_original(&original));
        } else {
            assert!(
                matches!(outcome, Outcome::IdempotencyIntentDrift),
                "{outcome:?}"
            );
        }
        assert_eq!(super::super::super::courts::native_head(&host), committed);
        assert_eq!(contacts, 0);
        assert_eq!(writes.published_rows.len() + writes.omitted_publications, 0);
    }
    let mut failed = request
        .edit_envelope()
        .idempotency("ordered-refused".to_owned())
        .runtime_stages(&host)
        .ordered_authored()
        .unwrap();
    failed
        .push_stage::<ModelTree, _>(created::Intent {
            key: "uncommitted:first".into(),
            previous: None,
            value: 1,
        })
        .unwrap();
    let refused = failed
        .push_stage::<ModelTree, _>(created::Intent {
            key: "uncommitted:second".into(),
            previous: Some("uncommitted:first".into()),
            value: 1001,
        })
        .unwrap();
    let (outcome, writes) = observe_maintenance_work(16, || {
        failed
            .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
            .unwrap()
    });
    let Outcome::DomainDenied(denial) = outcome else {
        panic!("{outcome:?}")
    };
    assert_eq!(denial.denial(&refused).unwrap(), &Denial::ValueOutOfRange);
    assert_eq!(writes.published_rows.len() + writes.omitted_publications, 0);
    assert_eq!(super::super::super::courts::native_head(&host), committed);
    let members = request
        .query(query::members::Read("a".into()))
        .execute()
        .unwrap();
    assert!(!members.rows()[0]
        .members
        .iter()
        .any(|m| m.key.starts_with("uncommitted:")));
}
#[test]
fn positioned_sources_wait_for_a_real_producer_between_positions() {
    let host = host(None, None);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut edit = request
        .edit_envelope()
        .idempotency("positioned-producer-barrier".to_owned())
        .runtime_stages(&host)
        .ordered_authored()
        .unwrap();
    edit.push_stage::<ModelTree, _>(fixture::model::Intent {
        key: "a".into(),
        value: 3,
        key_derived_target: false,
    })
    .unwrap();
    let dependent = edit
        .push_stage::<ProducedTree, _>(dependent::Intent { add: 7 })
        .unwrap();
    let before = super::super::super::courts::native_head(&host).0;
    let (outcome, writes) = observe_maintenance_work(16, || {
        edit.execute_performed(ExecutionAllocationPolicy::SystemAllocation)
            .unwrap()
    });
    let Outcome::Committed(receipt) = outcome else {
        panic!("{outcome:?}")
    };
    assert_eq!(receipt.result(&dependent).unwrap().value, 37);
    assert_eq!(receipt.result(&dependent).unwrap().ordinal, 1);
    assert_eq!(writes.published_rows.len() + writes.omitted_publications, 1);
    assert_eq!(super::super::super::courts::native_head(&host).1, [before]);
}

#[test]
fn positioned_conflicts_and_late_source_changes_keep_atomic_refusal() {
    for late_source_change in [false, true] {
        let host = host(None, None);
        let scope = request_scope();
        let principal = authenticate_operator(host.installed_schema(), &scope);
        let request = host.request(&principal, &scope);
        let mut edit = request
            .edit_envelope()
            .idempotency(format!("positioned-support:{late_source_change}"))
            .runtime_stages(&host)
            .ordered_authored()
            .unwrap();
        if late_source_change {
            edit.push_stage::<ModelTree, _>(fixture::cap::Intent {
                key: "envelope-cap".into(),
                value: 8,
                key_derived_target: false,
            })
            .unwrap();
            edit.push_stage::<ProducedTree, _>(dependent::Intent { add: 7 })
                .unwrap();
        } else {
            edit.push_stage::<ModelTree, _>(fixture::model::Intent {
                key: "a".into(),
                value: 2,
                key_derived_target: false,
            })
            .unwrap();
        }
        edit.push_stage::<ModelTree, _>(fixture::model::Intent {
            key: "a".into(),
            value: 3,
            key_derived_target: false,
        })
        .unwrap();
        let before = super::super::super::courts::native_head(&host);
        let target = request
            .query(query::model::Read("a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .native_id;
        let (outcome, writes) = observe_maintenance_work(24, || {
            edit.execute_performed(ExecutionAllocationPolicy::SystemAllocation)
                .unwrap()
        });
        let Outcome::Uncommitted(WorthQueryApplicationUncommitted::Denied(cause)) = outcome else {
            panic!("{outcome:?}")
        };
        if late_source_change {
            assert_eq!(
                cause.invariant_execution_failure().unwrap().kind(),
                WorthQueryInvariantExecutionDenialKind::CoupledReadSupportChanged
            );
        } else {
            assert_eq!(
                cause.coupled_producer_denial().unwrap().kind(),
                WorthQueryOutputDemandDenialKind::ConflictingIntent {
                    target: worth_relational::facade::transactions::RecordRef::Entity(target),
                    earlier: 0,
                    later: 1,
                }
            );
        }
        assert_eq!(writes.published_rows.len() + writes.omitted_publications, 0);
        assert_eq!(super::super::super::courts::native_head(&host), before);
        assert_eq!(
            request
                .query(query::model::Read("a".into()))
                .execute()
                .unwrap()
                .rows()[0]
                .value,
            1
        );
        assert!(request
            .query(query::first_output::Read("first:a".into()))
            .execute()
            .is_err());
    }
}
