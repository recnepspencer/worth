use super::{shared, BridgeHarnessError, SpeculationHarnessExecution};
use crate::harness::fixtures::BridgeHarnessFixture;

pub(super) fn execute_discard_certification(
    runtime_bridge: &crate::facade::RuntimeBridge,
    fixture: &BridgeHarnessFixture,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<SpeculationHarnessExecution, BridgeHarnessError> {
    let admitted = runtime_bridge
        .admit_preview_session(
            crate::speculation::BridgePreviewSessionIdentity::admit_bridge_owned(
                "harness:speculation-discard",
            ),
            shared::preview_declaration(
                crate::facade::BridgePreviewSessionDeclarationIdentity::admit_bridge_owned(
                    "harness:speculation-discard",
                ),
                crate::facade::BridgeSpeculativeBranchBindingIdentity::admit_bridge_owned(
                    "harness:speculation-discard:binding",
                ),
                crate::truth_identity_fixtures::truth_branch_fixture("main"),
                crate::facade::BridgeSignalBranchIdentity::admit_bridge_owned("signal:discard"),
                crate::truth_identity_fixtures::truth_snapshot_fixture(
                    "harness:speculation-discard:snapshot",
                ),
            ),
        )
        .map_err(|error| {
            BridgeHarnessError::new(format!("speculation admission failed: {error}"))
        })?;
    let (active, execution_record) = runtime_bridge.activate_preview_session(admitted, 4, 2, 2);
    let (_discarded, discard_record) = runtime_bridge
        .discard_preview_session(
            active,
            &execution_record,
            vec![
                crate::facade::BridgePreviewResidueClass::PreviewExecutionRetained,
                crate::facade::BridgePreviewResidueClass::ReplayRetainedNonAuthoritative,
                crate::facade::BridgePreviewResidueClass::TemporaryRoutingResidue,
                crate::facade::BridgePreviewResidueClass::TemporaryDiagnosticsResidue,
            ],
        )
        .map_err(|error| BridgeHarnessError::new(format!("speculation discard failed: {error}")))?;
    let routing_digest =
        shared::first_commit_routing_digest(runtime_bridge, fixture, resource_request)?;

    Ok(SpeculationHarnessExecution::Discard {
        execution_record,
        discard_record,
        routing_digest,
    })
}
