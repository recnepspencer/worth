//! Counted request stops at the shipped prior-redo read.
use super::*;
use crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial as Resource;
use crate::domain_computation::{
    WorthQueryProviderSessionControlStopKind as Stop,
    WorthQueryProviderSessionDenialKind as Session,
};

#[test]
fn admit_redo_keeps_the_prior_redo_read_stop_at_every_consult() {
    let (world, receipt) = crate::domain_computation::primary_graph::recoverable_application_world(
        213,
        "redo-read-stop",
    );
    let selected = world
        .application
        .with_host_advancement(|phase| world.selected_product(&phase.test_host_request()))
        .unwrap();
    // The shared real handle binding commits its own fixture once per process.
    // That commit is not this door's read, so it settles before the counted
    // runs rather than inside the first one's Work bound and consult count.
    let _ = WorthQueryRecoveryHandleBindingAxisProbe::real();
    let charged_work = crate::test_read_stops::every_advancement_consult_with_door(
        |counted| {
            let proved = WorthQueryProvedUndo::axis_probe_with_product_publication(
                WorthQueryProvedUndoAxisProbe {
                    original_operation: [0x44; 32],
                    principal_scope_digest: bound_scope_digest(),
                    compatibility_generation: 1,
                    runtime_instance: 7,
                },
                receipt.committed_product_publication().clone(),
            );
            let intent = WorthQueryRedoIntent::derive(
                &proved,
                receipt
                    .committed_product_publication()
                    .composite_commit()
                    .clone(),
                receipt.commit_reference().clone(),
            )
            .unwrap();
            let handle = probe_handle(Some([0x66; 32]));
            let authority = WorthQueryRecoveryEffectAuthority::mint(
                handle.runtime_authority(),
                handle.authority_identity(),
            );
            let recovery = WorthQueryRedoRecovery::axis_probe(proved, handle);
            let recovery = std::cell::RefCell::new(Some(recovery));

            // This empty index charges zero Work, so the harness has no
            // one-unit-short Work run for this proof.
            counted(&|request, ceiling| {
                let recovery = recovery.borrow_mut().take().unwrap();
                world
                    .application
                    .with_application_advancement(request, |phase| {
                        let (answer, report) = phase
                            .execution_request_for(&world.application.product_runtime)
                            .unwrap()
                            .run(
                                worth_execution::ExecutionWorkCeiling::new(Some(ceiling)),
                                |_| {
                                    selected
                                        .admit_redo(&phase, recovery, &authority, &intent)
                                        .map(|_| ())
                                        .map_err(|denial| denial.kind())
                                },
                            )
                            .unwrap();
                        assert_eq!(
                            report.charged_work(),
                            0,
                            "the empty indexed result charges no packet work"
                        );
                        answer
                    })
                    .unwrap()
            })
        },
        WorthQueryRedoDenialKind::ExecutionRequest(Err(Stop::Cancelled)),
        WorthQueryRedoDenialKind::ExecutionRequest(Ok(Session::ExecutionResource {
            denial: Resource::WorkExhausted,
            partition_identity: None,
            policy_ancestor: None,
        })),
    );
    assert_eq!(
        charged_work, 0,
        "the empty prior-redo index charges no packet work"
    );
}
