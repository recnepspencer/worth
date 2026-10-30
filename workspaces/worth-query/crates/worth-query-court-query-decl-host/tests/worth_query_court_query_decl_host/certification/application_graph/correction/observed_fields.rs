//! Actual parent field observations guard the lawful inverse of a cloned child.
//! This is field support, not SourceAspectRevision end-to-end evidence.
use super::*;
use model::{
    authored_identity_model::{Created, CreatedOutputs, Retired, RetiredOutputs},
    observed_clone::CloneNumber,
    query::full::{Read, Row},
};
use worth_query_host::facade::{
    application_invariants::EntityId, runtime::ExecutionAllocationPolicy,
};

const PARENT: &str = "numeric";
const CHILD: &str = "observed-field-child";

fn clone(host: &host::Host) -> WorthQueryApplicationCommitReceipt {
    host::edit(host, Value::Primary(41), "clone-seed-primary");
    host::edit(host, Value::Secondary(3), "clone-seed-secondary");
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let key = "observed-fields-clone".to_owned();
    let outcome = host
        .runtime()
        .request(&principal, &scope)
        .advanced()
        .mutate(CloneNumber {
            key: PARENT.into(),
            name: CHILD.into(),
        })
        .without_source()
        .idempotency(&key)
        .execute_in_program(host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
        panic!("genuine observed clone: {outcome:?}");
    };
    receipt
}

fn read(host: &host::Host, key: &str) -> Row {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let result = host
        .runtime()
        .request(&principal, &scope)
        .query(Read { key: key.into() })
        .execute()
        .unwrap();
    let [row] = result.rows() else {
        panic!("one actual source/child row: {:?}", result.rows());
    };
    row.clone()
}

fn expected(entity_id: EntityId, key: &str, primary: u64, secondary: u64) -> Row {
    Row {
        entity_id,
        key: key.into(),
        primary,
        secondary,
        read_enabled: true,
        ceiling: 100,
        calculated: 0,
    }
}

fn can_reverse(host: &host::Host, source: &WorthQueryApplicationCommitReceipt) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let limits = WorthQueryCorrectionHistoryLimits::new(None, NonZeroUsize::MIN);
    let selected = request.corrections().select(source, limits).unwrap();
    let prepared = request
        .corrections()
        .reverse(&selected)
        .idempotency("observed-fields-eligible-after-inverse")
        .limits(limits)
        .prepare()
        .unwrap();
    assert!(
        matches!(
            &prepared,
            WorthQueryApplicationCorrectionPreparation::Prepared(_)
        ),
        "the exact parent inverse permits preparation of the original clone"
    );
    // Dropping the ordinary candidate publishes nothing and releases its custody.
}

#[test]
fn observed_field_support_blocks_noops_until_their_performed_inverse() {
    let host = host::install();
    let source = clone(&host);
    let _source_pin = pin_current(&host, &source);
    let child = source
        .outputs_of::<CreatedOutputs>()
        .unwrap()
        .entity::<Created>()
        .unwrap()
        .entity_id();
    let parent = read(&host, PARENT).entity_id;
    assert_ne!(parent, child);
    assert_ne!(child.generation_value(), 0);
    assert_eq!(read(&host, PARENT), expected(parent, PARENT, 41, 3));
    assert_eq!(read(&host, CHILD), expected(child, CHILD, 41, 3));

    let equal = host::edit(&host, Value::Primary(41), "observed-fields-equal-parent");
    let _equal_pin = pin_current(&host, &equal);
    assert!(!equal.is_same_authoritative_commit(&source));
    assert_eq!(read(&host, PARENT), expected(parent, PARENT, 41, 3));
    assert_eq!(read(&host, CHILD), expected(child, CHILD, 41, 3));
    diverged(&host, &source);

    let undone = reverse(&host, &equal, "observed-fields-reverse-equal");
    let _undone_pin = pin_current(&host, &undone);
    assert_eq!(
        undone
            .aftermath_causality()
            .unwrap()
            .recorded_source()
            .relational(),
        equal.commit_reference()
    );
    assert_eq!(read(&host, PARENT), expected(parent, PARENT, 41, 3));
    can_reverse(&host, &source);
    let _still_undone_pin = pin_current(&host, &undone);

    let changed = host::edit(&host, Value::Primary(53), "observed-fields-change-parent");
    let _changed_pin = pin_current(&host, &changed);
    assert_eq!(read(&host, PARENT), expected(parent, PARENT, 53, 3));
    assert_eq!(read(&host, CHILD), expected(child, CHILD, 41, 3));
    diverged(&host, &source);
    let restored = reverse(&host, &changed, "observed-fields-restore-parent");
    let _restored_pin = pin_current(&host, &restored);
    assert_eq!(
        restored
            .aftermath_causality()
            .unwrap()
            .recorded_source()
            .relational(),
        changed.commit_reference()
    );
    assert_eq!(read(&host, PARENT), expected(parent, PARENT, 41, 3));
    let retired = reverse(&host, &source, "observed-fields-reverse-clone");
    assert_eq!(
        retired
            .aftermath_causality()
            .unwrap()
            .recorded_source()
            .relational(),
        source.commit_reference()
    );
    assert_eq!(
        retired
            .outputs_of::<RetiredOutputs>()
            .unwrap()
            .entity::<Retired>()
            .unwrap()
            .entity_id(),
        child
    );
    assert_eq!(read(&host, PARENT), expected(parent, PARENT, 41, 3));
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    match host.runtime().request(&principal, &scope).query(Read { key: CHILD.into() }).execute() {
        Err(WorthQueryApplicationRequestQueryDenial::ScopeResolution(cause)) => assert_eq!(cause.kind(), worth_query_host::facade::primary_graph::WorthQueryEntityResolutionDenialKind::Refused(worth_query_host::facade::primary_graph::WorthQueryEntityResolutionRefusalKind::UnknownEntity)),
        Err(other) => panic!("child read failed for another cause: {other:?}"),
        Ok(_) => panic!("the exact created child remained live after its reversal"),
    }
}

#[test]
fn observed_field_support_refuses_sibling_content_and_independent_write_back() {
    let host = host::install();
    let source = clone(&host);
    let _source_pin = pin_current(&host, &source);
    let parent = read(&host, PARENT).entity_id;
    let child = source
        .outputs_of::<CreatedOutputs>()
        .unwrap()
        .entity::<Created>()
        .unwrap()
        .entity_id();
    assert_eq!(read(&host, PARENT), expected(parent, PARENT, 41, 3));
    assert_eq!(read(&host, CHILD), expected(child, CHILD, 41, 3));
    // Primary's performed cancellation is covered above. Here an independent
    // secondary edit and write-back must each block the same original source.
    host::edit(&host, Value::Secondary(19), "observed-fields-secondary");
    assert_eq!(read(&host, PARENT), expected(parent, PARENT, 41, 19));
    assert_eq!(read(&host, CHILD), expected(child, CHILD, 41, 3));
    diverged(&host, &source);
    host::edit(
        &host,
        Value::Secondary(3),
        "observed-fields-independent-write-back",
    );
    assert_eq!(read(&host, PARENT), expected(parent, PARENT, 41, 3));
    assert_eq!(read(&host, CHILD), expected(child, CHILD, 41, 3));
    diverged(&host, &source);
}
