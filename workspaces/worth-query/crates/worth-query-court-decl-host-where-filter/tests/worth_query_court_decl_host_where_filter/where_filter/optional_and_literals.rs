//! End-to-end Host courts for optional query parameters (`G7`) and authored
//! literal equality constants (`DX-2`), including pinned/live consistency and
//! bounded work/memory metering.
use super::edit::{EditNotes, Edited};
use super::*;
use worth_query_decl::facade::application_query::{
    ApplicationQueryParameterRef, ApplicationQueryParameterSet,
};
use worth_query_decl::facade::application_schema::{
    ApplicationScalarValueBinding, ApplicationValueDecodeUnavailable, ApplicationValueEncodeDenial,
    ApplicationValueIsNotIdentity, ApplicationValueSignedAggregateUnavailable,
    ApplicationValueValidationDenial,
};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestExt,
};
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};

#[path = "optional_and_literals/metering.rs"]
mod metering;
#[path = "optional_and_literals/roster_width.rs"]
mod roster_width;

#[test]
fn open_inbox_combines_literal_status_equality_and_two_optional_filters() {
    let host = install(&board());
    let inbox = |max_due_days, text: Option<&str>| {
        keys(world::open_inbox(
            &host,
            OpenInbox {
                project: 1,
                max_due_days,
                text: text.map(str::to_owned),
            },
            None,
        ))
    };
    // Project 1 open tasks: 1(due=100, notes FLANGE), 2(due=None, title flange),
    // 4(due=200), 5(due=150, notes flange-adjacent). Task 3 is "done" and
    // excluded by `.where_equal_value(Status, "open")` with zero status param.
    assert_eq!(inbox(None, None), [1, 2, 4, 5]);
    assert_eq!(inbox(Some(150), None), [1, 5]);
    assert_eq!(inbox(None, Some("flange")), [1, 2, 5]);
    assert_eq!(inbox(Some(100), Some("flange")), [1]);
    assert!(inbox(Some(50), None).is_empty());
}

#[test]
fn literal_filter_evaluates_authored_string_and_date_constants_with_zero_parameters() {
    let host = install(&board());
    // status == "open" && status != "done" && (!is_some(due) || unwrap(due) <= 150)
    // Keeps open tasks 1(100), 2(absent due), 5(150); excludes 3(done), 4(200).
    assert_eq!(
        keys(world::literal_filter(
            &host,
            LiteralFilter { project: 1 },
            None
        )),
        [1, 2, 5]
    );
    assert_eq!(
        keys(world::literal_filter(
            &host,
            LiteralFilter { project: 2 },
            None
        )),
        [6]
    );
}

#[test]
fn exact_root_where_equal_value_matches_only_the_authored_constant() {
    let host = install(&board());
    let matched = world::exact_project(&host, 1).expect("project 1 matches literal 1");
    assert_eq!(matched.len(), 1);
    assert_eq!(matched[0].key, 1);
    let filtered = world::exact_project(&host, 2).expect("project 2 is filtered out");
    assert!(filtered.is_empty());

    // Chaining `.where_equal_value(Status, "open")` + `.where_equal(Rank, expected_rank)`
    // on an exact-root query intersects both structured guards.
    assert_eq!(keys(world::exact_open_task(&host, 1, 5)), [1]);
    assert!(keys(world::exact_open_task(&host, 1, 9)).is_empty());
    assert!(keys(world::exact_open_task(&host, 3, 7)).is_empty());
}

fn edit_notes(host: &world::Host, project: u64, task: u64, notes: &str, key: u64) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let outcome = host
        .runtime()
        .request(&principal, &scope)
        .advanced()
        .mutate(EditNotes {
            project,
            task,
            notes: notes.to_owned(),
        })
        .without_source()
        .idempotency(&key)
        .execute_in_program(host, ExecutionAllocationPolicy::SystemAllocation)
        .expect("notes edit commits");
    assert!(matches!(
        outcome,
        WorthQueryApplicationMutationOutcome::<_, Edited>::Committed { .. }
    ));
}

