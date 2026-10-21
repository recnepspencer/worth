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
                WorthQueryApplicationOutputDemandProgress::Pending
                | WorthQueryApplicationOutputDemandProgress::Progressed => None,
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
            WorthQueryApplicationOutputDemandProgress::Pending
            | WorthQueryApplicationOutputDemandProgress::Progressed => {
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
            .advanced()
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

/// Writes the Y of `$key`; every legal source write must be performed.
macro_rules! writes_y {
    ($request:expr, $application:expr, $key:expr, $y:expr, $idempotency:expr) => {{
        let selected = $request
            .query(PlanarRead {
                body_key: $key.to_owned(),
            })
            .execute()
            .unwrap();
        let changed = $request
            .advanced()
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
        match changed {
            Ok(WorthQueryApplicationPerformedMutationOutcome::Performed(_)) => (),
            Err(stop) => panic!("a legal source write must commit: {stop:?}"),
            Ok(_) => panic!("a legal source write was not performed"),
        }
    }};
}

fn chain_application() -> application_installation::WorthQueryProgramApplicationRuntime<
    CheckpointSchema,
    program::ChainProgram,
> {
    chain_application_from_checkpoint(None)
}

fn chain_application_from_checkpoint(
    checkpoint: Option<application_installation::WorthQueryApplicationCheckpoint>,
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
        source_world::seed,
    )
}

/// A, B and C form the dirty chain; D is required but reads none of it.
macro_rules! chain_with_unrelated {
    ($application:expr, $request:expr) => {
        chain_with_unrelated!($application, $request, (), ())
    };
    ($application:expr, $request:expr, $before:expr, $after:expr) => {{
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
        $before;
        settle!(a, $request);
        settle!(b, $request);
        settle!(c, $request);
        settle!(d, $request);
        $after;
        (a, b, c, d)
    }};
}

mod current_acknowledgement;
mod owned_continuation;
mod selection_at_inspection;

#[test]
fn an_unrelated_required_advance_progresses_the_dirty_required_set() {
    let _guard = checkpoint_recovery_test_guard();
    run_unrelated_required_advance(chain_application(), [1, 1, 1]);
}

#[test]
fn restored_required_demands_reenter_source_query() {
    let _guard = checkpoint_recovery_test_guard();
    let checkpoint = ready_chain_checkpoint();
    // The restored original demands must reenter their authentic source query.
    run_unrelated_required_advance(
        chain_application_from_checkpoint(Some(checkpoint)),
        [0, 1, 1],
    );
}

fn ready_chain_checkpoint() -> application_installation::WorthQueryApplicationCheckpoint {
    let application = chain_application();
    {
        let (scope, principal) = authenticate(&application);
        let request = application.request(&principal, &scope);
        let _demands = chain_with_unrelated!(application, request);
    }
    application
        .with_host_advancement(|host_phase| {
            application
                .capture_application_checkpoint(&host_phase, CapturePolicy::SystemAllocation)
                .expect("the settled required chain is checkpointable")
        })
        .expect("installed host call opens")
}

fn run_unrelated_required_advance(
    application: application_installation::WorthQueryProgramApplicationRuntime<
        CheckpointSchema,
        program::ChainProgram,
    >,
    initial_contacts: [usize; 3],
) {
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let (mut a, mut b, mut c, mut d) = chain_with_unrelated!(application, request);
    let initial_counts = [
        settle!(a, request).producer_contacts_in_this_demand(),
        settle!(b, request).producer_contacts_in_this_demand(),
        settle!(c, request).producer_contacts_in_this_demand(),
    ];
    assert_eq!(
        initial_counts, initial_contacts,
        "checkpoint capture keeps the root with no consumed edge; recorded_output.rs:73-77 excludes consumed B/C outputs, so each joins with one handler contact"
    );
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
    let basis = crate::checkpoint_recovery::support::retain_read(&application, &request).unwrap();
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
        [0, initial_counts[0], initial_counts[1], initial_counts[2]],
        "the fresh demand initiated nothing; each open demand retains its initial execution count"
    );
    assert_eq!(
        [
            contacts[0],
            contacts[1] - initial_counts[0],
            contacts[2] - initial_counts[1],
            contacts[3] - initial_counts[2]
        ],
        [0, 0, 0, 0],
        "the refreshed chain needs no new producer contact"
    );
    assert_eq!(
        query_entries(),
        before_chain,
        "the dirty chain was already current"
    );
    assert_eq!(
        crate::checkpoint_recovery::support::retain_read(&application, &request)
            .unwrap()
            .selected_commit(),
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
            [1, 1, 1],
            "cycle {cycle}: each admitted demand keeps its initial execution, with no additional contact"
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

mod pending_queue;
mod recovery_isolation;
mod reopened_dependent;
mod request_stops;

// Contact probes share this module's existing observer boundary.
#[path = "performed_head_movement.rs"]
mod performed_head_movement;

mod current_consumption;
mod retained_currentness;

mod checkpoint_failures;
