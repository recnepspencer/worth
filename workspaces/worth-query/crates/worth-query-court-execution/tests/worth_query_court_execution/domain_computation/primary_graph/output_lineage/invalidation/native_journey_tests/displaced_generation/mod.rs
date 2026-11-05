//! Every publication retires the row of the generation its record displaces,
//! whether or not a demand manages it. The lineage never names a displaced
//! generation again, so the owner keeps a release it could not complete.

use crate::domain_computation::primary_graph::output_lineage::invalidation::native_journey_tests::*;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::preimage_evidence::retained_status_program;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::preimage_evidence::RetentionMutationBreadth;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;

pub type Identity = Arc<RecordedSettlementIdentity>;

/// Runs `journey` over a world whose publications no demand owns: they carry
/// no prerequisites. `publish(status, replacement, key)` publishes one output
/// partition at the next product generation, and `live_rows` answers which
/// settlements the live mark root still holds.
pub fn with_unmanaged_publications(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    journey: impl FnOnce(
        &SourceInvalidationOwner,
        &dyn Fn(&str, &str, u8) -> Identity,
        &dyn Fn(&[&Identity]) -> Vec<bool>,
    ),
) {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&phase.test_host_request_for_test(), &world, &request)
        })
        .expect("the principal fixture opens its host call");
    let graph = (*world.application.runtime_for_test())
        .primary_graph_for_test()
        .unwrap();
    let handle = graph.integration_handle_for_test();
    let owner = &(*(*handle.source_owner_for_test()).invalidation_owner_for_test());
    let partition = [7; 32];
    let publish = |status: &str, replacement: &str, key: u8| {
        let program = world
            .application
            .with_host_advancement(|phase| {
                let phase_custody = &phase;
                let account = resolved_account(
                    &phase.test_host_request_for_test(),
                    &world,
                    status,
                    &request,
                );

                retained_status_program(
                    &(phase_custody).test_host_request_for_test(),
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
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        );
        let WorthQueryApplicationCommitOutcome::Committed(receipt) = outcome else {
            panic!("the output publication commits: {outcome:?}");
        };
        Arc::clone(
            receipt
                .exact_output_settlement_for_test()
                .expect("an output publication records its settlement"),
        )
    };
    let live_rows = |identities: &[&Identity]| {
        handle.with_runtime_for_test(request_execution, |runtime| {
            let (basis_handle, basis) = snapshot(runtime);
            let cell = owner.cell_for_read_for_test(&basis).unwrap().unwrap();

            let image = cell.read_image();
            let live = identities
                .iter()
                .map(|identity| {
                    (*(*image.payload().current_for_test()).settlements_for_test())
                        .contains_key(*identity)
                })
                .collect::<Vec<_>>();
            runtime.snapshots().release_snapshot(&basis_handle).unwrap();
            live
        })
    };
    journey(owner, &publish, &live_rows);
}

#[test]
fn a_publication_without_prerequisites_retires_the_row_its_generation_displaces() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
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
