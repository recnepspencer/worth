#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublicationCompanionRegistrationStop {
    OwnerUnavailable,
    PublicationPending,
    /// A publication holds its companion epoch; head-cell installation did not run.
    HeadCellPublicationContended,
    RebindRequired,
    Superseded,
    IdentityExhausted,
    ForeignRuntime,
    HeadUnavailable,
}
