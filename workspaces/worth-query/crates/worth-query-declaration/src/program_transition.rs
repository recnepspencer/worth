//! Pure declaration contracts for declarative Home program transitions and
//! deterministic package-default build identities.
//!
//! These declarations describe versioned schema/program evolution without
//! granting installation, execution, or physical storage authority.

mod build_identity;
mod denial;
#[macro_use]
mod macros;
mod sha1;
mod spec;

pub use build_identity::{
    WorthQueryApplicationHomeBuildIdentity, WORTH_PACKAGE_BUILD_ID_NAMESPACE,
};
pub use denial::WorthQueryProgramTransitionDeclarationDenial;
pub use spec::{
    WorthQueryProgramTransitionAddedRecord, WorthQueryProgramTransitionDescriptor,
    WorthQueryProgramTransitionDescriptorParts, WorthQueryProgramTransitionFieldDefault,
    WorthQueryProgramTransitionIndexRebuild, WorthQueryProgramTransitionRetainedRecord,
    WorthQueryProgramTransitionSpec,
};
