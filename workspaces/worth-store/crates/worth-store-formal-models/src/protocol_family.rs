#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProtocolFamily {
    DurabilityRecovery,
    RecoverySourcePrecedence,
    CompactionVisibility,
    LeaseReclaim,
    QuarantineReadmission,
    ImportPublication,
    ReplicationAdmission,
    SharedFrontiers,
}

impl ProtocolFamily {
    pub const fn all() -> [Self; 8] {
        [
            Self::DurabilityRecovery,
            Self::RecoverySourcePrecedence,
            Self::CompactionVisibility,
            Self::LeaseReclaim,
            Self::QuarantineReadmission,
            Self::ImportPublication,
            Self::ReplicationAdmission,
            Self::SharedFrontiers,
        ]
    }
}
