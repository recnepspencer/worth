use super::{shared, BridgeHarnessError, SpeculationHarnessExecution};
use crate::harness::fixtures::BridgeHarnessFixture;
use crate::routing::canonicalization::digest_string;

pub(super) fn execute_promotion_certification(
    runtime_bridge: &crate::facade::RuntimeBridge,
    fixture: &BridgeHarnessFixture,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<SpeculationHarnessExecution, BridgeHarnessError> {
    let promotion_session_identity =
        crate::speculation::BridgePreviewSessionIdentity::admit_bridge_owned(
            "harness:speculation-promotion",
        );
    let admitted = runtime_bridge
        .admit_preview_session(
            promotion_session_identity.clone(),
            shared::preview_declaration(
                crate::facade::BridgePreviewSessionDeclarationIdentity::admit_bridge_owned(
                    "harness:speculation-promotion",
                ),
                crate::facade::BridgeSpeculativeBranchBindingIdentity::admit_bridge_owned(
                    "harness:speculation-promotion:binding",
                ),
                crate::truth_identity_fixtures::truth_branch_fixture("main"),
                crate::facade::BridgeSignalBranchIdentity::admit_bridge_owned("signal:promotion"),
                crate::truth_identity_fixtures::truth_snapshot_fixture(
                    "harness:speculation-promotion:snapshot",
                ),
            ),
        )
        .map_err(|error| {
            BridgeHarnessError::new(format!("speculation admission failed: {error}"))
        })?;
    let (promoted_active, promoted_execution_record) =
        runtime_bridge.activate_preview_session(admitted, 3, 1, 2);
    let preview = crate::facade::BridgeSpeculativeSessionHandle::new(
        runtime_bridge.clone(),
        promoted_active,
        promoted_execution_record,
    );
    let promoted_execution_record = preview.execution_record().clone();
    let outcome = match preview.promote() {
        Ok(outcome) => outcome,
        Err((preview, denial)) => return Err(BridgeHarnessError::preview_refused(preview, denial)),
    };
    let promotion_record = outcome.record().clone();
    let promoted_replay_bundle = runtime_bridge
        .replay_preview_bundle(&promotion_session_identity)
        .map_err(|error| BridgeHarnessError::new(format!("speculation replay failed: {error}")))?;

    let discard_sibling_session_identity =
        crate::speculation::BridgePreviewSessionIdentity::admit_bridge_owned(
            "harness:speculation-discard-sibling",
        );
    let discarded_admitted = runtime_bridge
        .admit_preview_session(
            discard_sibling_session_identity.clone(),
            shared::preview_declaration(
                crate::facade::BridgePreviewSessionDeclarationIdentity::admit_bridge_owned(
                    "harness:speculation-discard-sibling",
                ),
                crate::facade::BridgeSpeculativeBranchBindingIdentity::admit_bridge_owned(
                    "harness:speculation-discard-sibling:binding",
                ),
                crate::truth_identity_fixtures::truth_branch_fixture("main"),
                crate::facade::BridgeSignalBranchIdentity::admit_bridge_owned("signal:promotion"),
                crate::truth_identity_fixtures::truth_snapshot_fixture(
                    "harness:speculation-discard-sibling:snapshot",
                ),
            ),
        )
        .map_err(|error| {
            BridgeHarnessError::new(format!("speculation sibling admission failed: {error}"))
        })?;
    let (discarded_active, discarded_execution_record) =
        runtime_bridge.activate_preview_session(discarded_admitted, 3, 1, 2);
    let preview = crate::facade::BridgeSpeculativeSessionHandle::new(
        runtime_bridge.clone(),
        discarded_active,
        discarded_execution_record,
    );
    let discarded_execution_record = preview.execution_record().clone();
    let outcome = match preview.discard(vec![
        crate::facade::BridgePreviewResidueClass::PreviewExecutionRetained,
        crate::facade::BridgePreviewResidueClass::ReplayRetainedNonAuthoritative,
        crate::facade::BridgePreviewResidueClass::TemporaryRoutingResidue,
        crate::facade::BridgePreviewResidueClass::TemporaryDiagnosticsResidue,
    ]) {
        Ok(outcome) => outcome,
        Err((preview, denial)) => return Err(BridgeHarnessError::preview_refused(preview, denial)),
    };
    let discarded_record = outcome.record().clone();
    let discarded_replay_bundle = runtime_bridge
        .replay_preview_bundle(&discard_sibling_session_identity)
        .map_err(|error| {
            BridgeHarnessError::new(format!("speculation sibling replay failed: {error}"))
        })?;

    let routing_digest =
        shared::first_commit_routing_digest(runtime_bridge, fixture, resource_request)?;
    let diagnostics_digest = digest_string(
        "speculation-diagnostics-digest",
        &format!(
            "promotion={}|discard={}|promoted-replay={}|discarded-replay={}|tier={:?}",
            runtime_bridge
                .diagnostics()
                .explain_preview_promotion_record(&promotion_record)
                .preview_promotion_record_identity(),
            runtime_bridge
                .diagnostics()
                .explain_preview_discard_record(&discarded_record)
                .preview_discard_record_identity(),
            runtime_bridge
                .diagnostics()
                .explain_preview_replay_bundle(&promoted_replay_bundle)
                .replay_bundle_digest(),
            runtime_bridge
                .diagnostics()
                .explain_preview_replay_bundle(&discarded_replay_bundle)
                .replay_bundle_digest(),
            runtime_bridge.policy().diagnostics_tier(),
        ),
    )
    .to_string();

    Ok(SpeculationHarnessExecution::Promotion {
        promoted_execution_record,
        promotion_record,
        promoted_replay_bundle,
        discarded_execution_record,
        discarded_record,
        discarded_replay_bundle,
        routing_digest,
        diagnostics_digest,
    })
}
