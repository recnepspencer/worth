//! Native-owned candidate Source interpretation and its retained materialization basis.
mod correspondence;
mod denial;
mod interpretation;
mod program;
pub use denial::ManagedIndexSourcePreparationDenial;
pub(crate) use program::SourceBindingCache;
mod observation;
mod preparation;
mod projection;
mod provider_failure;
pub use provider_failure::ManagedIndexSourceProviderFailure;
mod publication;
pub use publication::{ManagedIndexSourcePublication, ManagedIndexSourcePublicationRoot};
mod image;
pub(crate) use image::ManagedIndexSourceAssociation;
pub use image::ManagedIndexSourceCacheRetirement;
pub(crate) use image::PublicCacheEntry;
pub use image::{ManagedIndexBranchImage, ManagedIndexPublishedSourceRoot};
mod scope_closure;
mod staged;
pub(crate) use observation::{
    prepare as prepare_observed_root, read as read_observed_root, scopes as observed_scopes,
};
pub use observation::{ManagedIndexSourceRead, PreparedManagedIndexSourceRead};
pub(crate) use preparation::SourceViews;
mod applied;
pub(crate) mod routing;
pub use preparation::{
    ManagedIndexSourceAvailability, ManagedIndexSourceOccurrence, ManagedIndexSourcePreparation,
    ManagedIndexSourceProjectionOccurrence, ManagedIndexSourceRows, ManagedIndexSourceScope,
    PreparedManagedIndexProjectionRead, PreparedManagedIndexSourceRoot,
};
pub use program::{
    ManagedIndexSourceField, ManagedIndexSourceProgram, ManagedIndexSourceProjectionRoute,
};
pub use projection::{ManagedIndexSourceProjection, PreparedManagedIndexRelationRead};

pub(crate) use interpretation::{
    NativeManagedSourceInterpretation, NativeMaterializationSource,
    NativeSourceInterpretationIdentity, NativeSourceRestorationOrigin,
};
