//! Pure declaration contracts for declarative Home program transitions and
//! deterministic package-default build identities.
//!
//! These declarations describe versioned schema/program evolution without
//! granting installation, execution, or physical storage authority.

mod build_identity;
#[macro_use]
mod macros;
mod spec;

pub use build_identity::WorthQueryApplicationHomeBuildIdentity;
pub use spec::{
    WorthQueryProgramTransitionAddedRecord, WorthQueryProgramTransitionDeclarationDenial,
    WorthQueryProgramTransitionDescriptor, WorthQueryProgramTransitionDescriptorParts,
    WorthQueryProgramTransitionFieldDefault, WorthQueryProgramTransitionIndexRebuild,
    WorthQueryProgramTransitionRetainedRecord, WorthQueryProgramTransitionSpec,
};
