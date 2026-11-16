//! Conditional roots, genuine inverse custody and whole-envelope original replay.
use super::*;
use crate::certification::document_history::owned_outputs::member_edit::Edited;
use crate::certification::document_history::owned_outputs::synchronous_edit::courts::native_head;
use std::num::NonZeroUsize;
use worth_query_host::facade::{
    application_entry::WorthQueryEditEnvelopeOutcome as Outcome, runtime::ExecutionAllocationPolicy,
};
use worth_relational::facade::inspection::observe_maintenance_work;

fn cap(value: u64) -> fixture::cap::Intent {
    fixture::cap::Intent {
        key: "envelope-cap".into(),
        value,
        key_derived_target: false,
    }
}
#[test]
fn fixed_and_discovered_postures_have_distinct_full_intents() {
    let input =
        ApplicationEncodedInput::<member::retire::Input>::encode(member::retire::Intent).unwrap();
    type Fixed = ApplicationOutputGraph<RetireConnection, ApplicationOutputLeaf>;
    let fixed = ApplicationEditStageIntent::new::<Schema, member::retire::Binding, Fixed>(&input);
    let discovered =
        ApplicationEditStageIntent::new::<Schema, member::retire::Binding, RetireTree>(&input);
    let key = ApplicationEditEnvelopeKey::encode("discovered-mode", "same-key".to_owned()).unwrap();
    assert_ne!(
        key.intent(ApplicationEditEnvelopeOrder::OrderedAuthored, &[fixed])
            .unwrap(),
        key.intent(ApplicationEditEnvelopeOrder::OrderedAuthored, &[discovered])
            .unwrap(),
    );
}

