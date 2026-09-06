/// Truthful denial from the owning recovery catalog or its lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeWorldRecoveryDenial {
    OwnerUnavailable(crate::lifecycle::RuntimeWorldOwnerUnavailable),
    ForeignHandle,
    MissingRecord,
    Busy,
    TooYoung,
    ClockRegressed,
    CallerCapabilityLive,
    SettlementRequired,
    SettlementEvidenceUnavailable,
}
impl From<crate::lifecycle::RuntimeWorldOwnerUnavailable> for RuntimeWorldRecoveryDenial {
    fn from(value: crate::lifecycle::RuntimeWorldOwnerUnavailable) -> Self {
        Self::OwnerUnavailable(value)
    }
}
