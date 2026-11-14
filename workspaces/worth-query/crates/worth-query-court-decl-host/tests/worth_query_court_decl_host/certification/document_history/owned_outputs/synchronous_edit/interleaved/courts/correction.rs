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
    let _execution = worth_query_host::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
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
        let ((outcome, contacts), writes) = observe_maintenance_work(24, || {
            let before = host.producer_contacts_on_this_thread_for_test();
            let outcome = retry
                .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
                .unwrap();
            (
                outcome,
                host.producer_contacts_on_this_thread_for_test() - before,
            )
        });
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
        assert_eq!(contacts, 0);
        assert_eq!(writes.published_rows.len() + writes.omitted_publications, 0);
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
    // The declared root identity is checked before a later stage can run.
    assert_eq!(
        cause.kind(),
        worth_query_host::facade::primary_graph::WorthQueryApplicationCommitDenialKind::CorrectionIdentityMismatch,
        "{cause:?}"
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

#[test]
fn a_real_retirement_and_restoration_each_keep_their_final_edit_and_owned_history() {
    use program::RetirementProgram;
    let host: WorthQueryProgramApplicationRuntime<Schema, RetirementProgram> =
        world::install_configured(host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()), 1, None,
            world::Configuration { envelope_records:true,interleaved_records:true,..Default::default() });
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    // Establish actual issued owned publications, not a fabricated source event.
    macro_rules! settle {
        ($demand:expr, $tree:ty) => {{
            let mut operation = request
                .demand($demand)
                .retain_program_outputs::<RetirementProgram, $tree>(&host)
                .unwrap_or_else(|cause| panic!("{:?}", cause.denial()));
            for count in 0..128 {
                assert!(count < 127, "original owned output did not settle");
                operation = match operation.advance(&request) {
                    WorthQueryProgramOutputOperationAdvance::Pending(next) => next,
                    WorthQueryProgramOutputOperationAdvance::Settled(done) => {
                        done.deliver_current(&request).unwrap();
                        done.close(&request)
                            .unwrap_or_else(|cause| panic!("{:?}", cause.denial()));
                        break;
                    }
                    WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => {
                        panic!("{denial:?}")
                    }
                    WorthQueryProgramOutputOperationAdvance::Terminal(_) => {
                        panic!("owned output terminated")
                    }
                };
            }
        }};
    }
    settle!(first::Demand("a".into()), ModelTree);
    settle!(child::Demand("m".into()), program::SealTree);
    assert_eq!(
        request
            .query(query::child_output::Read("child:m".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        500
    );
    let before = super::super::super::courts::native_head(&host).0;
    let mut edit = request
        .edit_envelope()
        .idempotency("retire-and-final-edit".to_owned())
        .runtime_stages(&host)
        .ordered_authored()
        .unwrap();
    let root = edit
        .push_stage::<ModelTree, _>(member::retire::Intent)
        .unwrap();
    let final_edit = edit
        .push_stage::<ModelTree, _>(fixture::cap::Intent {
            key: "envelope-cap".into(),
            value: 8,
            key_derived_target: false,
        })
        .unwrap();
    assert_eq!((root.ordinal(), final_edit.ordinal()), (0, 1));
    let (outcome, writes) = observe_maintenance_work(24, || {
        edit.execute_performed(ExecutionAllocationPolicy::SystemAllocation)
            .unwrap()
    });
    let Outcome::Committed(retired) = outcome else {
        panic!("{outcome:?}")
    };
    assert_eq!(retired.stage_count(), 2);
    assert_eq!(retired.result(&final_edit).unwrap().value, 8);
    assert_eq!(writes.published_rows.len() + writes.omitted_publications, 1);
    assert_eq!(super::super::super::courts::native_head(&host).1, [before]);
    assert!(request
        .query(query::member_public::Read("m".into()))
        .execute()
        .is_err());
    assert!(request
        .query(query::child_output::Read("child:m".into()))
        .execute()
        .is_err());
    let limits = WorthQueryCorrectionHistoryLimits::new(None, NonZeroUsize::MIN);
    let source = request
        .corrections()
        .select(retired.receipt(), limits)
        .unwrap();
    let mut inverse = request
        .corrections()
        .reverse(&source)
        .idempotency("restore-and-final-edit")
        .limits(limits)
        .edit_envelope::<member::correction::UndoRetire, RetirementProgram, ModelTree>(&host)
        .unwrap()
        .ordered_authored()
        .unwrap();
    let final_edit = inverse
        .push_stage::<ModelTree, _>(fixture::cap::Intent {
            key: "envelope-cap".into(),
            value: 9,
            key_derived_target: false,
        })
        .unwrap();
    let before = super::super::super::courts::native_head(&host).0;
    let (outcome, writes) = observe_maintenance_work(24, || {
        inverse
            .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
            .unwrap()
    });
    let Outcome::Committed(restored) = outcome else {
        panic!("{outcome:?}")
    };
    assert_eq!(restored.stage_count(), 2);
    assert_eq!(restored.result(&final_edit).unwrap().value, 9);
    assert_eq!(writes.published_rows.len() + writes.omitted_publications, 1);
    assert_eq!(super::super::super::courts::native_head(&host).1, [before]);
    assert_eq!(
        request
            .query(query::member_public::Read("m".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        5
    );
    // The actual restoration installer restored the original owned publication,
    // including its current delivery/close route; no fresh Birth is requested.
    assert_eq!(
        request
            .query(query::child_output::Read("child:m".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        500
    );
    settle!(child::Demand("m".into()), program::SealTree);
}