#[test]
fn conditional_corrective_roots_and_final_actions_publish_once_and_replay_original() {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host: WorthQueryProgramApplicationRuntime<Schema, Program> = world::install_configured(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        1, None, world::Configuration { envelope_records: true, ..Default::default() });
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = native_head(&host);
    let mut omitted = request
        .edit_envelope()
        .idempotency("omit-conditional".to_owned())
        .runtime_stages(&host);
    omitted
        .push_stage::<ApplicationNoOutputGraph, _>(member::retire::Intent)
        .unwrap();
    assert!(omitted
        .resolve_idempotency(ExecutionAllocationPolicy::SystemAllocation)
        .is_err());
    assert_eq!(native_head(&host), before);
    let mut retire = request
        .edit_envelope()
        .idempotency("conditional-retire".to_owned())
        .runtime_stages(&host)
        .ordered_authored()
        .unwrap();
    retire
        .push_stage::<RetireTree, _>(member::retire::Intent)
        .unwrap();
    retire
        .push_stage::<ApplicationNoOutputGraph, _>(cap(7))
        .unwrap();
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let (outcome, writes) = observe_maintenance_work(24, || {
        retire
            .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
            .unwrap()
    });
    let Outcome::Committed(retired) = outcome else {
        panic!("{outcome:?}")
    };
    assert_eq!(retired.stage_count(), 2);
    assert_eq!(writes.published_rows.len() + writes.omitted_publications, 1);
    assert_eq!(native_head(&host).1, [before.0]);
    assert_eq!(host.producer_contacts_on_this_thread_for_test(), contacts);
    assert!(request
        .query(query::members::Read("a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .members
        .is_empty());
    let limits = WorthQueryCorrectionHistoryLimits::new(None, NonZeroUsize::MIN);
    let source = request
        .corrections()
        .select(retired.receipt(), limits)
        .unwrap();
    let make = |value, key: &str| {
        let mut edit = request
            .corrections()
            .reverse(&source)
            .idempotency(key)
            .limits(limits)
            .edit_envelope::<member::correction::UndoRetire, Program, RestoreTree>(&host)
            .unwrap()
            .ordered_authored()
            .unwrap();
        let prefix = edit.stage_token::<member::restore::Binding>(0).unwrap();
        let final_edit = edit
            .push_stage::<ApplicationNoOutputGraph, _>(cap(value))
            .unwrap();
        (edit, prefix, final_edit)
    };
    // Fail while this genuine original Retire is still eligible for reversal.
    let current = native_head(&host);
    let (failed, _, token) = make(1001, "conditional-failed");
    let (outcome, writes) = observe_maintenance_work(24, || {
        failed
            .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
            .unwrap()
    });
    let Outcome::DomainDenied(denial) = outcome else {
        panic!("{outcome:?}")
    };
    assert_eq!(denial.identity().ordinal(), 1);
    assert_eq!(
        denial.denial(&token).unwrap(),
        &super::super::super::Denial::ValueOutOfRange
    );
    let original_abort: WorthQueryApplicationUncommitted = denial.into_commit_outcome().unwrap();
    assert_eq!(writes.published_rows.len() + writes.omitted_publications, 0);
    assert_eq!(native_head(&host), current);
    drop(original_abort);
    assert!(request
        .query(query::members::Read("a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .members
        .is_empty());
    let (edit, prefix, final_edit) = make(8, "conditional-restore");
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let (outcome, writes) = observe_maintenance_work(24, || {
        edit.execute_performed(ExecutionAllocationPolicy::SystemAllocation)
            .unwrap()
    });
    let Outcome::Committed(restored) = outcome else {
        panic!("{outcome:?}")
    };
    assert_eq!(restored.stage_count(), 2);
    let _: &Edited = restored.result(&prefix).unwrap();
    assert_eq!(restored.result(&final_edit).unwrap().value, 8);
    assert_eq!(
        restored.stage_receipt(&prefix).unwrap().commit_reference(),
        restored.receipt().commit_reference()
    );
    assert_eq!(writes.published_rows.len() + writes.omitted_publications, 1);
    assert_eq!(native_head(&host).1, [current.0]);
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test() - contacts,
        1
    );
    let members = request
        .query(query::members::Read("a".into()))
        .execute()
        .unwrap();
    assert!(members.rows()[0]
        .members
        .iter()
        .any(|m| m.key == "m" && m.value == 5));
    assert_eq!(
        request
            .query(query::first_output::Read("first:a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        10
    );
    let page = request
        .corrections()
        .alternatives(&source, limits, NonZeroUsize::MIN)
        .unwrap();
    assert_eq!(page.entries().len(), 1);
    let alternative = page.select(&page.entries()[0], limits).unwrap();
    let current = native_head(&host);
    let mut reapply = request
        .corrections()
        .reapply(&alternative)
        .idempotency("conditional-reapply")
        .limits(limits)
        .edit_envelope::<member::correction::UndoRetire, Program, RetireTree>(&host)
        .unwrap()
        .ordered_authored()
        .unwrap();
    let prefix = reapply.stage_token::<member::retire::Binding>(0).unwrap();
    reapply
        .push_stage::<ApplicationNoOutputGraph, _>(cap(9))
        .unwrap();
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let (outcome, writes) = observe_maintenance_work(24, || {
        reapply
            .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
            .unwrap()
    });
    let Outcome::Committed(reapplied) = outcome else {
        panic!("{outcome:?}")
    };
    let _: &Edited = reapplied.result(&prefix).unwrap();
    assert_eq!(writes.published_rows.len() + writes.omitted_publications, 1);
    assert_eq!(native_head(&host).1, [current.0]);
    assert_eq!(host.producer_contacts_on_this_thread_for_test(), contacts);
    assert!(request
        .query(query::members::Read("a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .members
        .is_empty());
    // The original Restore answer is selected after the later Retire, without suppliers.
    let current = native_head(&host);
    for value in [8, 1001] {
        let (retry, _, _) = make(value, "conditional-restore");
        let (outcome, writes) = observe_maintenance_work(12, || {
            retry
                .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
                .unwrap()
        });
        if value == 8 {
            let Outcome::RecoveredCommitted(receipt) = outcome else {
                panic!("{outcome:?}")
            };
            assert!(restored.receipt().matches_recorded_original(&receipt));
        } else {
            assert!(
                matches!(outcome, Outcome::IdempotencyIntentDrift),
                "{outcome:?}"
            );
        }
        assert_eq!(writes.published_rows.len() + writes.omitted_publications, 0);
        assert_eq!(native_head(&host), current);
        assert_eq!(host.producer_contacts_on_this_thread_for_test(), contacts);
    }
}