#[test]
fn pinned_read_remains_consistent_across_edits_for_literal_and_optional_query() {
    let host = install(&board());
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let pinned_basis = host
        .runtime()
        .request(&principal, &scope)
        .retain_current_read()
        .expect("retaining current basis succeeds");
    let before = host
        .runtime()
        .request(&principal, &scope)
        .at(&pinned_basis)
        .query(OpenInbox {
            project: 1,
            max_due_days: Some(200),
            text: Some("bracket".into()),
        })
        .execute()
        .expect("initial inbox read succeeds");
    assert_eq!(
        before.rows().iter().map(|row| row.key).collect::<Vec<_>>(),
        [5]
    );

    // Editing excluded open task 4's notes makes it match on current head,
    // while a pinned read on the prior basis continues observing [5].
    edit_notes(&host, 1, 4, "spare bracket ready", 1);
    let current = keys(world::open_inbox(
        &host,
        OpenInbox {
            project: 1,
            max_due_days: Some(200),
            text: Some("bracket".into()),
        },
        None,
    ));
    assert_eq!(current, [4, 5]);
    let pinned = host
        .runtime()
        .request(&principal, &scope)
        .at(&pinned_basis)
        .query(OpenInbox {
            project: 1,
            max_due_days: Some(200),
            text: Some("bracket".into()),
        })
        .execute()
        .expect("pinned inbox read succeeds");
    assert_eq!(
        pinned.rows().iter().map(|row| row.key).collect::<Vec<_>>(),
        [5]
    );

    // Chained `.where_equal_value(Status, "open")` + `.where_equal(Rank, expected_rank)`
    // on an exact-root query also admits and executes through retained/pinned commits
    // across subsequent writes.
    let pinned_exact = |task, expected_rank| {
        host.runtime()
            .request(&principal, &scope)
            .at(&pinned_basis)
            .query(queries::ExactOpenTask {
                task,
                expected_rank,
            })
            .execute()
            .expect("retained multi-predicate exact-root read succeeds")
            .rows()
            .iter()
            .map(|row| row.key)
            .collect::<Vec<_>>()
    };
    assert_eq!(pinned_exact(1, 5), [1]);
    assert!(pinned_exact(1, 9).is_empty());
    assert!(pinned_exact(3, 7).is_empty());
    edit_notes(&host, 1, 1, "post-pin update", 2);
    assert_eq!(pinned_exact(1, 5), [1]);
    assert_eq!(keys(world::exact_open_task(&host, 1, 5)), [1]);
}

#[test]
fn bind_optional_refuses_present_codec_encoding_null() {
    struct NullEncodingBinding;
    impl ApplicationScalarValueBinding for NullEncodingBinding {
        type Value = ();
        type Unit = ();
        type Decode = ApplicationValueDecodeUnavailable;
        type Identity = ApplicationValueIsNotIdentity;
        type SignedAggregate = ApplicationValueSignedAggregateUnavailable;
        const IDENTITY_NAME: &'static str = "where-filter.null-codec.v1";
        const SCALAR_FAMILY: worth_foundational::facade::ScalarAspectType =
            worth_foundational::facade::ScalarAspectType::String;
        fn validate(_: &Self::Value) -> Result<(), ApplicationValueValidationDenial> {
            Ok(())
        }
        fn encode(
            _: &Self::Value,
        ) -> Result<worth_foundational::facade::AspectValue, ApplicationValueEncodeDenial> {
            Ok(worth_foundational::facade::AspectValue::Null)
        }
    }
    struct DummyParam;
    let required_ref = ApplicationQueryParameterRef::<
        queries::DueQuery,
        DummyParam,
        NullEncodingBinding,
    >::from_query_identifier("null_param");
    let required_denial = ApplicationQueryParameterSet::new()
        .bind(required_ref, ())
        .expect_err("required parameter encoding Null must refuse");
    assert!(matches!(
        required_denial,
        ApplicationValueEncodeDenial::CodecRejected { .. }
    ));

    let reference = ApplicationQueryParameterRef::<
        queries::DueQuery,
        DummyParam,
        NullEncodingBinding,
        true,
    >::optional("null_param");
    let denial = ApplicationQueryParameterSet::new()
        .bind_optional(reference, Some(()))
        .expect_err("present optional parameter encoding Null must refuse");
    assert!(matches!(
        denial,
        ApplicationValueEncodeDenial::CodecRejected { .. }
    ));
}
