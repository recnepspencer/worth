//! Progress of the original attached-Home root bootstrap.
use super::{PerformedRuntimeWorldBootstrap, RuntimeWorldBootstrapNoEffectCause};
#[derive(Debug)]
pub enum WorldDurableBootstrapRefusal {
    NotHomeBound,
    NoPendingBootstrap,
    Preparation(RuntimeWorldBootstrapNoEffectCause),
    Native(worth_relational::facade::branch::RelationalRecordedComponentRefusal),
    SignalInitialRoot(worth_signal::facade::branch::SignalInitialEmptyRootRefusal),
    BridgeBaseline(worth_runtime_bridge::facade::RuntimeWorldBaselineRefusal),
    Medium(crate::durability::WorldDurableMediumDenial),
}
#[derive(Debug)]
pub enum WorldDurableBootstrapOutcome {
    Performed(PerformedRuntimeWorldBootstrap),
    Pending,
    EvidenceRequired,
    Indeterminate,
    Refused(WorldDurableBootstrapRefusal),
    /// Selection is durable; the same pre-install bundle remains retained.
    RecoveryRequired(WorldDurableBootstrapRefusal),
}
