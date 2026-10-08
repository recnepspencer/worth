//! Current-output selection keeps a family's real Preserve head across recovery.
use super::*;
use worth_query_consumer_values::{
    PlanarCurrentOutputExpectation, PlanarDerivedOutput, PlanarOperation,
};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationOutputDemandProgress,
};
use worth_query_host::facade::application_installation::WorthQueryCheckpointCapturePolicy as CapturePolicy;

#[test]
fn preserved_family_head_reopens_for_current_output_before_any_demand() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None);
    let (scope, principal) = support::authenticate(&application);
    let request = application.request(&principal, &scope);
    let mut demand = request
        .demand(PlanarFinalOutputDemand::new("anchor-a"))
        .start_dependent_in_program::<CheckpointProgram, FinalConnection>(&application)
        .unwrap();
    let initial = (0..64)
        .find_map(|_| match demand.advance(&request).unwrap() {
            WorthQueryApplicationOutputDemandProgress::Pending => None,
            WorthQueryApplicationOutputDemandProgress::Settled(value) => Some(value),
        })
        .expect("the actual Initial producer creates the output");
    let entity = initial
        .outputs_of::<FinalPlanarOutputs>()
        .unwrap()
        .entity::<FinalAnchorOutput<CheckpointSchema>>()
        .unwrap()
        .entity_id();
    drop(initial);
    drop(demand);
    drop(request);
    drop(principal);
    drop(scope);
    let initial_checkpoint = application
        .capture_application_checkpoint(CapturePolicy::SystemAllocation)
        .unwrap();
    let initial_bytes = initial_checkpoint.bytes().to_vec();
    drop((initial_checkpoint, application));

    // The old Initial row is genuine recovered custody, so ready-cache
    // replacement cannot remove it from the next capture's recovered input.
    let application = install(Some(
        application_installation::WorthQueryApplicationCheckpoint::from_untrusted_bytes(
            initial_bytes,
        ),
    ));
    let (scope, principal) = support::authenticate(&application);
    let request = application.request(&principal, &scope);
    let observed = request
        .query(PlanarRead {
            body_key: "anchor-a".into(),
        })
        .execute()
        .unwrap();
    request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".into(),
            replacement_y: support::length(9),
        })
        .expect_source(observed.observed_sources()[0].clone())
        .idempotency(&0x9176_6201_u64)
        .execute_performed::<CheckpointProgram, CheckpointRoot>(
            &application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    drop(observed);
    let mut demand = request
        .demand(PlanarFinalOutputDemand::new("anchor-a"))
        .start_dependent_in_program::<CheckpointProgram, FinalConnection>(&application)
        .unwrap();
    let preserved = (0..64)
        .find_map(|_| match demand.advance(&request).unwrap() {
            WorthQueryApplicationOutputDemandProgress::Pending => None,
            WorthQueryApplicationOutputDemandProgress::Settled(value) => Some(value),
        })
        .expect("a distinct Preserve producer publishes the successor");
    assert_eq!(
        preserved
            .outputs_of::<FinalPlanarPreserveOutputs>()
            .unwrap()
            .entity::<FinalPreservedAnchorOutput<CheckpointSchema>>()
            .unwrap()
            .entity_id(),
        entity,
    );
    assert_eq!(
        request
            .query(PlanarOutputRead {
                body_key: "final:anchor-a".into()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        support::length(11),
    );
    drop(preserved);
    drop(demand);
    drop(request);
    drop(principal);
    drop(scope);
    let checkpoint = application
        .capture_application_checkpoint(CapturePolicy::SystemAllocation)
        .unwrap();
    let bytes = checkpoint.bytes().to_vec();
    drop((checkpoint, application));

    // Reenter through untrusted bytes and installed producer readmission.
    // No demand or supplier replay is allowed to repair this current-output read.
    super::super::producer::reset_provider_contacts();
    super::super::final_output::reset_provider_contacts();
    let reopened = install(Some(
        application_installation::WorthQueryApplicationCheckpoint::from_untrusted_bytes(bytes),
    ));
    let (scope, principal) = support::authenticate(&reopened);
    let request = reopened.request(&principal, &scope);
    let verify = |command| {
        let source = request
            .query(PlanarRead {
                body_key: "anchor-a".into(),
            })
            .execute()
            .unwrap();
        request
            .mutate(PlanarEdit(PlanarMutation {
                scope_key: "anchor-a".into(),
                operation: PlanarOperation::VerifyFinalCurrentOutputs(vec![
                    PlanarCurrentOutputExpectation {
                        producer_key: "anchor-a".into(),
                        output_key: "final:anchor-a".into(),
                    },
                ]),
                validator_work: 4096,
            }))
            .expect_source(source.observed_sources()[0].clone())
            .idempotency(&command)
            .execute_in_program(
                &reopened,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
    };
    assert!(matches!(
        verify(0x9176_6202_u64).unwrap(),
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    assert_eq!(super::super::producer::provider_contacts(), 0);
    assert_eq!(super::super::final_output::provider_contacts(), 0);

    let source = request
        .query(PlanarRead {
            body_key: "anchor-a".into(),
        })
        .execute()
        .unwrap();
    request
        .mutate(PlanarEdit(PlanarMutation {
            scope_key: "anchor-a".into(),
            operation: PlanarOperation::PublishDerivedOutput(PlanarDerivedOutput {
                body_key: "final:anchor-a".into(),
                value: support::length(12),
            }),
            validator_work: 4096,
        }))
        .expect_source(source.observed_sources()[0].clone())
        .idempotency(&0x9176_6203_u64)
        .execute_in_program(
            &reopened,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert_stale(verify(0x9176_6204_u64).unwrap_err());
    assert_eq!(super::super::producer::provider_contacts(), 0);
    assert_eq!(super::super::final_output::provider_contacts(), 0);
}

fn assert_stale(
    denial: worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial,
) {
    use std::error::Error;
    use worth_query_host::facade::{
        application_entry::WorthQueryApplicationRequestMutationDenial,
        primary_graph::{
            MutationHandlerExecutionDenial, WorthQueryCurrentOutputDenial,
            WorthQueryCurrentOutputDenialKind,
        },
    };
    let WorthQueryApplicationRequestMutationDenial::Handler(
        MutationHandlerExecutionDenial::Handler(handler),
    ) = denial
    else {
        panic!("{denial:?}")
    };
    let cause = handler
        .source()
        .and_then(|cause| cause.downcast_ref::<WorthQueryCurrentOutputDenial>())
        .expect("typed source-currentness denial");
    assert_eq!(cause.kind(), WorthQueryCurrentOutputDenialKind::StaleSource);
}
