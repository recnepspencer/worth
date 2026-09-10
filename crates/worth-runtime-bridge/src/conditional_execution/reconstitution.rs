/// Reconstructive evidence for derived Bridge state readmitted under the same
/// sealed Signal owner and definition. This report grants no execution authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BridgeConditionalRuntimeReconstitutionReport {
    signal: worth_signal::facade::branch::SignalConditionalReconstitutionReport,
    correspondence: crate::correspondence::BridgeCorrespondenceRebuildReport,
    signal_graph_instance_id: u64,
    readmitted_lowering_count: usize,
}

impl BridgeConditionalRuntimeReconstitutionReport {
    pub(crate) const fn new(
        signal: worth_signal::facade::branch::SignalConditionalReconstitutionReport,
        correspondence: crate::correspondence::BridgeCorrespondenceRebuildReport,
        signal_graph_instance_id: u64,
        readmitted_lowering_count: usize,
    ) -> Self {
        Self {
            signal,
            correspondence,
            signal_graph_instance_id,
            readmitted_lowering_count,
        }
    }

    pub const fn signal(
        self,
    ) -> worth_signal::facade::branch::SignalConditionalReconstitutionReport {
        self.signal
    }
    pub const fn correspondence(self) -> crate::correspondence::BridgeCorrespondenceRebuildReport {
        self.correspondence
    }
    pub const fn signal_graph_instance_id(self) -> u64 {
        self.signal_graph_instance_id
    }
    pub const fn readmitted_lowering_count(self) -> usize {
        self.readmitted_lowering_count
    }
}
