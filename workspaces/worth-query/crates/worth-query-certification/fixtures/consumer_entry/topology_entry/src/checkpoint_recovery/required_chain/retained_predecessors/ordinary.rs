//! Genuine ordinary complete-tree ownership and retirement-safe terminal disposal.
mod successor_custody;
mod unpublished;
use super::super::*;
use super::program::{Program, Root};
use worth_query_host::facade::application_contribution::{
    WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
};
use worth_query_host::facade::application_entry::*;
use worth_query_host::facade::primary_graph::WorthQueryOutputDemandRecoveryPosture;
type Pending = WorthQueryProgramOutputOperation<CheckpointSchema, Program, Root>;
type Settled = WorthQuerySettledProgramOutputOperation<CheckpointSchema, Program, Root>;
fn finish(
    mut operation: Pending,
    request: &WorthQueryApplicationRequest<'_, '_, '_, CheckpointSchema>,
) -> Settled {
    // Fixture hang guard; production advance has no hidden pump or retry quota.
    for _ in 0..256 {
        operation = match operation.advance(request) {
            WorthQueryProgramOutputOperationAdvance::Pending(next) => next,
            WorthQueryProgramOutputOperationAdvance::Settled(value) => return value,
            WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => panic!("{denial:?}"),
            WorthQueryProgramOutputOperationAdvance::Terminal(value) => {
                panic!("{:?}", value.denial())
            }
        };
    }
    panic!("the finite installed tree did not settle");
}
#[test]
fn ordinary_complete_tree_escapes_scope_resumes_reads_current_and_closes() {
    let _guard = checkpoint_recovery_test_guard();
    let mut application = support::install_program::<Program>(None, Default::default());
    let (refusal, original_commit) = {
        let (scope, principal) = authenticate(&application);
        let request = application.request(&principal, &scope);
        let basis = request.retain_current_read().unwrap();
        let original = basis.selected_commit().clone();
        let cancelled = authentication::WorthQueryCancellationSource::new();
        cancelled.cancel();
        let stopped = authentication::WorthQueryRequestScope::new(None, cancelled.token());
        let refusal = match application
            .request(&principal, &stopped)
            .at(&basis)
            .demand(PlanarOutputDemand::new("anchor-a"))
            .controls(Default::default())
            .retain_program_outputs::<Program, Root>(&application)
        {
            Err(refusal) => refusal,
            Ok(_) => panic!("cancelled construction must retain original inputs"),
        };
        assert!(matches!(
            refusal.denial(),
            WorthQueryProgramOutputConstructionDenial::Execution(
                WorthQueryAdvancementDenial::Interrupted(
                    WorthQueryManagedComputationInterruption::Cancelled
                )
            )
        ));
        (refusal, original)
    };
    // The original authentication, request and application borrow have ended.
    assert_eq!(
        application.close_conditional_runtime().unwrap_err().kind(),
        primary_graph::WorthQueryConditionalRuntimeInstallationDenialKind::RuntimeKeepaliveRetained
    );
    assert_eq!(
        refusal.source_observation().unwrap().selected_commit(),
        &original_commit
    );
    let settled = {
        let (scope, principal) = authenticate(&application);
        let request = application.request(&principal, &scope);
        let mut operation = refusal
            .retry(&request)
            .unwrap_or_else(|stop| panic!("{:?}", stop.denial()));
        assert_eq!(
            operation.source_observation().selected_commit(),
            &original_commit
        );
        take_all_decisions();
        let cancellation = authentication::WorthQueryCancellationSource::new();
        let stopped_scope = authentication::WorthQueryRequestScope::new(None, cancellation.token());
        binding::cancel_during_next_decision("anchor-b", cancellation);
        let stopped = application.request(&principal, &stopped_scope);
        let mut retained = None;
        for _ in 0..256 {
            operation = match operation.advance(&stopped) {
                WorthQueryProgramOutputOperationAdvance::Pending(next) => next,
                WorthQueryProgramOutputOperationAdvance::Refused {
                    operation: next,
                    denial,
                } => {
                    assert!(
                        matches!(denial, WorthQueryProgramOutputAdvanceDenial::Outputs(
                        WorthQueryRequiredOutputPreparationDenial::Demand(WorthQueryApplicationOutputDemandDenial::Demand(cause)))
                        if cause.kind() == WorthQueryOutputDemandDenialKind::Cancelled
                            && cause.recovery_posture() == WorthQueryOutputDemandRecoveryPosture::Retryable)
                    );
                    retained = Some(next);
                    break;
                }
                WorthQueryProgramOutputOperationAdvance::Terminal(value) => panic!(
                    "B cancellation must return retryable custody; terminal: {:?}; decisions: {:?}; interruption: {:?}",
                    value.denial(), binding::decisions_snapshot(), stopped_scope.interruption()
                ),
                WorthQueryProgramOutputOperationAdvance::Settled(value) => panic!(
                    "B cancellation must precede full settlement; outputs: {}; decisions: {:?}; interruption: {:?}",
                    value.settlement().output_count(), binding::decisions_snapshot(), stopped_scope.interruption()
                ),
            };
        }
        let settled = finish(
            retained.expect("original cancelled owner returned"),
            &request,
        );
        assert_eq!(
            settled.settlement().root_producer_contacts_in_this_demand(),
            1
        );
        assert_eq!(settled.settlement().output_count(), 4);
        let decisions = take_all_decisions();
        assert_eq!(
            decisions
                .iter()
                .filter(|(key, _)| key == "anchor-b")
                .count(),
            2
        );
        assert_eq!(
            decisions
                .iter()
                .filter(|(key, _)| key == "anchor-c")
                .count(),
            1
        );
        settled
    };
    {
        // Settled custody also escapes the scope that drove its producers.
        let (scope, principal) = authenticate(&application);
        let request = application.request(&principal, &scope);
        let read = settled
            .read_current(
                &request,
                PlanarOutputRead {
                    body_key: "anchor-c".into(),
                },
            )
            .unwrap();
        assert_eq!(read.rows()[0].value, length(11));
        let observation = settled
            .deliver_current(
                &request,
                WorthQueryOperationDeliveryPolicy::currentness(
                    NonZeroUsize::new(1_000_000).unwrap(),
                ),
            )
            .unwrap();
        request
            .at(&observation)
            .require_current_program_output(
                settled.settlement(),
                NonZeroUsize::new(1_000_000).unwrap(),
            )
            .unwrap();
        drop((read, observation));
        settled
            .close(&request)
            .unwrap_or_else(|stop| panic!("{:?}", stop.denial()));
    }
    application
        .close_conditional_runtime()
        .expect("original complete tree interests and keepalive close");
}

