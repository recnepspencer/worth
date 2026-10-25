mod access;
mod authority;
pub mod bundle;
mod canonical_history;
pub mod cdc;
pub mod data;
pub mod patch;

pub(crate) use access::invariant_failure_diagnostic;
pub use access::{
    PublicationArtifactsAccess, PublicationDiagnosticsAccess, PublicationPatchStreamAccess,
    PublicationSubscriberStreamAccess, PublicationSurface,
};
pub(crate) use authority::production_post_commit_consumer;
pub use authority::PublicationAuthority;
pub(crate) use authority::PublicationPreparationAuthority;
pub use authority::RelationalSettlementPort;
pub use authority::{
    PostCommitConsumer, PostCommitConsumptionContext, PostCommitConsumptionFailure,
};
pub(crate) use canonical_history::{
    durable_canonical_envelopes, retained_canonical_envelope_at_position,
    retained_canonical_envelopes_after,
};
