//! Native-owned candidate Source interpretation and its retained materialization basis.
mod context;
mod correspondence;
pub(crate) mod denial;
mod interpretation;
pub(crate) mod opaque_custody;
pub use context::{
    ManagedIndexSourceContextFactory, ManagedIndexSourceContextReceipt,
    ManagedIndexSourceContextShare, ManagedIndexSourcePreparationMetadata,
};
pub use interpretation::NativeSourceInterpretationIdentity;
pub(crate) mod program;
pub use denial::ManagedIndexSourcePreparationDenial;
pub(crate) use program::SourceProgramCommitmentDenial;
mod observation;
mod preparation;
mod preparation_group;
pub(crate) use preparation_group::SourcePreparationGroup;
mod projection;
pub(crate) use projection::selected_rows;
pub(crate) mod provider_failure;
pub use provider_failure::ManagedIndexSourceProviderFailure;
mod publication;
pub use publication::{ManagedIndexSourcePublication, ManagedIndexSourcePublicationRoot};
pub(crate) mod image;
pub(crate) use image::ManagedIndexSourceAssociation;
pub use image::ManagedIndexSourceCacheRetirement;
pub(crate) use image::PublicCacheEntry;
pub use image::{ManagedIndexBranchImage, ManagedIndexPublishedSourceRoot};
mod scope_closure;
mod source_changes;
pub use source_changes::ManagedIndexSourceChanges;
mod staged;
pub(crate) use observation::{
    prepare as prepare_observed_root, read as read_observed_root, scopes as observed_scopes,
};
pub use observation::{ManagedIndexSourceRead, PreparedManagedIndexSourceRead};
pub(crate) use preparation::SourceViews;
mod applied;
mod applied_read;
pub use applied_read::{ManagedIndexAppliedSourcePublication, ManagedIndexAppliedSourceRead};
pub(crate) mod authored_history;
pub(crate) mod routing;
pub use preparation::{
    ManagedIndexSourceAvailability, ManagedIndexSourceOccurrence, ManagedIndexSourcePreparation,
    ManagedIndexSourceProjectionOccurrence, ManagedIndexSourceRows, ManagedIndexSourceScope,
    PreparedManagedIndexProjectionRead, PreparedManagedIndexSourceOccurrence,
    PreparedManagedIndexSourceRoot, PreparedManagedIndexSourceRowRead,
};
pub use program::{
    ManagedIndexSourceField, ManagedIndexSourceProgram, ManagedIndexSourceProjectionRoute,
};
pub use projection::{ManagedIndexSourceProjection, PreparedManagedIndexRelationRead};

pub(crate) use interpretation::{
    NativeManagedSourceInterpretation, NativeMaterializationSource, NativeSourceRestorationOrigin,
};
