/// Why retained history cannot admit the requested branch/version visibility basis.
/// These refusals carry no replay or reconstruction authority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum HistoricalVisibilityDenial {
    UnknownVersion,
    AuthoringBranchUnavailable,
    MvccIntervalUnavailable,
    CertificationReconstructionRequired,
    RetentionUnavailable,
    RetentionCapacityExhausted,
    RetentionIdentityExhausted,
}
