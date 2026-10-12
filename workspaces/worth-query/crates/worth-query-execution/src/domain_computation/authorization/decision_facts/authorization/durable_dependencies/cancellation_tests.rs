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
        let _ = source;
    }
}
