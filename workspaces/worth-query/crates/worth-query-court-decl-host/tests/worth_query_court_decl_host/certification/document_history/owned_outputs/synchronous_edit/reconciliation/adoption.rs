//! Actual changed-input/equal-output execution adopts, then replay is Current.
use super::super::failure;
use super::*;
use worth_relational::facade::inspection::observe_maintenance_work;

pub(super) fn raise_cap(
    host: &WorthQueryProgramApplicationRuntime<Schema, program::Lazy>,
    request: &family::Request<'_, '_, '_>,
) {
    // The legacy setup is fully Current. Change a genuinely consumed
    // owned cap, with min(2,5) == min(2,6), to require actual computation.
    let edit = request
        .advanced()
        .mutate(member_edit::Intent {
            key: "m".into(),
            value: 6,
        })
        .without_source()
        .idempotency(&"raise-reconciliation-cap".to_owned())
        .execute_in_program(
            host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(
        matches!(edit, WorthQueryApplicationMutationOutcome::Committed { .. }),
        "{edit:?}"
    );
}

pub(super) fn verify(
    host: &WorthQueryProgramApplicationRuntime<Schema, program::Lazy>,
    request: &family::Request<'_, '_, '_>,
    expected: snapshot::Snapshot,
) {
    let advance = |maximum_contacts: u64| {
        let started = host.producer_contacts_on_this_thread_for_test();
        let mut demand = request
            .advanced()
            .start_program_outputs::<program::Lazy, program::LazyTree>(
                host,
                Demand("a".into()),
                WorthQueryOutputDemandControls::default(),
            )
            .unwrap();
        for _ in 0..128 {
            let progress = demand.advance(host, request);
            let actual = host.producer_contacts_on_this_thread_for_test() - started;
            assert!(actual <= maximum_contacts,
                "fresh adopting support must not re-execute beyond this demand's contact bound: actual={actual} maximum={maximum_contacts}");
            assert!(progress.is_ok(),
                "executed adoption must install usable Ready support: contacts={actual} refusal={:?}", progress.as_ref().err());
            if let WorthQueryApplicationProgramOutputProgress::Settled(_) = progress.unwrap() {
                return demand;
            }
        }
        panic!("reconciliation output did not settle");
    };
    let roots = std::iter::once(
        request
            .query(query::model::Read("a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .native_id,
    )
    .chain(
        expected
            .rows
            .iter()
            .flat_map(|row| row.sources.iter().map(|source| source.native_id)),
    )
    .collect::<Vec<_>>();
    let before = roots
        .iter()
        .map(|root| host.producer_contacts_at_root_on_this_thread_for_test(*root))
        .collect::<Vec<_>>();
    let native_before = failure::checkpoint(host);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let (_, executed) = observe_maintenance_work(8, || advance(1));
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 1,
        "the adopting handler is an actual producer contact; commits={} per-root before={before:?} after={:?}",
        executed.published_rows.len() + executed.omitted_publications,
        roots.iter().map(|root| host.producer_contacts_at_root_on_this_thread_for_test(*root)).collect::<Vec<_>>()
    );
    assert_eq!(
        executed.published_rows.len() + executed.omitted_publications,
        0,
        "the identity-equal executed completion adopts its Native publication"
    );
    native_unchanged::assert_unchanged(host, &native_before);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let (_, current) = observe_maintenance_work(8, || advance(0));
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts,
        "fresh adopting support makes the next demand Current"
    );
    assert_eq!(
        current.published_rows.len() + current.omitted_publications,
        0
    );
    assert_eq!(
        request
            .query(snapshot::Read("a".into()))
            .execute()
            .unwrap()
            .rows()[0],
        expected,
        "adoption preserves complete named rows and identities"
    );
}

fn adoption(worker: bool) {
    courts::with_legacy_rows(worker, |host, request, expected| {
        raise_cap(host, request);
        verify(host, request, expected);
    });
}

#[test]
fn executed_no_effect_adopts_and_retains_current_support() {
    adoption(false);
}
#[test]
fn worker_only_executed_no_effect_adopts_and_retains_current_support() {
    adoption(true);
}
