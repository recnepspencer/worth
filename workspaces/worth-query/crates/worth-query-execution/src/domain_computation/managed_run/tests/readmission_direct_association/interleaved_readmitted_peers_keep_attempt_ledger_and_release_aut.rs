//! interleaved readmitted peers keep attempt ledger and release authority associated.
use super::*;

#[test]
fn interleaved_readmitted_peers_keep_attempt_ledger_and_release_authority_associated() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;
        let active_request = execution;

        let (first, second, bridge, runtime) = shared_yielded_peers(execution);
        let first_binding = first.inspection().operation_binding_identity().to_owned();
        let second_binding = second.inspection().operation_binding_identity().to_owned();

        let first = {
            let fixture_phase_result = {
                let installed_fixture_phase = first.fixture_phase(active_request);
                first.readmit_same_runtime(&installed_fixture_phase, &runtime, &bridge)
            };
            match fixture_phase_result {
                crate::domain_computation::WorthQueryDirectReadmissionOutcome::Readmitted(
                    readmitted,
                ) => readmitted.into_active(),
                _ => panic!("first shared peer should readmit"),
            }
        };
        let second = {
            let fixture_phase_result = {
                let installed_fixture_phase = second.fixture_phase(active_request);
                second.readmit_same_runtime(&installed_fixture_phase, &runtime, &bridge)
            };
            match fixture_phase_result {
                crate::domain_computation::WorthQueryDirectReadmissionOutcome::Readmitted(
                    readmitted,
                ) => readmitted.into_active(),
                _ => panic!("second shared peer should readmit"),
            }
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

        let first = {
            let fixture_phase_result = {
                let installed_fixture_phase = first.fixture_phase(execution);
                first.advance(&installed_fixture_phase)
            };
            match fixture_phase_result {
                WorthQueryDirectGraphStepOutcome::Completed(completion) => completion,
                _ => panic!("first restored peer should complete"),
            }
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
        let second = {
            let fixture_phase_result = {
                let installed_fixture_phase = second.fixture_phase(execution);
                second.advance(&installed_fixture_phase)
            };
            match fixture_phase_result {
                WorthQueryDirectGraphStepOutcome::Completed(completion) => completion,
                _ => panic!("second peer must remain live after first cleanup"),
            }
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
