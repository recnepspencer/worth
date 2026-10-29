//! Native-owned candidate Source interpretation and its retained materialization basis.
mod denial;
mod interpretation;
mod program;
pub use denial::ManagedIndexSourcePreparationDenial;
mod observation;
mod preparation;
mod projection;
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
    ManagedIndexSourceRows, ManagedIndexSourceScope, PreparedManagedIndexSourceRoot,
};
pub use program::{ManagedIndexSourceField, ManagedIndexSourceProgram};
pub use projection::ManagedIndexSourceProjection;

pub(crate) use interpretation::{
    NativeManagedSourceInterpretation, NativeMaterializationSource,
    NativeSourceInterpretationIdentity,
};
