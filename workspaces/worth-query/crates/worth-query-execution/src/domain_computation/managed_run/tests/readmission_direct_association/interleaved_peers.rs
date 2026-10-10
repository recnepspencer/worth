use super::*;

#[test]
fn interleaved_readmitted_peers_keep_attempt_ledger_and_release_authority_associated() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;
        let active_request = execution;

        let (first, second, bridge, runtime) = shared_yielded_peers(execution, resource_request);
        let first_binding = first.inspection().operation_binding_identity().to_owned();
        let second_binding = second.inspection().operation_binding_identity().to_owned();

        let first = match first.readmit_same_runtime(active_request, &runtime, &bridge) {
            crate::domain_computation::WorthQueryDirectReadmissionOutcome::Readmitted(
                readmitted,
            ) => readmitted.into_active(),
            _ => panic!("first shared peer should readmit"),
        };
        let second = match second.readmit_same_runtime(active_request, &runtime, &bridge) {
            crate::domain_computation::WorthQueryDirectReadmissionOutcome::Readmitted(
                readmitted,
            ) => readmitted.into_active(),
            _ => panic!("second shared peer should readmit"),
        };
        let first_resource = first.resource_attempt_identity().to_owned();
        let second_resource = second.resource_attempt_identity().to_owned();
        let first_session = first.provider_session_identity().to_owned();
        let second_session = second.provider_session_identity().to_owned();
        let first_bridge_basis = first.bridge_basis_identity().to_owned();
        let second_bridge_basis = second.bridge_basis_identity().to_owned();
        let first_bridge_intent = worth_runtime_bridge::facade::BridgeManagedExecutionIntent::new(
            first_binding,
            first_resource.clone(),
        )
        .identity()
        .as_str()
        .to_owned();
        let second_bridge_intent = worth_runtime_bridge::facade::BridgeManagedExecutionIntent::new(
            second_binding,
            second_resource.clone(),
        )
        .identity()
        .as_str()
        .to_owned();
        assert_ne!(first_resource, second_resource);
        assert_ne!(first_session, second_session);
        assert_ne!(first_bridge_basis, second_bridge_basis);
        assert_ne!(first_bridge_intent, second_bridge_intent);

        let first = match first.advance(execution) {
            WorthQueryDirectGraphStepOutcome::Completed(completion) => completion,
            _ => panic!("first restored peer should complete"),
        }
        .into_running()
        .completed()
        .expect("first peer should terminalize");
        assert_eq!(
            first.provider_work().provider_session_identity(),
            first_session
        );
        let first = first.cleanup().expect("first peer should clean up");
        assert_cleanup_association(&first, &first_resource, &first_session);

        assert_eq!(second.retained_capacity_reservation_count(), 2);
        let second = match second.advance(execution) {
            WorthQueryDirectGraphStepOutcome::Completed(completion) => completion,
            _ => panic!("second peer must remain live after first cleanup"),
        }
        .into_running()
        .completed()
        .expect("second peer should terminalize");
        assert_eq!(
            second.provider_work().provider_session_identity(),
            second_session
        );
        let second = second.cleanup().expect("second peer should clean up");
        assert_cleanup_association(&second, &second_resource, &second_session);
    });
}
