//! Native-owned candidate Source interpretation and its retained materialization basis.
pub use availability::ManagedIndexSourceAvailability;
pub(crate) use worth_relational_contracts::internal_port::validation::candidate_source::availability;
mod context;
mod correspondence;
pub(crate) use worth_relational_contracts::internal_port::validation::candidate_source::denial;
mod interpretation;
mod interpretation_identity;
pub(crate) use worth_relational_branch::internal_port::validation::candidate_source::opaque_custody;
pub(crate) mod registration_observation;
pub use context::{
    ManagedIndexSourceContextFactory, ManagedIndexSourceContextReceipt,
    ManagedIndexSourceContextShare, ManagedIndexSourcePreparationMetadata,
};
pub use denial::ManagedIndexSourcePreparationDenial;
pub use interpretation_identity::NativeSourceInterpretationIdentity;
pub(crate) use program::SourceProgramCommitmentDenial;
pub(crate) use worth_relational_contracts::internal_port::validation::candidate_source::program;
mod observation;
mod preparation;
mod preparation_group;
pub(crate) use preparation_group::SourcePreparationGroup;
mod projection;
pub(crate) use projection::selected_rows;
pub use provider_failure::ManagedIndexSourceProviderFailure;
pub(crate) use worth_relational_contracts::internal_port::validation::candidate_source::provider_failure;
pub(crate) mod publication;
pub use publication::{ManagedIndexSourcePublication, ManagedIndexSourcePublicationRoot};
pub(crate) mod history_material;
pub(crate) mod image;
pub use image::ManagedIndexSourceCacheRetirement;
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
pub(crate) mod applied;
mod applied_affinity;
pub(crate) mod applied_read;
pub(crate) mod authored_history;
pub(crate) mod routing;
pub use preparation::{
    ManagedIndexSourceOccurrence, ManagedIndexSourcePreparation,
    ManagedIndexSourceProjectionOccurrence, ManagedIndexSourceRows, ManagedIndexSourceScope,
    PreparedManagedIndexProjectionRead, PreparedManagedIndexSourceOccurrence,
    PreparedManagedIndexSourceRoot, PreparedManagedIndexSourceRowRead,
};
pub use program::{
    ManagedIndexSourceField, ManagedIndexSourceProgram, ManagedIndexSourceProjectionRoute,
};
pub use projection::{ManagedIndexSourceProjection, PreparedManagedIndexRelationRead};

pub(crate) use interpretation::{
    ManagedSourceIssueInputs, NativeManagedSourceInterpretation, NativeMaterializationSource,
    NativeSourceRestorationOrigin,
};
