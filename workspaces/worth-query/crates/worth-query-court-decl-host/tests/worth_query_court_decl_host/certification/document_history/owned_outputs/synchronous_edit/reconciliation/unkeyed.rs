//! Real orbit evidence needs funded full verification after executed adoption.
use super::*;
fn run(worker: bool) {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let host = world::install_configured::<program::Lazy>(
        limits,
        2,
        None,
        world::Configuration {
            reconciliation_peer_caps: true,
            ..world::Configuration::default()
        },
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let _original = courts::settle(&host, &request);
    let expected = request
        .query(snapshot::Read("a".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .clone();
    assert_eq!(expected.rows.len(), 3);
    // Establish this route's Preserve implementation before measuring adoption;
    // Initial→Preserve is an implementation change and correctly publishes.
    adoption::raise_cap(&host, &request);
    let _preserve = courts::settle(&host, &request);
    let changed = request
        .advanced()
        .mutate(member_edit::Intent {
            key: "m".into(),
            value: 7,
        })
        .without_source()
        .idempotency(&"raise-derived-cap-seven".to_owned())
        .execute_in_program(
            &host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(
        matches!(
            changed,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{changed:?}"
    );
    // min(requested2, orbit cap6) == min(requested2, orbit cap7).
    // The derived selection is actual consumed support, not a synthetic
    // unsupported tag. This executes once and then fully verifies that support.
    adoption::verify(&host, &request, expected);
    // The accepted output is now backed by real, unsupported orbit evidence.
    // Changing that consumed cap must be compared in full before reuse. A
    // verifier that merely declares the old evidence Current leaves two members.
    let changed = request
        .advanced()
        .mutate(cap_edit::Intent {
            key: "cap".into(),
            value: 1,
        })
        .without_source()
        .idempotency(&"lower-derived-cap-one".to_owned())
        .execute_in_program(
            &host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(
        matches!(
            changed,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{changed:?}"
    );
    let _reduced = courts::settle(&host, &request);
    let reduced = request.query(snapshot::Read("a".into())).execute().unwrap();
    assert_eq!(
        reduced.rows()[0]
            .rows
            .iter()
            .map(|row| row.row.key.as_str())
            .collect::<Vec<_>>(),
        vec!["reconciled:member:a:0", "reconciled:root:a"],
        "full verification must reject stale derived support before reuse"
    );
}
#[test]
fn derived_support_is_fully_verified_after_adoption() {
    run(false);
}
#[test]
fn worker_only_derived_support_is_fully_verified_after_adoption() {
    run(true);
}
