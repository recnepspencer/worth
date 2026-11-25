//! Declarative Home program transition contracts, macros, and deterministic
//! package-default build identity helpers.
//!
//! Consumers import these from `worth_query_decl::facade::program_transition`
//! (or `worth_query_decl::program_transition`) and compile them onto WORTH's
//! real `WorthQueryHomeProgramTransition` through
//! `worth_query_host::facade::program_transition`.

pub use worth_query_declaration::facade::program_transition::{
    WorthQueryApplicationHomeBuildIdentity, WorthQueryProgramTransitionAddedRecord,
    WorthQueryProgramTransitionDeclarationDenial, WorthQueryProgramTransitionDescriptor,
    WorthQueryProgramTransitionDescriptorParts, WorthQueryProgramTransitionFieldDefault,
    WorthQueryProgramTransitionIndexRebuild, WorthQueryProgramTransitionRetainedRecord,
    WorthQueryProgramTransitionSpec,
};
pub use worth_query_declaration::{worth_query_default_build_id, worth_query_program_transition};
