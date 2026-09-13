#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PerformedPublicationRecoveryDenial {
    OwnerUnavailable(crate::lifecycle::RuntimeWorldOwnerUnavailable),
    Catalog(crate::history::CompositeHistoryCatalogDenial),
    MissingCommit,
    NotPerformed,
    Claimed,
    Consumed,
    ProtectionMismatch,
}
impl From<crate::history::CompositeHistoryCatalogDenial> for PerformedPublicationRecoveryDenial {
    fn from(value: crate::history::CompositeHistoryCatalogDenial) -> Self {
        Self::Catalog(value)
    }
}
