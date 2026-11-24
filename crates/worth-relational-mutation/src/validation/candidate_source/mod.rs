//! Native-owned candidate Source interpretation and its retained materialization basis.
pub use crate::contracts::validation::candidate_source::availability::ManagedIndexSourceAvailability;
pub mod context;
pub mod correspondence;
pub mod interpretation;
pub mod interpretation_identity;
pub mod registration_observation;
pub use crate::contracts::validation::candidate_source::denial::ManagedIndexSourcePreparationDenial;
pub use context::{
    ManagedIndexSourceContextFactory, ManagedIndexSourceContextReceipt,
    ManagedIndexSourceContextShare, ManagedIndexSourcePreparationMetadata,
};
pub use interpretation_identity::NativeSourceInterpretationIdentity;
pub mod observation;
pub mod preparation;
pub mod preparation_group;
pub use preparation_group::SourcePreparationGroup;
pub mod projection;
pub use crate::contracts::validation::candidate_source::provider_failure::ManagedIndexSourceProviderFailure;
pub use projection::selected_rows;
pub mod publication;
pub use publication::{ManagedIndexSourcePublication, ManagedIndexSourcePublicationRoot};
pub mod history_material;
pub mod image;
pub use image::ManagedIndexSourceCacheRetirement;
pub use image::{ManagedIndexBranchImage, ManagedIndexPublishedSourceRoot};
pub mod scope_closure;
pub mod source_changes;
pub use source_changes::ManagedIndexSourceChanges;
pub mod staged;
pub(crate) use observation::{
    prepare as prepare_observed_root, read as read_observed_root, scopes as observed_scopes,
};
pub use observation::{ManagedIndexSourceRead, PreparedManagedIndexSourceRead};
pub use preparation::SourceViews;
pub mod applied;
pub mod applied_affinity;

pub mod authored_history;
pub mod routing;
pub use crate::contracts::validation::candidate_source::program::projection_routes::ManagedIndexSourceProjectionRoute;
pub use crate::contracts::validation::candidate_source::program::{
    ManagedIndexSourceField, ManagedIndexSourceProgram,
};
pub use preparation::{
    ManagedIndexSourceOccurrence, ManagedIndexSourcePreparation,
    ManagedIndexSourceProjectionOccurrence, ManagedIndexSourceRows, ManagedIndexSourceScope,
    PreparedManagedIndexProjectionRead, PreparedManagedIndexSourceOccurrence,
    PreparedManagedIndexSourceRoot, PreparedManagedIndexSourceRowRead,
};
pub use projection::{ManagedIndexSourceProjection, PreparedManagedIndexRelationRead};

pub(crate) use interpretation::{
    NativeManagedSourceInterpretation, NativeMaterializationSource, NativeSourceRestorationOrigin,
};
