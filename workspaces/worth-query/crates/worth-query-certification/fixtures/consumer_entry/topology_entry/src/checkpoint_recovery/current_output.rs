use super::*;
use crate::{PlanarEdit, PlanarMutation};
use worth_query_consumer_values::{PlanarCurrentOutputExpectation, PlanarOperation};
use worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome;
use worth_query_host::facade::application_installation::WorthQueryCheckpointCapturePolicy as CapturePolicy;

#[test]
fn restored_current_output_is_tracked_before_any_producer_demand_and_rejects_source_change() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    drop(settle(&request, &application));
    drop(principal);
    drop(scope);
    let checkpoint = application
        .capture_application_checkpoint(CapturePolicy::SystemAllocation)
        .unwrap();
    drop(application);

    super::super::producer::reset_provider_contacts();
    let restored = install(Some(checkpoint));
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
            .mutate(PlanarEdit(PlanarMutation {
                scope_key: "anchor-a".into(),
                operation: PlanarOperation::VerifyCurrentOutputs(vec![
                    PlanarCurrentOutputExpectation {
                        producer_key: "anchor-a".into(),
                        output_key: "anchor-a".into(),
                    },
                ]),
                validator_work: 4096,
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
        super::super::producer::provider_contacts(),
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
    assert_eq!(super::super::producer::provider_contacts(), 0);
}
