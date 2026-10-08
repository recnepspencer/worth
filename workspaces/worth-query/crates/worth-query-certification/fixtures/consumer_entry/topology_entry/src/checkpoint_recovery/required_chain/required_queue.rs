//! Production advance consumes the queued required work it proves current, so
//! one advance by any required caller progresses the whole dirty set.

use super::*;
use worth_query_host::facade::application_entry::WorthQueryApplicationPerformedMutationOutcome;
use worth_query_host::facade::application_installation::WorthQueryCheckpointCapturePolicy as CapturePolicy;
use worth_query_host::facade::primary_graph::inexact_native_deliveries_on_this_thread_for_test as inexact_deliveries;

macro_rules! settle {
    ($demand:expr, $request:expr) => {
        (0..256)
            .find_map(|_| match $demand.advance(&$request).unwrap() {
                WorthQueryApplicationOutputDemandProgress::Pending => None,
                WorthQueryApplicationOutputDemandProgress::Settled(settled) => Some(settled),
            })
            .expect("the required demand initially settles")
    };
}

/// One advance must settle: the queued dirty set is consumed inside it.
macro_rules! settled_in_one_advance {
    ($demand:expr, $request:expr, $what:literal) => {
        match $demand.advance(&$request).unwrap() {
            WorthQueryApplicationOutputDemandProgress::Settled(settled) => settled,
            WorthQueryApplicationOutputDemandProgress::Pending => {
                panic!(concat!($what, " needs another advance"))
            }
        }
    };
}

