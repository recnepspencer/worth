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
            .to_wire_string()
            .unwrap();
        let retained = WorthQueryDurableAuthorizationDependencies {
            primary: encoded.clone(),
            delegation: None,
            activation: Some(encoded.clone()),
        };
        let observed = WorthQueryDurableAuthorizationDependencies {
            primary: encoded,
            delegation: None,
            activation: None,
        };
        use crate::domain_computation::primary_graph::application_attempt::consult_observation::{
            Door, ReadConsultObservationForTest,
        };
        let count = {
            let probe = ReadConsultObservationForTest::install_on_this_thread_for_test(
                Door::Activation,
                None,
            );
            assert_eq!(
                retained.matches_with_retained_activation(execution, &observed, runtime, snapshot),
                Ok(true)
            );
            probe.consults()
        };
        assert!(count > 1);
        println!("retained activation door: {count} consults");
        for k in 1..=count {
            let cancel = worth_execution::CancellationSource::new();
            let serial = worth_execution::SerialRequest::from_memory(
                worth_execution::SerialMemoryBudget::new(64 << 20),
                cancel.token(),
                None,
            );
            let execution = worth_execution::ExecutionRequest::serial(&serial);
            let probe = ReadConsultObservationForTest::install_callback_on_this_thread_for_test(
                Door::Activation,
                Some(k),
                move || cancel.cancel(),
            );
            assert_eq!(retained.matches_with_retained_activation(&execution, &observed, runtime, snapshot), Err(worth_relational::facade::transactions::RelationalExecutionDenialCause::Cancelled));
            assert!(probe.consults() >= k);
        }
        for k in 1..=count {
            let serial = worth_execution::SerialRequest::from_memory(
                worth_execution::SerialMemoryBudget::new(64 << 20),
                worth_execution::CancellationToken::new(),
                None,
            );
            let request = worth_execution::ExecutionRequest::serial(&serial);
            let probe = ReadConsultObservationForTest::install_with_work_on_this_thread_for_test(
                Door::Activation,
                Some(k),
                move |context| {
                    assert_eq!(
                        context
                            .expect("the caller's active work scope")
                            .checkpoint(k as u64),
                        Err(worth_execution::MapKernelStop::WorkCeiling)
                    );
                },
            );
            let (answer, _) = request
                .run(
                    worth_execution::ExecutionWorkCeiling::new((k - 1) as u64),
                    |_| {
                        retained.matches_with_retained_activation(
                            &request, &observed, runtime, snapshot,
                        )
                    },
                )
                .unwrap();
            assert!(
                probe.consults() >= k,
                "the activation read reached consult {k}"
            );
            assert_eq!(answer, Err(worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkExhausted));
        }
        let _ = source;
    }
}
