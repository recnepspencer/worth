//! End-to-end Host courts for optional query parameters (`G7`) and authored
//! literal equality constants (`DX-2`), including pinned/live consistency and
//! bounded work metering.
use super::edit::{EditNotes, Edited};
use super::world::{funded_due, funded_due_with_budget, Host, Seed};
use super::*;
use worth_query_decl::facade::application_query::{
    ApplicationQueryParameterRef, ApplicationQueryParameterSet,
};
use worth_query_decl::facade::application_schema::{
    ApplicationScalarValueBinding, ApplicationValueDecodeUnavailable, ApplicationValueEncodeDenial,
    ApplicationValueIsNotIdentity, ApplicationValueSignedAggregateUnavailable,
    ApplicationValueValidationDenial,
};
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryAdmissionClock, WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestExt,
    WorthQueryApplicationRequestQueryDenial as Denial,
};
use worth_query_host::facade::primary_graph::WorthQueryApplicationOneShotDenialKind as Kind;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};

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

    // Chaining `.where_equal_value(Status, "open")` + `.where_equal(TaskKey, expected_key)`
    // on an exact-root query intersects both structured guards.
    assert_eq!(keys(world::exact_open_task(&host, 1, 1)), [1]);
    assert!(keys(world::exact_open_task(&host, 1, 2)).is_empty());
    assert!(keys(world::exact_open_task(&host, 3, 3)).is_empty());
}

fn wide_notes_board(width: usize) -> Vec<Seed> {
    (1..=16)
        .map(|key| {
            (
                key,
                1,
                "task",
                Some("n".repeat(width)),
                "open",
                1,
                1_000_000,
                Some(100),
            )
        })
        .collect()
}

fn cancellable() -> WorthQueryRequestScope {
    let source = WorthQueryCancellationSource::new();
    WorthQueryRequestScope::new(None, source.token(), WorthQueryAdmissionClock::system())
}

fn is_budget_exhaustion(denial: &Denial) -> bool {
    match denial {
        Denial::Execution(denial) => matches!(
            denial.kind(),
            Kind::ExecutionRequest(_)
                | Kind::WorkLimitExceeded
                | Kind::ResultBufferLimitExceeded
                | Kind::Projection(_)
        ),
        Denial::ExecutionRequest(_)
        | Denial::Admission(_)
        | Denial::ScopeResolution(_)
        | Denial::PrincipalResolution(_) => true,
        _ => false,
    }
}

fn minimal_due_budget(host: &Host, intent: &DueBy) -> u64 {
    let (mut low, mut high) = (1_u64, 1 << 24);
    let scope = cancellable();
    assert!(funded_due(host, &scope, high, intent.clone()).is_ok());
    while low < high {
        let middle = low + (high - low) / 2;
        match funded_due(host, &scope, middle, intent.clone()) {
            Ok(_) => high = middle,
            Err(denial) if is_budget_exhaustion(&denial) => low = middle + 1,
            Err(other) => panic!("unexpected denial while calibrating work threshold: {other:?}"),
        }
    }
    low
}

fn minimal_due_memory_budget(host: &Host, intent: &DueBy) -> u64 {
    let (mut low, mut high) = (1024_u64, 16 * 1024 * 1024);
    let scope = cancellable();
    assert!(funded_due_with_budget(host, &scope, high, 1 << 24, intent.clone()).is_ok());
    while low < high {
        let middle = low + (high - low) / 2;
        match funded_due_with_budget(host, &scope, middle, 1 << 24, intent.clone()) {
            Ok(_) => high = middle,
            Err(denial) if is_budget_exhaustion(&denial) => low = middle + 1,
            Err(other) => panic!("unexpected denial while calibrating memory threshold: {other:?}"),
        }
    }
    low
}

#[test]
fn where_filter_refuses_when_memory_budget_exhausts_after_parameter_admission() {
    let mut single = wide_notes_board(16);
    single.truncate(1);
    let host = install(&single);
    let absent = DueBy {
        project: 1,
        max_due_days: None,
        text: None,
    };
    let present = DueBy {
        project: 1,
        max_due_days: None,
        text: Some("m".repeat(60 * 1024)),
    };
    let absent_mem = minimal_due_memory_budget(&host, &absent);
    let present_mem = minimal_due_memory_budget(&host, &present);
    assert!(
        present_mem > 60 * 1024 && present_mem > absent_mem,
        "60 KiB optional parameter must raise peak memory above absent baseline: {absent_mem} vs {present_mem}"
    );
    let scope = cancellable();
    assert!(funded_due_with_budget(&host, &scope, present_mem, 1 << 24, present.clone()).is_ok());
    // One byte under the calibrated memory floor passes parameter admission
    // (which peaked earlier) and refuses specifically inside `where_filter`.
    match funded_due_with_budget(&host, &scope, present_mem - 1, 1 << 24, present) {
        Err(Denial::Execution(denial)) => {
            assert!(
                matches!(denial.kind(), Kind::ExecutionRequest(_)),
                "{denial:?}"
            );
            assert_eq!(denial.subject(), "where_filter");
        }
        other => panic!("finite memory exhaustion must refuse inside where_filter: {other:?}"),
    }
}

#[test]
fn optional_parameter_work_scales_when_present_and_short_circuits_when_absent() {
    let host = install(&wide_notes_board(2048));
    // Both `absent_intent` and `matching_large` return all 16 rows, holding
    // projection work identical while `Some(large)` pays parameter and scan work.
    let absent_intent = DueBy {
        project: 1,
        max_due_days: None,
        text: None,
    };
    let matching_large = DueBy {
        project: 1,
        max_due_days: None,
        text: Some("n".repeat(512)),
    };
    let none_budget = minimal_due_budget(&host, &absent_intent);
    let large_budget = minimal_due_budget(&host, &matching_large);
    assert!(
        large_budget > none_budget + 16 * 128,
        "Some(large text) must charge parameter and scanning work beyond short-circuited None: {none_budget} vs {large_budget}"
    );
    // With a non-matching optional needle, residual evaluation is the last
    // metered work so one unit short refuses inside `where_filter`.
    let non_matching = DueBy {
        project: 1,
        max_due_days: None,
        text: Some("absent".repeat(64)),
    };
    let non_matching_budget = minimal_due_budget(&host, &non_matching);
    let scope = cancellable();
    let (rows, _) = funded_due(&host, &scope, non_matching_budget, non_matching.clone())
        .expect("exact minimal budget completes");
    assert!(rows.is_empty());
    match funded_due(&host, &scope, non_matching_budget - 1, non_matching) {
        Err(Denial::Execution(denial)) => {
            assert!(
                matches!(denial.kind(), Kind::ExecutionRequest(_)),
                "{denial:?}"
            );
            assert_eq!(denial.subject(), "where_filter");
        }
        other => panic!("one unit short must refuse inside where_filter: {other:?}"),
    }
}

fn edit_notes(host: &Host, project: u64, task: u64, notes: &str, key: u64) {
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
