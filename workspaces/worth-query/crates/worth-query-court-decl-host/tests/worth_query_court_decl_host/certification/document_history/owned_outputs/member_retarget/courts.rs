//! The new source is absent from the retired member's genuine C catalog.
use super::*;
use std::num::NonZeroUsize;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use WorthQueryEditEnvelopeOutcome as Outcome;
type Host = WorthQueryProgramApplicationRuntime<Schema, program::Program>;

fn publish(
    host: &Host,
    request: &family::Request<'_, '_, '_>,
    input: Intent,
    key: &str,
) -> WorthQueryApplicationCommitReceipt {
    let mut edit = request
        .edit_envelope()
        .idempotency(key.to_owned())
        .runtime_stages(host)
        .ordered_authored()
        .unwrap();
    edit.push_stage::<program::Tree, _>(input).unwrap();
    let outcome = edit
        .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let Outcome::Committed(done) = outcome else {
        panic!("{outcome:?}")
    };
    done.receipt().clone()
}
fn input(request: &family::Request<'_, '_, '_>, key: &str) -> query::Row {
    request
        .query(query::input_member::Read(key.into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone()
}
fn first_value(request: &family::Request<'_, '_, '_>, key: &str) -> u64 {
    request
        .query(query::first_output::Read(format!("first:{key}")))
        .execute()
        .unwrap()
        .rows()[0]
        .value
}

#[test]
fn a_member_edit_keeps_its_new_source_route_through_reverse_and_reapply() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install_program_with_other_model::<program::Program>(host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()), 0, Some(7));
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    publish(
        &host,
        &request,
        Intent {
            owner: "a".into(),
            value: 1,
            retire: vec![],
            create: vec![Row {
                key: "old".into(),
                value: 11,
            }],
        },
        "initialize-input",
    );
    let old = input(&request, "old");
    let owner = request
        .query(query::model::Read("a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .native_id;
    assert_eq!(first_value(&request, "a"), 10);
    assert!(request
        .query(query::second_output::Read("second:b".into()))
        .execute()
        .is_err());
    let before = super::super::courts::native_head(&host).0;
    let receipt = publish(
        &host,
        &request,
        Intent {
            owner: "a".into(),
            value: 2,
            retire: vec!["old".into()],
            create: vec![Row {
                key: "new".into(),
                value: 22,
            }],
        },
        "retarget-input",
    );
    assert_eq!(
        super::super::courts::native_head(&host).1,
        [before],
        "the member edit and both producer scopes publish once"
    );
    assert_eq!(
        (
            first_value(&request, "a"),
            request
                .query(query::second_output::Read("second:b".into()))
                .execute()
                .unwrap()
                .rows()[0]
                .value
        ),
        (20, 1007)
    );
    assert!(request
        .query(query::input_member::Read("old".into()))
        .execute()
        .is_err());
    assert_eq!(input(&request, "new").value, 22);
    assert_eq!(
        request
            .query(query::model::Read("a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .native_id,
        owner
    );

    let limits = WorthQueryCorrectionHistoryLimits::new(None, NonZeroUsize::MIN);
    let selected = request.corrections().select(&receipt, limits).unwrap();
    let edit = request
        .corrections()
        .reverse(&selected)
        .idempotency("undo-retarget")
        .limits(limits)
        .edit_envelope::<correction::Revision, program::Program, program::Tree>(&host)
        .unwrap()
        .ordered_authored()
        .unwrap();
    let reversed = edit
        .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(reversed, Outcome::Committed(_)), "{reversed:?}");
    let restored = input(&request, "old");
    assert_eq!((restored.key, restored.value), (old.key, old.value));
    assert_ne!(
        restored.native_id, old.native_id,
        "restoration uses the authentic fresh member generation"
    );
    assert!(request
        .query(query::input_member::Read("new".into()))
        .execute()
        .is_err());
    assert_eq!(first_value(&request, "a"), 10);
    let selected = request.corrections().select(&receipt, limits).unwrap();
    let page = request
        .corrections()
        .alternatives(&selected, limits, NonZeroUsize::MIN)
        .unwrap();
    let alternative = page.select(&page.entries()[0], limits).unwrap();
    let edit = request
        .corrections()
        .reapply(&alternative)
        .idempotency("redo-retarget")
        .limits(limits)
        .edit_envelope::<correction::Revision, program::Program, program::Tree>(&host)
        .unwrap()
        .ordered_authored()
        .unwrap();
    let reapplied = edit
        .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(reapplied, Outcome::Committed(_)), "{reapplied:?}");
    assert_eq!(
        (
            first_value(&request, "a"),
            request
                .query(query::second_output::Read("second:b".into()))
                .execute()
                .unwrap()
                .rows()[0]
                .value
        ),
        (20, 1007)
    );
    assert!(request
        .query(query::input_member::Read("old".into()))
        .execute()
        .is_err());
    assert_eq!(input(&request, "new").value, 22);
}
