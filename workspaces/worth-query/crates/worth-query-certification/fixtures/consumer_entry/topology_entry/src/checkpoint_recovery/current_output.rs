use super::*;
use crate::{PlanarEdit, PlanarMutation};
use worth_query_consumer_values::{PlanarCurrentOutputExpectation, PlanarOperation};
use worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome;
use worth_query_host::facade::application_installation::WorthQueryCheckpointCapturePolicy as CapturePolicy;

#[test]
fn restored_current_output_is_tracked_before_any_producer_demand_and_rejects_source_change() {
    let _guard = checkpoint_recovery_test_guard();
    let provider_observation = crate::producer::ProviderContacts::default();
    let application = support::observed::install(None, &provider_observation);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    drop(settle(&request, &application));
    drop(principal);
    drop(scope);
    let checkpoint = application
        .with_host_advancement(|host_phase| {
            application
                .capture_application_checkpoint(&host_phase, CapturePolicy::SystemAllocation)
                .unwrap()
        })
        .expect("installed host call opens");
    drop(application);

    provider_observation.reset();
    let restored = support::observed::install(Some(checkpoint), &provider_observation);
    let (scope, principal) = authenticate(&restored);
    let request = restored.request(&principal, &scope);
    let verify = |command| {
        let observed = request
            .query(PlanarRead {
                body_key: "anchor-a".into(),
            })
            .execute()
            .unwrap();
        request
            .advanced()
            .mutate(PlanarEdit(PlanarMutation {
                scope_key: "anchor-a".into(),
                // Every expectation resolves its source entity, consumes current_output,
                // and reads the output field through the real decision reader.
                operation: PlanarOperation::VerifyCurrentOutputs(
                    (0..8_193)
                        .map(|_| PlanarCurrentOutputExpectation {
                            producer_key: "anchor-a".into(),
                            output_key: "anchor-a".into(),
                        })
                        .collect(),
                ),
            }))
            .expect_source(observed.observed_sources()[0].clone())
            .idempotency(&command)
            .execute_in_program(
                &restored,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
    };
    let outcome =
        verify(901_u64).expect("restored lineage is available to the ordinary tracked reader");
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "current output verification must commit"
    );
    assert_eq!(
        provider_observation.count(),
        0,
        "current_output must not warm a producer"
    );

    let observed = request
        .query(PlanarRead {
            body_key: "anchor-a".into(),
        })
        .execute()
        .unwrap();
    request
        .advanced()
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".into(),
            replacement_y: length(2),
        })
        .expect_source(observed.observed_sources()[0].clone())
        .idempotency(&902_u64)
        .execute_performed::<CheckpointProgram, CheckpointRoot>(
            &restored,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let denial = verify(903_u64).expect_err("restored facts must still reject a changed source");
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
        .expect("typed currentness cause");
    assert_eq!(cause.kind(), WorthQueryCurrentOutputDenialKind::StaleSource);
    assert_eq!(provider_observation.count(), 0);

    drop(settle(&request, &restored));
    assert!(provider_observation.count() > 0);
    let refreshed = request
        .query(PlanarOutputRead {
            body_key: "anchor-a".into(),
        })
        .execute()
        .expect("the refreshed producer output is readable");
    assert_eq!(refreshed.rows()[0].value, length(3));
    assert!(matches!(
        verify(904_u64).expect("fresh output verifies after ordinary producer demand"),
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
}
