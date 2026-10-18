//! Real source-value movement with unchanged native membership and complete input.
use super::super::super::super::custody_source::{
    install, UpstreamRoot, ValueSourceDemand, ValueSourceProgram, ValueSourceRoot,
    ValueSourceSchema,
};
use super::*;

fn finish_value(
    mut operation: WorthQueryProgramOutputOperation<
        ValueSourceSchema,
        ValueSourceProgram,
        ValueSourceRoot,
    >,
    request: &WorthQueryApplicationRequest<'_, '_, '_, ValueSourceSchema>,
) -> WorthQuerySettledProgramOutputOperation<ValueSourceSchema, ValueSourceProgram, ValueSourceRoot>
{
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
    panic!("the native one-root consumer did not settle");
}

#[test]
fn closed_complete_consumer_reuses_after_same_member_source_value_edit() {
    let _guard = checkpoint_recovery_test_guard();
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    take();
    let mut application = install(None);
    {
        let (scope, principal) = authenticate(&application);
        let request = application.request(&principal, &scope);
        let mut upstream = request
            .demand(PlanarOutputDemand::new("anchor-a"))
            .start_in_program::<ValueSourceProgram, UpstreamRoot>(&application)
            .unwrap();
        let original_upstream = settle!(upstream, &request);
        let open = || {
            request
                .demand(ValueSourceDemand("anchor-b".into()))
                .retain_program_outputs::<ValueSourceProgram, ValueSourceRoot>(&application)
                .unwrap_or_else(|stop| panic!("{:?}", stop.denial()))
        };
        let first = finish_value(open(), &request);
        assert!(first.settlement().root_producer_contacts_in_this_demand() > 0);
        let original = first
            .close(&request)
            .unwrap_or_else(|stop| panic!("{:?}", stop.denial()));
        drop((upstream, original_upstream));
        let closed = take();
        assert!(
            closed
                .registry_custody
                .iter()
                .flatten()
                .any(|event| event.transition == Transition::CachedReadyClosed
                    && event.prerequisites == 1),
            "complete close retains the actual posted consumer: {closed:?}"
        );
        assert_eq!(closed.registry_custody_omitted, 0);
        let before = request
            .query(PlanarRead {
                body_key: "anchor-b".into(),
            })
            .execute()
            .unwrap();
        let sibling = request
            .query(PlanarRead {
                body_key: "anchor-c".into(),
            })
            .execute()
            .unwrap();
        request
            .advanced()
            .mutate(PlanarSourceAdjustment {
                scope_key: "anchor-c".into(),
                replacement_y: length(11),
            })
            .expect_source(sibling.observed_sources()[0].clone())
            .idempotency(&0x9176_33b0_u64)
            .execute_performed::<ValueSourceProgram, UpstreamRoot>(
                &application,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        drop(sibling);
        let changed = request
            .query(PlanarRead {
                body_key: "anchor-c".into(),
            })
            .execute()
            .unwrap();
        assert_eq!(changed.rows()[0].y, length(11));
        let after = request
            .query(PlanarRead {
                body_key: "anchor-b".into(),
            })
            .execute()
            .unwrap();
        assert_eq!(
            before.rows(),
            after.rows(),
            "the entire projected input stays equal"
        );
        drop((before, after, changed));
        let edit = take();
        eprintln!("closed value consumer source edit: {edit:?}");
        assert_eq!(edit.registry_custody_omitted, 0);
        let cancellation = authentication::WorthQueryCancellationSource::new();
        let stopped_scope = authentication::WorthQueryRequestScope::new(None, cancellation.token());
        let stopped = application.request(&principal, &stopped_scope);
        cancellation.cancel();
        let operation = match open().advance(&stopped) {
            WorthQueryProgramOutputOperationAdvance::Refused { operation, denial } => {
                assert!(matches!(
                    denial,
                    WorthQueryProgramOutputAdvanceDenial::Execution(
                        WorthQueryAdvancementDenial::Interrupted(
                            WorthQueryManagedComputationInterruption::Cancelled
                        )
                    )
                ));
                operation
            }
            _ => panic!("cancelled advance must retain the original native owner"),
        };
        let reused = finish_value(operation, &request);
        let observed = take();
        eprintln!("closed value consumer successor: {observed:?}");
        assert_eq!(
            reused.settlement().root_producer_contacts_in_this_demand(),
            0
        );
        assert!(
            observed
                .input_cutoffs
                .iter()
                .flatten()
                .any(|event| event.producer
                    == "worth.query.certification.consumed-value-source-producer.v1"
                    && event.gate
                        == primary_graph::WorthQueryInputCutoffGateForTest::PublicationApply
                    && event.outcome
                        == primary_graph::WorthQueryInputCutoffOutcomeForTest::Published),
            "the real native cutoff must publish: {observed:?}"
        );
        assert_eq!(observed.registry_custody_omitted, 0);
        let output = reused
            .read_current(
                &request,
                PlanarOutputRead {
                    body_key: "anchor-b".into(),
                },
            )
            .unwrap();
        assert_eq!(output.rows()[0].value, length(16));
        drop(output);
        let closed_successor = reused
            .close(&request)
            .unwrap_or_else(|stop| panic!("{:?}", stop.denial()));
        drop((original, closed_successor));
    }
    application.close_conditional_runtime().unwrap();
}
