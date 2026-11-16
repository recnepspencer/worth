//! Healthy selection reads currentness evidence without disclosing producer sources.
use super::*;

#[test]
fn a_member_edit_preserves_its_required_writer_and_verifies_an_unaffected_source() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    use member_retarget::{courts::publish, operation::Intent, program::Program, records::Row};
    let host = world::install_program_with_other_model::<Program>(host_limits(
        worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system(),
    ), 1, Some(7));
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    publish(
        &host,
        &request,
        Intent {
            owner: "a".into(),
            value: Some(2),
            retire: vec![],
            create: vec![Row {
                key: "old".into(),
                value: 11,
            }],
        },
        "initialize-current-input",
    );
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let first = request
        .query(query::first_output::Read("first:a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone();
    let second = request
        .query(query::second_output::Read("second:b".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone();
    publish(
        &host,
        &request,
        Intent {
            owner: "a".into(),
            value: None,
            retire: vec!["old".into()],
            create: vec![Row {
                key: "new".into(),
                value: 22,
            }],
        },
        "member-only-edit",
    );
    assert_eq!(host.producer_contacts_on_this_thread_for_test(), contacts + 1,
        "the member-linked writer publishes; the unaffected source verifies without a producer call");
    assert_eq!(
        request
            .query(query::first_output::Read("first:a".into()))
            .execute()
            .unwrap()
            .rows()[0],
        first
    );
    assert_eq!(
        request
            .query(query::second_output::Read("second:b".into()))
            .execute()
            .unwrap()
            .rows()[0],
        second
    );
    assert!(request
        .query(query::input_member::Read("old".into()))
        .execute()
        .is_err());
    assert_eq!(
        request
            .query(query::input_member::Read("new".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        22
    );
}
fn exercise(host: &world::Host, configured: bool) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let courts::Edited {
        document,
        before,
        selection,
        ..
    } = courts::edited(host, &request, false);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let projection = source_projection::Observation::begin();
    let selected = if configured {
        let document = courts::configure_all_trees(host, document);
        let selected = courts::checkout(&document, &request, before, selection, 0x00f0_5706)
            .unwrap_or_else(|cause| panic!("{cause:?}"));
        host.with_host_advancement(|phase| document.close(&phase).unwrap())
            .unwrap();
        selected
    } else {
        let selected = courts::checkout(&document, &request, before, selection, 0x00f0_5707)
            .unwrap_or_else(|cause| panic!("{cause:?}"));
        host.with_host_advancement(|phase| document.close(&phase).unwrap())
            .unwrap();
        selected
    };
    assert_eq!(selected.step().id(), before);
    assert_eq!(host.producer_contacts_on_this_thread_for_test(), contacts);
    assert_eq!(
        projection.reads(),
        0,
        "healthy selection must not project a producer source"
    );
}
#[test]
fn healthy_selection_does_not_open_configured_original_roots() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    exercise(&world::install(), true);
}
#[test]
fn healthy_selection_needs_no_fallback_driver() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    exercise(&world::install(), false);
}
#[test]
fn worker_only_healthy_selection_does_not_open_original_roots() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    exercise(&world::install_worker_only(), true);
}
#[test]
fn worker_only_healthy_selection_needs_no_fallback_driver() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    exercise(&world::install_worker_only(), false);
}

