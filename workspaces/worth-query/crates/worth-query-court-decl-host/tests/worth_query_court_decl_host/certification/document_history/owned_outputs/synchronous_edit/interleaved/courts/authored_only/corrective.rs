//! Genuine correction roles and later actions remain in one rootless envelope.
use super::*;
use std::num::NonZeroUsize;

#[test]
fn plain_corrective_and_reapplication_roles_share_a_rootless_final_edit() {
    use super::super::super::super::super::program as base;
    let host: world::Host = world::install_configured(
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()), 1, None,
        world::Configuration { envelope_records: true, ..Default::default() },
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let retired = request
        .advanced()
        .mutate(member::retire::Intent)
        .without_source()
        .idempotency(&"rootless-retire".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed {
        receipt: retired, ..
    } = retired
    else {
        panic!("{retired:?}")
    };
    let limits = WorthQueryCorrectionHistoryLimits::new(None, NonZeroUsize::MIN);
    let source = request.corrections().select(&retired, limits).unwrap();
    let make = |value| {
        let mut edit = request.corrections().reverse(&source).idempotency("rootless-restore").limits(limits)
            .edit_envelope::<member::correction::UndoRetire, base::Program, ApplicationNoOutputGraph>(&host).unwrap().ordered_authored().unwrap();
        let prefix = edit.stage_token::<member::restore::Binding>(0).unwrap();
        assert!(edit.stage_token::<member::retire::Binding>(0).is_none());
        assert!(edit.stage_token::<member::restore::Binding>(2).is_none());
        let final_edit = edit
            .push_stage::<ApplicationNoOutputGraph, _>(cap(value))
            .unwrap();
        assert_eq!(
            edit.stage_token::<fixture::cap::Binding>(1)
                .unwrap()
                .ordinal(),
            final_edit.ordinal()
        );
        (edit, prefix, final_edit)
    };
    let before = super::super::super::super::courts::native_head(&host);
    let (edit, prefix, final_edit) = make(8);
    let (outcome, writes) = observe_maintenance_work(16, || {
        edit.execute_performed(ExecutionAllocationPolicy::SystemAllocation)
            .unwrap()
    });
    let Outcome::Committed(restored) = outcome else {
        panic!("{outcome:?}")
    };
    let _: &super::super::super::super::super::member_edit::Edited =
        restored.result(&prefix).unwrap();
    assert_eq!(
        restored.stage_receipt(&prefix).unwrap().commit_reference(),
        restored.receipt().commit_reference()
    );
    assert_eq!(
        (
            restored.stage_count(),
            final_edit.ordinal(),
            restored.result(&final_edit).unwrap().value
        ),
        (2, 1, 8)
    );
    assert_eq!(writes.published_rows.len() + writes.omitted_publications, 1);
    assert_eq!(
        super::super::super::super::courts::native_head(&host).1,
        [before.0]
    );
    assert!(request
        .query(query::members::Read("a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .members
        .iter()
        .any(|row| row.key == "m" && row.value == 5));
    let current = super::super::super::super::courts::native_head(&host);
    for value in [8, 1001] {
        let (retry, _, _) = make(value);
        let (outcome, writes) = observe_maintenance_work(8, || {
            retry
                .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
                .unwrap()
        });
        if value == 8 {
            let Outcome::RecoveredCommitted(recorded) = outcome else {
                panic!("{outcome:?}")
            };
            assert!(restored.receipt().matches_recorded_original(&recorded));
        } else {
            assert!(
                matches!(outcome, Outcome::IdempotencyIntentDrift),
                "{outcome:?}"
            );
        }
        assert_eq!(writes.published_rows.len() + writes.omitted_publications, 0);
        assert_eq!(
            super::super::super::super::courts::native_head(&host),
            current
        );
    }
    let page = request
        .corrections()
        .alternatives(&source, limits, NonZeroUsize::MIN)
        .unwrap();
    assert!(page.continuation().is_none());
    assert_eq!(page.entries().len(), 1);
    assert_eq!(
        page.entries()[0].performed_commit(),
        restored.receipt().commit_reference()
    );
    let alternative = page.select(&page.entries()[0], limits).unwrap();
    let mut reapply = request
        .corrections()
        .reapply(&alternative)
        .idempotency("rootless-reapply")
        .limits(limits)
        .edit_envelope::<member::correction::UndoRetire, base::Program, ApplicationNoOutputGraph>(
            &host,
        )
        .unwrap()
        .ordered_authored()
        .unwrap();
    let prefix = reapply.stage_token::<member::retire::Binding>(0).unwrap();
    let final_edit = reapply
        .push_stage::<ApplicationNoOutputGraph, _>(cap(9))
        .unwrap();
    let (outcome, writes) = observe_maintenance_work(16, || {
        reapply
            .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
            .unwrap()
    });
    let Outcome::Committed(reapplied) = outcome else {
        panic!("{outcome:?}")
    };
    let _: &super::super::super::super::super::member_edit::Edited =
        reapplied.result(&prefix).unwrap();
    assert_eq!(
        reapplied.stage_receipt(&prefix).unwrap().commit_reference(),
        reapplied.receipt().commit_reference()
    );
    assert_eq!(
        (
            reapplied.stage_count(),
            reapplied.result(&final_edit).unwrap().value
        ),
        (2, 9)
    );
    assert_eq!(writes.published_rows.len() + writes.omitted_publications, 1);
    assert_eq!(
        super::super::super::super::courts::native_head(&host).1,
        [current.0]
    );
    assert!(!request
        .query(query::members::Read("a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .members
        .iter()
        .any(|row| row.key == "m"));
}
