//! Every publication retires the row of the generation its record displaces,
//! whether or not a demand manages it. The lineage never names a displaced
//! generation again, so the owner keeps a release it could not complete.

use super::*;
use crate::domain_computation::primary_graph::tests::application_attempt::{
    authenticated_principal, idempotency,
    preimage_evidence::{retained_status_program, RetentionMutationBreadth},
    resolved_account,
};
use crate::domain_computation::primary_graph::WorthQueryApplicationCommitOutcome;

type Identity = Arc<RecordedSettlementIdentity>;

/// Runs `journey` over a world whose publications no demand owns: they carry
/// no prerequisites. `publish(status, replacement, key)` publishes one output
/// partition at the next product generation, and `live_rows` answers which
/// settlements the live mark root still holds.
fn with_unmanaged_publications(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    journey: impl FnOnce(
        &SourceInvalidationOwner,
        &dyn Fn(&str, &str, u8) -> Identity,
        &dyn Fn(&[&Identity]) -> Vec<bool>,
    ),
) {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&phase.test_host_request(), &world, &request)
        })
        .expect("the principal fixture opens its host call");
    let graph = world.application.runtime.primary_graph().unwrap();
    let handle = graph.integration_handle();
    let owner = &handle.source_owner.invalidation_owner;
    let partition = [7; 32];
    let publish = |status: &str, replacement: &str, key: u8| {
        let program = world
            .application
            .with_host_advancement(|phase| {
                let phase_custody = &phase;
                let account =
                    resolved_account(&phase.test_host_request(), &world, status, &request);

                retained_status_program(
                    &(phase_custody).test_host_request(),
                    &world,
                    &principal,
                    &account,
                    &request,
                    replacement,
                    RetentionMutationBreadth::Narrow,
                )
            })
            .expect("the publication fixture opens its preparation call");
        let outcome = world.application.compare_and_commit_application(
            program,
            idempotency(key, key + 1).bind_source_partition(&partition),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        );
        let WorthQueryApplicationCommitOutcome::Committed(receipt) = outcome else {
            panic!("the output publication commits: {outcome:?}");
        };
        Arc::clone(
            receipt
                .exact_output_settlement()
                .expect("an output publication records its settlement"),
        )
    };
    let live_rows = |identities: &[&Identity]| {
        handle.with_runtime_mut(request_execution, |runtime| {
            let (basis_handle, basis) = snapshot(runtime);
            let cell = owner.cell_for_read(&basis).unwrap().unwrap();

            let image = cell.read_image();
            let live = identities
                .iter()
                .map(|identity| image.payload().current.settlements.contains_key(*identity))
                .collect::<Vec<_>>();
            runtime.snapshots().release_snapshot(&basis_handle).unwrap();
            live
        })
    };
    journey(owner, &publish, &live_rows);
}

#[test]
fn a_publication_without_prerequisites_retires_the_row_its_generation_displaces() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    with_unmanaged_publications(request_execution, |_, publish, live_rows| {
        let first = publish("open", "frozen", 31);
        assert_eq!(live_rows(&[&first]), [true]);
        let second = publish("frozen", "closed", 33);
        assert_ne!(first, second);
        assert_eq!(
            live_rows(&[&first, &second]),
            [false, true],
            "the newer generation's publication retires the row it displaces"
        );
    });
}
