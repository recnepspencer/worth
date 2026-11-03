use worth_runtime_bridge::facade::BridgeExecutionBasisReadmissionCounters;

use super::WorthQueryReadmissionCounters;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryReadmissionEvidence {
    query: WorthQueryReadmissionCounters,
    bridge: Option<BridgeExecutionBasisReadmissionCounters>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct WorthQueryReadmissionProgress {
    query: WorthQueryReadmissionCounters,
    bridge: Option<BridgeExecutionBasisReadmissionCounters>,
}

impl WorthQueryReadmissionEvidence {
    pub const fn query_counters(self) -> WorthQueryReadmissionCounters {
        self.query
    }

    pub const fn bridge_counters(self) -> Option<BridgeExecutionBasisReadmissionCounters> {
        self.bridge
    }
}

impl WorthQueryReadmissionProgress {
    pub(super) fn checked_preflight(&mut self) {
        self.query.checked_preflight();
    }

    pub(super) fn minted_fresh_resource_attempt(&mut self) {
        self.query.minted_fresh_resource_attempt();
    }

    pub(super) fn attempted_bridge_readmission(&mut self) {
        self.query.attempted_bridge_readmission();
    }

    pub(super) fn attempted_provider_restore(&mut self) {
        self.query.attempted_provider_restore();
    }

    pub(super) fn attempted_artifact_generation(&mut self) {
        self.query.attempted_artifact_generation();
    }

    pub(super) fn committed_artifact_generation(&mut self) {
        self.query.committed_artifact_generation();
    }

    pub(super) fn committed_attempt(&mut self) {
        self.query.committed_attempt();
    }

    pub(super) const fn observe_bridge(
        &mut self,
        counters: BridgeExecutionBasisReadmissionCounters,
    ) {
        self.bridge = Some(counters);
    }

    pub(super) const fn evidence(self) -> WorthQueryReadmissionEvidence {
        WorthQueryReadmissionEvidence {
            query: self.query,
            bridge: self.bridge,
        }
    }
}
