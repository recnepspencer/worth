mod catalog;
mod commit;
mod parentage;
mod publication;
mod reclamation;
mod retention;

pub(crate) use catalog::{
    CompositeHistoryCatalog, ReservedCompositeCommitCapacity, RuntimeWorldHistoryCatalogContract,
};
pub use catalog::{CompositeHistoryCatalogDenial, CompositeHistoryTraversal};
#[allow(
    unused_imports,
    reason = "the internal denial is asserted by the real constructor contract test"
)]
pub(crate) use commit::CompositeCommitConstructionDenial;
pub use commit::{
    CompositeCallerCorrelation, CompositeCommitParent, CompositeCommitProvenance,
    CompositeComponentChangePosture, CompositeRuntimeWorldCommit,
    CompositeSignalPublicationIdentity,
};
pub use parentage::OrdinaryParent;
pub(crate) use publication::{
    CanonicalPublicationEnvelope, PerformedPublicationFacts, PreparedPublicationRecord,
    PublicationDeliveryClaim,
};
pub use reclamation::{
    CompositeHistoryReclamationRequest, HistoryReclamationDenial, HistoryReclamationOutcome,
};
pub(crate) use retention::{
    ExplicitCommitHistoryProtectionObligation, ProductHeadHistoryProtectionObligation,
};

pub use catalog::{HistoryCatalogCounters, HistoryMetadataLedger};