#[test]
fn ordinary_terminal_tree_close_preserves_custody_then_releases_after_supersession() {
    use std::sync::{atomic::AtomicBool, Arc};
    let _guard = checkpoint_recovery_test_guard();
    let mut application = support::install_with_domain_denial(Arc::new(AtomicBool::new(true)));
    crate::producer::reset_provider_contacts();
    let decisions_before = crate::handler::handler_contacts();
    {
        let terminal = {
            let (scope, principal) = authenticate(&application);
            let request = application.request(&principal, &scope);
            let before = request.retain_current_read().unwrap();
            let mut pending = request
                .demand(PlanarOutputDemand::new("anchor-a"))
                .retain_program_outputs::<CheckpointProgram, CheckpointRoot>(&application)
                .unwrap_or_else(|stop| panic!("{:?}", stop.denial()));
            let mut terminal = None;
            for _ in 0..256 {
                pending = match pending.advance(&request) {
                    WorthQueryProgramOutputOperationAdvance::Pending(next) => next,
                    WorthQueryProgramOutputOperationAdvance::Terminal(value) => {
                        terminal = Some(value);
                        break;
                    }
                    _ => panic!("registered real producer must deny missing current output"),
                };
            }
            let terminal = terminal.expect("actual producer terminal is owned");
            assert_eq!(
                before.selected_commit(),
                request.retain_current_read().unwrap().selected_commit(),
                "terminal producer does not publish a result"
            );
            terminal
        };
        assert!(
            matches!(terminal.denial(), WorthQueryProgramOutputAdvanceDenial::Outputs(
            WorthQueryRequiredOutputPreparationDenial::Demand(WorthQueryApplicationOutputDemandDenial::Demand(cause)))
            if cause.kind() == WorthQueryOutputDemandDenialKind::ProducerDomainDenied)
        );
        // Provider contacts include input/key/resource callbacks. The actual
        // handler decision, not that callback census, counts execution.
        let contacts = crate::producer::provider_contacts();
        let producer_decisions = crate::handler::handler_contacts();
        assert_eq!(producer_decisions, decisions_before + 1);
        assert!(contacts > 0);
        let foreign = support::install(None);
        let (foreign_scope, foreign_principal) = authenticate(&foreign);
        let refused = match terminal.close(&foreign.request(&foreign_principal, &foreign_scope)) {
            Err(value) => value,
            Ok(_) => panic!("foreign close must retain whole original tree"),
        };
        assert!(matches!(
            refused.denial(),
            WorthQueryProgramOutputCloseDenial::FreshRequestMismatch
        ));
        let (scope, principal) = authenticate(&application);
        let cancellation = authentication::WorthQueryCancellationSource::new();
        cancellation.cancel();
        let stopped = authentication::WorthQueryRequestScope::new(None, cancellation.token());
        let refused = match refused.retry_close(&application.request(&principal, &stopped)) {
            Err(value) => value,
            Ok(_) => panic!("cancelled close must retain actual terminal"),
        };
        assert!(matches!(
            refused.denial(),
            WorthQueryProgramOutputCloseDenial::Execution(
                WorthQueryAdvancementDenial::Interrupted(
                    WorthQueryManagedComputationInterruption::Cancelled
                )
            )
        ));
        assert!(refused.terminal_denial().is_some());
        assert_eq!(crate::handler::handler_contacts(), producer_decisions);
        assert_eq!(crate::producer::provider_contacts(), contacts);
        let request = application.request(&principal, &scope);
        // Deliberate native court edit supersedes the original source. It is an
        // actual PlanarEdit action, not a fabricated no-op source/tree entrance.
        let before = request
            .query(PlanarRead {
                body_key: "anchor-a".into(),
            })
            .execute()
            .unwrap();
        let edited = request
            .advanced()
            .mutate(PlanarEdit(PlanarMutation {
                scope_key: "anchor-a".into(),
                operation: worth_query_consumer_values::PlanarOperation::Adjust(vec![
                    worth_query_consumer_values::PlanarAdjustment {
                        body_key: "anchor-a".into(),
                        replacement_y: length(2),
                    },
                ]),
            }))
            .expect_source(before.observed_sources()[0].clone())
            .idempotency(&0x9176_6601_u64)
            .execute_in_program(
                &application,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        assert!(matches!(
            edited,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ));
        drop(before);
        let source = request
            .query(PlanarRead {
                body_key: "anchor-a".into(),
            })
            .execute()
            .unwrap();
        assert_eq!(source.rows()[0].y, length(2));
        drop(source);
        let decisions_after_edit = crate::handler::handler_contacts();
        assert_eq!(decisions_after_edit, producer_decisions + 1);
        let closed = refused
            .retry_close(&request)
            .unwrap_or_else(|stop| panic!("{:?}", stop.denial()));
        assert!(
            matches!(closed.terminal_denial(), Some(WorthQueryProgramOutputAdvanceDenial::Outputs(
            WorthQueryRequiredOutputPreparationDenial::Demand(WorthQueryApplicationOutputDemandDenial::Demand(cause))))
            if cause.kind() == WorthQueryOutputDemandDenialKind::ProducerDomainDenied)
        );
        assert_eq!(
            crate::producer::provider_contacts(),
            contacts,
            "close does not rerun a producer or create a settlement"
        );
        assert_eq!(
            crate::handler::handler_contacts(),
            decisions_after_edit,
            "read close must not execute the producer or authored edit again"
        );
    }
    application
        .close_conditional_runtime()
        .expect("supersession cannot strand native read custody");
}
