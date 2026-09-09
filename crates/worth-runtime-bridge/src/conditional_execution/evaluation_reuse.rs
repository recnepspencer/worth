use super::{
    BridgeConditionalDenial, BridgeConditionalDenialKind, BridgeConditionalEvaluationSession,
    BridgeOwnedSignalRuntime,
};

pub struct BridgeConditionalEvaluationReadmissionRequest<'a> {
    pub predecessor: &'a BridgeConditionalEvaluationSession,
    pub transitions: &'a [&'a crate::correspondence::BridgeCorrespondenceDeliveryReceipt],
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BridgeConditionalEvaluationReadmissionCounters {
    transitions_checked: usize,
    targets_applied: usize,
    persistent_roots_forked: usize,
    source_opens: usize,
    unrelated_lowering_scans: usize,
}

impl BridgeConditionalEvaluationReadmissionCounters {
    pub const fn transitions_checked(self) -> usize {
        self.transitions_checked
    }

    pub const fn targets_applied(self) -> usize {
        self.targets_applied
    }

    pub const fn persistent_roots_forked(self) -> usize {
        self.persistent_roots_forked
    }

    pub const fn source_opens(self) -> usize {
        self.source_opens
    }

    pub const fn unrelated_lowering_scans(self) -> usize {
        self.unrelated_lowering_scans
    }
}

impl BridgeOwnedSignalRuntime {
    pub fn readmit_conditional_evaluation(
        &self,
        request: BridgeConditionalEvaluationReadmissionRequest<'_>,
    ) -> Result<BridgeConditionalEvaluationSession, BridgeConditionalDenial> {
        if request.predecessor.bridge_runtime_key != self.bridge.signal_runtime_key {
            return Err(BridgeConditionalDenial::new(
                BridgeConditionalDenialKind::OperationAuthorityMismatch,
                "conditional predecessor belongs to another Bridge runtime",
            ));
        }
        self.require_live_installed_lowering(&request.predecessor.lowering)?;
        let mut signal_transitions = Vec::with_capacity(request.transitions.len());
        for receipt in request.transitions {
            let transition = receipt.conditional_transition().ok_or_else(|| {
                BridgeConditionalDenial::new(
                    BridgeConditionalDenialKind::ConditionalTransitionChainMismatch,
                    "delivery receipt does not retain a conditional successor transition",
                )
            })?;
            signal_transitions.push(transition);
        }
        let completion = request
            .predecessor
            .signal_port
            .readmit_evaluation(
                worth_signal::facade::branch::SignalConditionalEvaluationReadmissionRequest {
                    predecessor: &request.predecessor.signal,
                    transitions: &signal_transitions,
                },
            )
            .map_err(map_signal_readmission_denial)?;
        let (signal, counters) = completion.into_parts();
        let counters = BridgeConditionalEvaluationReadmissionCounters {
            transitions_checked: counters.transitions_checked(),
            targets_applied: counters.targets_applied(),
            persistent_roots_forked: counters.persistent_roots_forked(),
            source_opens: counters.source_opens(),
            unrelated_lowering_scans: counters.unrelated_lowering_scans(),
        };
        Ok(BridgeConditionalEvaluationSession {
            bridge_runtime_key: request.predecessor.bridge_runtime_key,
            lowering: std::sync::Arc::clone(&request.predecessor.lowering),
            source_snapshot: request
                .predecessor
                .source_snapshot
                .as_ref()
                .map(std::sync::Arc::clone),
            signal,
            signal_port: request.predecessor.signal_port.clone(),
            snapshot_admission_attempts: 0,
            managed_source_record: request.predecessor.managed_source_record,
            observation_baselines: std::sync::Arc::clone(
                &request.predecessor.observation_baselines,
            ),
            readmission_counters: Some(counters),
        })
    }
}

fn map_signal_readmission_denial(
    denial: worth_signal::facade::branch::SignalConditionalEvaluationReadmissionDenial,
) -> BridgeConditionalDenial {
    use worth_signal::facade::branch::SignalConditionalEvaluationReadmissionDenial as Denial;
    let kind = match denial {
        Denial::PredecessorNotExecuted => {
            BridgeConditionalDenialKind::ConditionalPredecessorNotExecuted
        }
        Denial::TransitionChainIncomplete => {
            BridgeConditionalDenialKind::ConditionalTransitionChainIncomplete
        }
        Denial::TransitionChainMismatch | Denial::DefinitionMismatch => {
            BridgeConditionalDenialKind::ConditionalTransitionChainMismatch
        }
        Denial::SlotBusy => BridgeConditionalDenialKind::ConditionalEvaluationBusy,
        Denial::SlotPoisoned => BridgeConditionalDenialKind::ConditionalEvaluationPoisoned,
        Denial::UnconsumedUnwind => BridgeConditionalDenialKind::ConditionalEvaluationUnwindPending,
        Denial::AdmissionCapacityExhausted => {
            BridgeConditionalDenialKind::ConditionalEvaluationAdmissionCapacity
        }
        Denial::StaleBasisAdmission | Denial::DefinitionReadmissionRequired => {
            BridgeConditionalDenialKind::StaleLowering
        }
        Denial::OwnerUnavailable(_)
        | Denial::OwnerAdmission(_)
        | Denial::EvaluationIdentityExhausted
        | Denial::AdmissionUnavailable
        | Denial::SlotAdmission(_) => BridgeConditionalDenialKind::SignalExecution,
    };
    BridgeConditionalDenial::new(
        kind,
        format!("Signal conditional successor readmission was denied: {denial:?}"),
    )
}