fn current_source_permission_is_not_needed_for_healthy_verification(host: &world::Host) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    restricted_owned::settle(host, &request, "m");
    let document = world::attach(host);
    let before = document.current_step(&request).unwrap().id();
    let original = host
        .request(&principal, &scope)
        .on_branch(
            document
                .selection(&request)
                .unwrap()
                .physical_endpoint()
                .product_branch(),
        )
        .query(query::child_output::Read("restricted-owned:m".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone();
    let document = host
        .with_host_advancement(|phase| {
            document
                .with_outputs::<program::Program, program::RestrictedOwnedTree>(
                    &phase,
                    &request,
                    host,
                    restricted_owned::Demand("m".into()),
                    WorthQueryOutputDemandControls::default(),
                )
                .unwrap_or_else(|(_, cause)| panic!("{cause:?}"))
        })
        .unwrap();
    let mut edit = document
        .edit(edit::Intent {
            key: "a".into(),
            value: 2,
        })
        .idempotency_owned(&request, "revoke-before-healthy-selection".to_owned())
        .unwrap()
        .retain_in_program::<program::Program, program::EditAction>(host)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    loop {
        edit = match edit.advance(&request) {
            WorthQueryHistoryEditAdvance::Progress(next) => next,
            WorthQueryHistoryEditAdvance::Ready(next) => {
                edit = next;
                break;
            }
            WorthQueryHistoryEditAdvance::Refused { denial, .. } => panic!("{denial:?}"),
        };
    }
    let WorthQueryHistoryEditFinish { document, outcome } = edit
        .finish(&request)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    assert!(matches!(
        outcome,
        WorthQueryHistoryEditOutcome::Recorded { .. }
    ));
    drop(outcome);
    let current = document.selection(&request).unwrap();
    let live = host
        .request(&principal, &scope)
        .on_branch(current.physical_endpoint().product_branch())
        .query(query::member_public::Read("m".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone();
    assert_eq!(live.value, 5, "the source is still live at C");
    let denial = host
        .request(&principal, &scope)
        .on_branch(current.physical_endpoint().product_branch())
        .query(query::member::Read("m".into()))
        .execute()
        .err()
        .expect("C must deny the producer-source query");
    assert!(
        matches!(denial, WorthQueryApplicationRequestQueryDenial::Admission(ref cause)
        if matches!(cause.kind(), WorthQueryApplicationQueryAdmissionDenialKind::Authorization(_))),
        "{denial:?}"
    );
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let selected = courts::checkout(&document, &request, before, current, 0x00f0_5708)
        .unwrap_or_else(|cause| {
            panic!("healthy verification must not admit a source query: {cause:?}")
        });
    let output = host
        .request(&principal, &scope)
        .on_branch(selected.physical_endpoint().product_branch())
        .query(query::child_output::Read("restricted-owned:m".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone();
    assert_eq!(output, original);
    assert_eq!(host.producer_contacts_on_this_thread_for_test(), contacts);
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}
#[test]
fn healthy_verification_does_not_borrow_restored_source_grants() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    current_source_permission_is_not_needed_for_healthy_verification(&world::install());
}
#[test]
fn worker_only_healthy_verification_needs_no_source_admission() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    current_source_permission_is_not_needed_for_healthy_verification(&world::install_worker_only());
}

// Whole-source retirement distinguishes a current read from an extra writer.
fn exercise_retirement_parent(current: bool) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    use super::authored_member as member;
    use super::synchronous_edit::interleaved::{ModelTree, Program};
    use worth_query_host::facade::{
        application_entry::WorthQueryEditEnvelopeOutcome as Outcome,
        runtime::ExecutionAllocationPolicy,
    };
    use worth_relational::facade::{
        inspection::observe_maintenance_work, publication::AuthoredRetirementRefusal,
    };
    let host = world::install_configured::<Program>(host_limits(
        worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()), 1, None,
        world::Configuration { envelope_records:true, interleaved_records:true, ..Default::default() });
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut original = None;
    if current {
        let mut demand = request
            .demand(first::Demand("a".into()))
            .start_in_program::<Program, ModelTree>(&host)
            .unwrap();
        loop {
            match demand.settle(&request).unwrap() {
                WorthQueryApplicationOutputDemandProgress::Settled(output) => {
                    drop(output);
                    break;
                }
                WorthQueryApplicationOutputDemandProgress::Progressed
                | WorthQueryApplicationOutputDemandProgress::Pending => {}
            }
        }
        demand.close();
        original = Some(
            request
                .query(query::first_output::Read("first:a".into()))
                .execute()
                .unwrap()
                .rows()[0]
                .clone(),
        );
    }
    let before = super::synchronous_edit::native_head(&host);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let mut edit = request
        .edit_envelope()
        .idempotency(format!("retire-with-parent-{current}"))
        .runtime_stages(&host)
        .ordered_authored()
        .unwrap();
    edit.push_stage::<ModelTree, _>(member::retire::Intent)
        .unwrap();
    let (outcome, work) = observe_maintenance_work(8, || {
        edit.execute_performed(ExecutionAllocationPolicy::SystemAllocation)
            .unwrap()
    });
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts,
        "a retirement may verify this parent, never run an unlisted producer"
    );
    if current {
        assert!(matches!(outcome, Outcome::Committed(_)), "{outcome:?}");
        assert_eq!(work.published_rows.len() + work.omitted_publications, 1);
        assert!(request
            .query(query::member_public::Read("m".into()))
            .execute()
            .is_err());
        let row = request
            .query(query::first_output::Read("first:a".into()))
            .execute()
            .unwrap();
        let original = original.unwrap();
        assert_eq!(row.rows()[0].native_id, original.native_id);
        assert_eq!((row.rows()[0].value, original.value), (10, 10));
    } else {
        let Outcome::Uncommitted(WorthQueryApplicationUncommitted::Denied(denial)) = outcome else {
            panic!("{outcome:?}");
        };
        assert_eq!(
            denial
                .authored_retirement_error()
                .unwrap()
                .authored_retirement_refusal(),
            Some(AuthoredRetirementRefusal::ExtraCoupledOwner)
        );
        assert_eq!(work.published_rows.len() + work.omitted_publications, 0);
        assert_eq!(super::synchronous_edit::native_head(&host), before);
        assert_eq!(
            request
                .query(query::member_public::Read("m".into()))
                .execute()
                .unwrap()
                .rows()[0]
                .key,
            "m"
        );
    }
}
#[test]
fn retirement_reads_a_current_parent_outside_its_writer_catalog() {
    exercise_retirement_parent(true);
}
#[test]
fn retirement_cannot_create_a_parent_outside_its_writer_catalog() {
    exercise_retirement_parent(false);
}
