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
        crate::test_read_stops::every_consult(
            |request| retained.matches_with_retained_activation(request, &observed, runtime, snapshot),
            crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::Stopped(Err(crate::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled)),
            crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::Stopped(Ok(crate::domain_computation::WorthQueryProviderSessionDenialKind::ExecutionResource { denial: crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial::WorkExhausted, partition_identity: None, policy_ancestor: None })),
        );
        source.cancel();
        assert_eq!(
            retained.matches_with_retained_activation(execution, &observed, runtime, snapshot),
            Err(crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::Stopped(Err(crate::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled)))
        );
    }
}
