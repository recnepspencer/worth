//! Output settlement posture classification.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryOutputSettlementPosture {
    Performed,
    StableReused,
    RecoveredPerformed,
    RecoveredStableReused,
    HistoricallySelected,
}
