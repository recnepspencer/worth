mod access;
pub(crate) mod authority;
pub mod bundle;
pub(crate) mod canonical_history;
pub mod cdc;
pub(crate) mod close_custody;
pub mod data;
pub(crate) mod diagnostic_store;
pub(crate) mod live_publication_inputs;
pub(crate) mod ownership;
pub mod patch;

pub(crate) use access::invariant_failure_diagnostic;
pub use access::{
    PublicationArtifactsAccess, PublicationDiagnosticsAccess, PublicationPatchStreamAccess,
    PublicationSubscriberStreamAccess, PublicationSurface,
};
pub(crate) use authority::production_post_commit_consumer;
pub use authority::PublicationAuthority;
pub(crate) use authority::PublicationPreparationAuthority;
pub use authority::{
    PostCommitConsumer, PostCommitConsumptionContext, PostCommitConsumptionFailure,
};
pub(crate) use canonical_history::{
    durable_canonical_envelopes, retained_canonical_envelope_at_position,
    retained_canonical_envelopes_after,
};