/// Change the root producer's declared input.
macro_rules! change_root_input {
    ($request:expr, $application:expr, $y:expr, $idempotency:expr) => {{
        let selected = $request
            .query(PlanarRead {
                body_key: "anchor-a".to_owned(),
            })
            .execute()
            .unwrap();
        let outcome = $request
            .mutate(PlanarSourceAdjustment {
                scope_key: "anchor-a".to_owned(),
                replacement_y: length($y),
            })
            .expect_source(selected.observed_sources()[0].clone())
            .idempotency(&$idempotency)
            .execute_performed::<program::ChainProgram, program::ChainRoot>(
                &$application,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        assert!(
            matches!(
                outcome,
                WorthQueryApplicationPerformedMutationOutcome::Performed(_)
            ),
            "the root input change commits"
        );
    }};
}

/// Writes the Y of `$key`, and answers whether the index had room for it.
macro_rules! writes_y {
    ($request:expr, $application:expr, $key:expr, $y:expr, $idempotency:expr) => {{
        let selected = $request
            .query(PlanarRead {
                body_key: $key.to_owned(),
            })
            .execute()
            .unwrap();
        let changed = $request
            .mutate(PlanarSourceAdjustment {
                scope_key: $key.to_owned(),
                replacement_y: length($y),
            })
            .expect_source(selected.observed_sources()[0].clone())
            .idempotency(&$idempotency)
            .execute_performed::<program::ChainProgram, program::ChainRoot>(
                &$application,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            );
        matches!(
            changed,
            Ok(WorthQueryApplicationPerformedMutationOutcome::Performed(_))
        )
    }};
}

fn chain_application() -> application_installation::WorthQueryProgramApplicationRuntime<
    CheckpointSchema,
    program::ChainProgram,
> {
    let profile =
        worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile::standard();
    chain_application_with_marking_work(
        None,
        u64::try_from(profile.limits().source_currentness_work()).unwrap(),
    )
}

fn chain_application_with_marking_work(
    checkpoint: Option<application_installation::WorthQueryApplicationCheckpoint>,
    maximum_marking_work: u64,
) -> application_installation::WorthQueryProgramApplicationRuntime<
    CheckpointSchema,
    program::ChainProgram,
> {
    let profile =
        worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile::standard();
    support::install_program_with_seed::<program::ChainProgram>(
        checkpoint,
        profile,
        4_096,
        128 * 1_024 * 1_024,
        maximum_marking_work,
        source_world::seed,
    )
}

/// A, B and C form the dirty chain; D is required but reads none of it.
macro_rules! chain_with_unrelated {
    ($application:expr, $request:expr) => {{
        let mut a = $request
            .demand(PlanarOutputDemand::new("anchor-a"))
            .start_in_program::<program::ChainProgram, program::ChainRoot>(&$application)
            .unwrap();
        let mut b = $request
            .demand(ChainDemand("anchor-b".to_owned()))
            .start_dependent_in_program::<program::ChainProgram, program::ChainConnection>(
                &$application,
            )
            .unwrap();
        let mut c = $request
            .demand(ChainDemand("anchor-c".to_owned()))
            .start_dependent_in_program::<program::ChainProgram, program::ChainConnection>(
                &$application,
            )
            .unwrap();
        let mut d = $request
            .demand(PlanarOutputDemand::new("anchor-source-b"))
            .start_in_program::<program::ChainProgram, program::ChainRoot>(&$application)
            .unwrap();
        settle!(a, $request);
        settle!(b, $request);
        settle!(c, $request);
        settle!(d, $request);
        (a, b, c, d)
    }};
}

#[test]
fn an_unrelated_required_advance_progresses_the_dirty_required_set() {
    let _guard = checkpoint_recovery_test_guard();
    run_unrelated_required_advance(chain_application());
}

#[test]
fn restored_required_demands_reenter_source_query_under_small_marking_work() {
    let _guard = checkpoint_recovery_test_guard();
    let checkpoint = ready_chain_checkpoint();
    // Keep the ordinary Query profile and reduce only invalidation marking.
    // Retained source re-entry must use Query's own admitted preparation
    // allowance, not this independent marking ceiling.
    run_unrelated_required_advance(chain_application_with_marking_work(
        Some(checkpoint),
        65_536,
    ));
}

fn ready_chain_checkpoint() -> application_installation::WorthQueryApplicationCheckpoint {
    let application = chain_application();
    {
        let (scope, principal) = authenticate(&application);
        let request = application.request(&principal, &scope);
        let _demands = chain_with_unrelated!(application, request);
    }
    application
        .capture_application_checkpoint(CapturePolicy::SystemAllocation)
        .expect("the settled required chain is checkpointable")
}

fn run_unrelated_required_advance(
    application: application_installation::WorthQueryProgramApplicationRuntime<
        CheckpointSchema,
        program::ChainProgram,
    >,
) {
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let (mut a, mut b, mut c, mut d) = chain_with_unrelated!(application, request);
    let inexact = inexact_deliveries();
    change_root_input!(request, application, 2, 0x9176_3200_u64);

    let before_unrelated = query_entries();
    settled_in_one_advance!(d, request, "the unrelated required demand");
    assert!(
        query_entries() > before_unrelated,
        "the unrelated advance re-evidences the queued dirty chain"
    );

    let mut fresh_root = request
        .demand(PlanarOutputDemand::new("anchor-a"))
        .start_in_program::<program::ChainProgram, program::ChainRoot>(&application)
        .unwrap();
    let basis = request.retain_read().unwrap();
    let before_chain = query_entries();
    let fresh = settled_in_one_advance!(fresh_root, request, "the refreshed root");
    // The open root demand follows the refresh the unrelated advance made.
    let open = settled_in_one_advance!(a, request, "the open root demand");
    assert_eq!(
        open.application_commit_receipt(),
        fresh.application_commit_receipt(),
        "the open root demand settles on the refreshed output"
    );
    let contacts = [
        fresh.producer_contacts_in_this_demand(),
        open.producer_contacts_in_this_demand(),
        settled_in_one_advance!(b, request, "the refreshed middle consumer")
            .producer_contacts_in_this_demand(),
        settled_in_one_advance!(c, request, "the refreshed last consumer")
            .producer_contacts_in_this_demand(),
    ];
    assert_eq!(
        contacts,
        [0, 0, 0, 0],
        "the refreshed chain needs no new producer contact"
    );
    assert_eq!(
        query_entries(),
        before_chain,
        "the dirty chain was already current"
    );
    assert_eq!(
        request.retain_read().unwrap().selected_commit(),
        basis.selected_commit(),
        "settling the refreshed chain commits nothing"
    );
    assert_eq!(
        inexact_deliveries(),
        inexact,
        "every publication stays exact"
    );
    drop((a, b, c, d, fresh_root));
}

#[test]
fn queued_required_work_stays_live_and_exact_across_cycles() {
    let _guard = checkpoint_recovery_test_guard();
    let application = chain_application();
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let (mut a, mut b, mut c, mut d) = chain_with_unrelated!(application, request);
    let inexact = inexact_deliveries();
    for cycle in 0..6_u64 {
        change_root_input!(request, application, 2 + cycle % 2, 0x9176_3300_u64 + cycle);
        let before_unrelated = query_entries();
        settled_in_one_advance!(d, request, "the unrelated required demand");
        assert!(
            query_entries() > before_unrelated,
            "cycle {cycle}: the unrelated advance re-evidences the dirty chain"
        );
        let before_chain = query_entries();
        let contacts = [
            settled_in_one_advance!(c, request, "the last consumer")
                .producer_contacts_in_this_demand(),
            settled_in_one_advance!(b, request, "the middle consumer")
                .producer_contacts_in_this_demand(),
            settled_in_one_advance!(a, request, "the open root demand")
                .producer_contacts_in_this_demand(),
        ];
        assert_eq!(
            contacts,
            [0, 0, 0],
            "cycle {cycle}: no new producer contact"
        );
        assert_eq!(
            query_entries(),
            before_chain,
            "cycle {cycle}: chain was current"
        );
        assert_eq!(
            inexact_deliveries(),
            inexact,
            "cycle {cycle}: exact publications"
        );
    }
    drop((a, b, c, d));
}

/// The chain program with smaller required custody or invalidation retention,
/// and the invalidation resources it retains.
fn limited_application(
    required_custody_bytes: usize,
    invalidation_bytes: u64,
    retained_positions: usize,
) -> (
    application_installation::WorthQueryProgramApplicationRuntime<
        CheckpointSchema,
        program::ChainProgram,
    >,
    worth_query_host::facade::runtime::WorthQueryInvalidationResources,
) {
    let profile =
        worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile::standard()
            .with_registry_required_retained_bytes(
                std::num::NonZeroUsize::new(required_custody_bytes).unwrap(),
            );
    let invalidation = support::invalidation(
        invalidation_bytes,
        u64::try_from(profile.limits().source_currentness_work()).unwrap(),
        retained_positions,
    );
    let application = support::install_program_with_limits::<program::ChainProgram>(
        None,
        profile,
        // A small idempotency window fills within a few cycles, so the
        // steady-retention cycles also prove completed evidence is evicted.
        support::limits(4_096, invalidation.clone()).with_completed_evidence_resources(
            worth_query_host::facade::runtime::WorthQueryCompletedEvidenceResourceProfile::bounded(
                std::num::NonZeroUsize::new(4_096).unwrap(),
            ),
        ),
        source_world::seed,
    );
    (application, invalidation)
}

#[test]
fn a_queued_chain_no_advance_can_fund_never_starves_an_unrelated_caller() {
    let _guard = checkpoint_recovery_test_guard();
    // Room for three Ready rows per settled demand holds the settled chain
    // but not a refreshed generation: the queued chain fails on every
    // advance that tries it.
    let row = worth_query_host::facade::primary_graph::required_ready_custody_bytes_for_test();
    let (application, _) = limited_application(4 * 3 * row, 128 * 1_024 * 1_024, 128);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let (a, b, c, mut d) = chain_with_unrelated!(application, request);
    let inexact = inexact_deliveries();
    change_root_input!(request, application, 2, 0x9176_3a00_u64);
    let before_unrelated = query_entries();
    for attempt in 0..4 {
        let settled = settled_in_one_advance!(d, request, "the unrelated required demand");
        assert_eq!(
            settled.producer_contacts_in_this_demand(),
            0,
            "attempt {attempt}: the unrelated demand needs no producer contact"
        );
    }
    assert!(
        query_entries() > before_unrelated,
        "the unrelated advances attempted the queued chain"
    );
    assert_eq!(
        inexact_deliveries(),
        inexact,
        "every publication stays exact"
    );
    drop((a, b, c, d));
}

#[test]
fn dependents_of_a_failed_required_refresh_never_stay_pending() {
    let _guard = checkpoint_recovery_test_guard();
    let row = worth_query_host::facade::primary_graph::required_ready_custody_bytes_for_test();
    let (application, _) = limited_application(4 * 3 * row, 128 * 1_024 * 1_024, 128);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let (a, mut b, mut c, mut d) = chain_with_unrelated!(application, request);
    change_root_input!(request, application, 2, 0x9176_3b00_u64);
    settled_in_one_advance!(d, request, "the unrelated required demand");
    for attempt in 0..3 {
        for (name, pending) in [
            ("the last consumer", c.advance(&request)),
            ("the middle consumer", b.advance(&request)),
        ] {
            assert!(
                !matches!(
                    pending,
                    Ok(WorthQueryApplicationOutputDemandProgress::Pending)
                ),
                "attempt {attempt}: {name} is Pending with no work a later advance can finish"
            );
        }
    }
    drop((a, b, c, d));
}

mod custody_stops;
mod exhausted_index;
mod reopened_dependent;
#[cfg(feature = "test-output-delivery-faults")]
mod request_stops;
mod steady_retention;
