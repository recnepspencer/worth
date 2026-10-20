//! Native dependency currentness keeps the caller's interruption.
use super::*;
impl WorthQueryAuthorizationDecisionFact {
    pub(in crate::domain_computation) fn assert_retained_activation_read_cancelled(
        &self,
        runtime: &RelationalRuntime,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        snapshot: &SnapshotHandle,
        source: &worth_execution::CancellationSource,
    ) {
        let encoded = runtime
            .capture_authorization_durable_dependencies(&self.relational, execution)
            .unwrap()
            .to_wire(execution)
            .unwrap();
        let retained = WorthQueryDurableAuthorizationDependencies {
            recorded_actor: super::DurableRecordedActor::NotApplicable,
            primary: encoded.as_str().to_owned(),
            delegation: None,
            activation: Some(encoded.as_str().to_owned()),
        };
        let observed = encoding::conventional_observed_view(&encoded);
        assert_eq!(
            retained
                .matches_with_retained_activation(execution, &observed, runtime, snapshot)
                .map_err(|denial| denial.kind()),
            Ok(true)
        );
        crate::test_read_stops::every_consult(
            |request| retained.matches_with_retained_activation(request, &observed, runtime, snapshot).map_err(|denial| denial.kind()),
            crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::Stopped(Err(crate::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled)),
            crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::Stopped(Ok(crate::domain_computation::WorthQueryProviderSessionDenialKind::ExecutionResource { denial: crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial::WorkExhausted, partition_identity: None, policy_ancestor: None })),
        );
        use crate::domain_computation::primary_graph::application_attempt::consult_observation::{
            Door, ReadConsultObservationForTest,
        };
        let count = {
            let probe = ReadConsultObservationForTest::install_on_this_thread_for_test(
                Door::Activation,
                None,
            );
            assert_eq!(
                retained
                    .matches_with_retained_activation(execution, &observed, runtime, snapshot)
                    .map_err(|denial| denial.kind()),
                Ok(true)
            );
            probe.consults()
        };
        assert!(count > 1);
        println!("retained activation door: {count} consults");
        for k in 1..=count {
            let cancel = worth_execution::CancellationSource::new();
            let serial = worth_execution::SerialRequest::from_memory(
                worth_execution::SerialMemoryBudget::new(None),
                cancel.token(),
                None,
            );
            let execution = worth_execution::ExecutionRequest::serial(&serial);
            let probe = ReadConsultObservationForTest::install_callback_on_this_thread_for_test(
                Door::Activation,
                Some(k),
                move || cancel.cancel(),
            );
            assert_eq!(retained.matches_with_retained_activation(&execution, &observed, runtime, snapshot).map_err(|denial| denial.kind()), Err(crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::Stopped(Err(crate::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled))));
            assert!(probe.consults() >= k);
        }
        for k in 1..=count {
            let serial = worth_execution::SerialRequest::from_memory(
                worth_execution::SerialMemoryBudget::new(None),
                worth_execution::CancellationToken::new(),
                None,
            );
            let request = worth_execution::ExecutionRequest::serial(&serial);
            let probe = ReadConsultObservationForTest::install_with_work_on_this_thread_for_test(
                Door::Activation,
                Some(k),
                move |context| {
                    assert_eq!(
                        {
                            let context = context.expect("the caller's active work scope");
                            context.checkpoint(
                                context
                                    .remaining_work()
                                    .expect("this adversarial scope selected a finite ceiling")
                                    + 1,
                            )
                        },
                        Err(worth_execution::MapKernelStop::WorkCeiling)
                    );
                },
            );
            let (answer, _) = request
                .run(
                    worth_execution::ExecutionWorkCeiling::new(Some(u64::MAX - 1)),
                    |_| {
                        retained
                            .matches_with_retained_activation(
                                &request, &observed, runtime, snapshot,
                            )
                            .map_err(|denial| denial.kind())
                    },
                )
                .unwrap();
            assert!(
                probe.consults() >= k,
                "the activation read reached consult {k}"
            );
            assert_eq!(answer, Err(crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::Stopped(crate::facade::primary_graph::relational_execution_kind(worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkExhausted, None))));
        }

        source.cancel();
        assert_eq!(
            retained.matches_with_retained_activation(execution, &observed, runtime, snapshot).map_err(|denial| denial.kind()),
            Err(crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::Stopped(Err(crate::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled)))
        );
    }
}
