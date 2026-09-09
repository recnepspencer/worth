use std::sync::Arc;

use super::{
    BridgeConditionalDecisionEvidence, BridgeConditionalDenial, BridgeConditionalDenialKind,
    BridgeConditionalExecutionRequest, BridgeInstalledConditionalLowering, BridgeManagedDueWake,
    BridgeOwnedSignalRuntime,
};

/// Exact managed-wake input for one Bridge-owned conditional evaluation.
/// The Signal wake identity remains private to Bridge.
pub struct BridgeManagedConditionalExecutionRequest<'a> {
    pub due_wake: &'a BridgeManagedDueWake,
    pub lowering: &'a Arc<BridgeInstalledConditionalLowering>,
    pub signal_basis: &'a super::BridgeConditionalSignalBasisBinding,
    pub query_binding_identity: &'a str,
    pub query_capability_identity: u64,
    pub snapshot_identity: &'a str,
    pub truth_branch_identity: Option<&'a str>,
    pub bridge_snapshot_identity: Option<&'a crate::snapshot::TruthSnapshotIdentity>,
    pub triggering_correspondence:
        Option<&'a crate::correspondence::BridgeCorrespondenceDeliveryReceipt>,
    pub attempt: u64,
}

impl BridgeOwnedSignalRuntime {
    pub fn execute_managed_due_wake(
        &self,
        request: BridgeManagedConditionalExecutionRequest<'_>,
        compute_context: &mut dyn std::any::Any,
    ) -> Result<BridgeConditionalDecisionEvidence, BridgeConditionalDenial> {
        self.validate_managed_due_wake(&request)?;
        self.validate_triggering_correspondence(&request)?;
        let retained_trigger = request
            .triggering_correspondence
            .map(|trigger| {
                super::retention::BridgeRetainedTrigger::prepare(
                    &self.retention,
                    trigger.change_set(),
                )
            })
            .transpose()?;
        let execution_identity = format!(
            "managed-wake:{}:revision={}:signal={}:scheduled={}:ready={}",
            request.due_wake.intent_identity().as_str(),
            request.due_wake.revision(),
            request.due_wake.signal_wake_id.get(),
            request.due_wake.signal_scheduled_ordinal(),
            request.due_wake.signal_ready_ordinal(),
        );
        let admission = match request.bridge_snapshot_identity {
            Some(identity) => {
                super::BridgeConditionalEvaluationAdmissionRequest::source_present_at_signal_basis(
                    request.signal_basis,
                    identity,
                )
            }
            None => {
                super::BridgeConditionalEvaluationAdmissionRequest::source_free_at_signal_basis(
                    request.signal_basis,
                )
            }
        };
        let mut session = self.admit_conditional_evaluation_with_record(
            admission,
            Some(request.due_wake.source_record_identity()),
        )?;
        session.observation_baselines = Arc::clone(&request.due_wake.observation_baselines);
        let mut evidence = self.execute_admitted_conditional(
            &session,
            BridgeConditionalExecutionRequest {
                lowering: request.lowering,
                query_binding_identity: request.query_binding_identity,
                query_capability_identity: request.query_capability_identity,
                snapshot_identity: request.snapshot_identity,
                truth_branch_identity: request.truth_branch_identity,
                bridge_snapshot_identity: request.bridge_snapshot_identity,
                execution_identity: &execution_identity,
                attempt: request.attempt,
            },
            compute_context,
        )?;
        if let Some(trigger) = retained_trigger {
            std::sync::Arc::get_mut(&mut evidence.core)
                .expect("new managed decision core is uniquely owned")
                .triggering_change_set = Some(trigger);
        }
        Ok(evidence)
    }

    fn validate_managed_due_wake(
        &self,
        request: &BridgeManagedConditionalExecutionRequest<'_>,
    ) -> Result<(), BridgeConditionalDenial> {
        let lane = self
            .managed_clock_lanes
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(request.due_wake.binding_identity())
            .cloned();
        let retained = match lane {
            Some(lane) => super::managed_time::lock_lane(&lane)
                .map_err(|denial| {
                    BridgeConditionalDenial::new(
                        BridgeConditionalDenialKind::ManagedClockQuarantined,
                        denial.detail(),
                    )
                })?
                .retains_due_wake(request.lowering, request.due_wake),
            None => false,
        };
        if retained {
            Ok(())
        } else {
            Err(BridgeConditionalDenial::new(
                BridgeConditionalDenialKind::ManagedWakeMismatch,
                "managed due wake lost its exact clock, intent, or conditional lowering affinity",
            ))
        }
    }

    fn validate_triggering_correspondence(
        &self,
        request: &BridgeManagedConditionalExecutionRequest<'_>,
    ) -> Result<(), BridgeConditionalDenial> {
        let Some(trigger) = request.triggering_correspondence else {
            return Ok(());
        };
        let change_set = trigger.change_set();
        let dependency = change_set.dependency();
        let installed = request
            .lowering
            .correspondences
            .iter()
            .any(|correspondence| correspondence.dependency() == dependency);
        let record = request.due_wake.source_record_identity();
        let touches_wake = change_set
            .changes()
            .iter()
            .any(|change| change.relational_record_identity() == Some(record));
        if installed && touches_wake {
            return Ok(());
        }
        Err(BridgeConditionalDenial::new(
            BridgeConditionalDenialKind::ManagedWakeMismatch,
            "triggering correspondence does not match the installed dependency and due record",
        ))
    }
}
