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
        assert_eq!(
            retained.matches_with_retained_activation(execution, &observed, runtime, snapshot),
            Ok(true)
        );
        source.cancel();
        assert_eq!(
            retained.matches_with_retained_activation(execution, &observed, runtime, snapshot),
            Err(worth_execution::MapKernelStop::Cancelled)
        );
    }
}
