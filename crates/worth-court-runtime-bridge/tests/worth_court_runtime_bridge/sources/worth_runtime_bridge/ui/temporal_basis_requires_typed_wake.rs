use worth_signal::facade::{ClockAdvanceOrdinal, ClockDomain, ClockTick};
use worth_runtime_bridge::facade::{
    AdmittedBridgeTemporalBasis, BridgeTemporalSignalBasis, BridgeTemporalTruthViewBasis,
    TruthBranchIdentity, TruthCommitIdentity, TruthSnapshotIdentity,
};

fn main() {
    let truth_basis = BridgeTemporalTruthViewBasis::authoritative(
        TruthBranchIdentity::new("branch-a"),
        TruthCommitIdentity::new("commit-a"),
        TruthSnapshotIdentity::new("snapshot-a"),
    );
    let signal_basis = BridgeTemporalSignalBasis::new(
        TruthBranchIdentity::new("branch-a"),
        ClockDomain::MonotonicExecution,
        ClockTick::new(1),
        ClockAdvanceOrdinal::new(1),
        None,
    );

    let _ = AdmittedBridgeTemporalBasis::admit(
        truth_basis,
        signal_basis,
        Some("wake-a".to_string()),
    );
}
