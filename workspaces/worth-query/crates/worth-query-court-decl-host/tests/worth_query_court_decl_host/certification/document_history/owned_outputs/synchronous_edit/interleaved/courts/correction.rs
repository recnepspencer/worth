//! Only the genuine corrective root can supply the recorded Member inverse.
use super::super::super::super::authored_member as member;
use super::*;
use std::num::NonZeroUsize;
fn corrected_host(wrong: bool) -> Host {
    world::install_configured(host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),1,None,
        world::Configuration { envelope_records:true,interleaved_records:true,wrong_inverse_member:wrong,..Default::default() })
}
fn retire(host: &Host) -> WorthQueryApplicationCommitReceipt {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let outcome = host
        .request(&principal, &scope)
        .advanced()
        .mutate(member::retire::Intent)
        .without_source()
        .idempotency(&"recorded-member-retirement".to_owned())
        .execute_in_program(host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
        panic!("{outcome:?}")
    };
    receipt
}
#[test]
fn corrective_root_and_ordinary_member_create_and_final_edit_publish_once_with_authentic_replay() {
    let host = corrected_host(false);
    let retired = retire(&host);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let limits = WorthQueryCorrectionHistoryLimits::new(None, NonZeroUsize::MIN);
    let source = request.corrections().select(&retired, limits).unwrap();
    let mut edit = request
        .corrections()
        .reverse(&source)
        .idempotency("member-inverse-envelope")
        .limits(limits)
        .edit_envelope::<member::correction::UndoRetire, Program, ModelTree>(&host)
        .unwrap()
        .ordered_authored()
        .unwrap();
    let created = edit
        .push_stage::<ModelTree, _>(created::Intent {
            key: "additional-member".into(),
            previous: Some("m".into()),
            value: 4,
        })
        .unwrap();
    let final_edit = edit
        .push_stage::<ModelTree, _>(fixture::cap::Intent {
            key: "envelope-cap".into(),
            value: 8,
            key_derived_target: false,
        })
        .unwrap();
    assert_eq!((created.ordinal(), final_edit.ordinal()), (1, 2));
    let before = super::super::super::courts::native_head(&host).0;
    let (outcome, writes) = observe_maintenance_work(24, || {
        edit.execute_performed(ExecutionAllocationPolicy::SystemAllocation)
            .unwrap()
    });
    let Outcome::Committed(receipt) = outcome else {
        panic!("{outcome:?}")
    };
    assert_eq!(receipt.stage_count(), 3);
    assert_eq!(
        receipt.result(&created).unwrap().value,
        9,
        "the genuine root restored m=5 at D"
    );
    assert_eq!(writes.published_rows.len() + writes.omitted_publications, 1);
    assert_eq!(super::super::super::courts::native_head(&host).1, [before]);
    let members = request
        .query(query::members::Read("a".into()))
        .execute()
        .unwrap();
    assert!(members.rows()[0]
        .members
        .iter()
        .any(|m| m.key == "m" && m.value == 5));
    assert!(members.rows()[0]
        .members
        .iter()
        .any(|m| m.key == "additional-member" && m.value == 9));
    let head = super::super::super::courts::native_head(&host);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    for value in [8, 1001] {
        let mut retry = request
            .corrections()
            .reverse(&source)
            .idempotency("member-inverse-envelope")
            .limits(limits)
            .edit_envelope::<member::correction::UndoRetire, Program, ModelTree>(&host)
            .unwrap()
            .ordered_authored()
            .unwrap();
        retry
            .push_stage::<ModelTree, _>(created::Intent {
                key: "additional-member".into(),
                previous: Some("m".into()),
                value: 4,
            })
            .unwrap();
        retry
            .push_stage::<ModelTree, _>(fixture::cap::Intent {
                key: "envelope-cap".into(),
                value,
                key_derived_target: false,
            })
            .unwrap();
        let outcome = retry
            .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        if value == 8 {
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
        assert_eq!(super::super::super::courts::native_head(&host), head);
        assert_eq!(host.producer_contacts_on_this_thread_for_test(), contacts);
    }
}
#[test]
fn a_wrong_inverse_root_cannot_borrow_a_later_correct_member_and_owner_link() {
    let host = corrected_host(true);
    let retired = retire(&host);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let limits = WorthQueryCorrectionHistoryLimits::new(None, NonZeroUsize::MIN);
    let source = request.corrections().select(&retired, limits).unwrap();
    let mut edit = request
        .corrections()
        .reverse(&source)
        .idempotency("invalid-member-root")
        .limits(limits)
        .edit_envelope::<member::correction::UndoRetire, Program, ModelTree>(&host)
        .unwrap()
        .ordered_authored()
        .unwrap();
    edit.push_stage::<ModelTree, _>(created::Intent {
        key: "m".into(),
        previous: None,
        value: 5,
    })
    .unwrap();
    let before = super::super::super::courts::native_head(&host);
    let (outcome, writes) = observe_maintenance_work(24, || {
        edit.execute_performed(ExecutionAllocationPolicy::SystemAllocation)
            .unwrap()
    });
    let Outcome::Uncommitted(WorthQueryApplicationUncommitted::Denied(cause)) = outcome else {
        panic!("{outcome:?}")
    };
    assert!(
        cause.relation_preimage_failure().is_some(),
        "wrong phase/owner for the inverse refusal: {cause:?}"
    );
    assert_eq!(writes.published_rows.len() + writes.omitted_publications, 0);
    assert_eq!(super::super::super::courts::native_head(&host), before);
    assert!(!request
        .query(query::members::Read("a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .members
        .iter()
        .any(|m| m.key == "m" || m.key == "wrong-inverse-member"));
}
