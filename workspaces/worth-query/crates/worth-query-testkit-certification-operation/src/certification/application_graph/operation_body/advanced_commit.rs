//! Even lawful operation admission cannot replace an authored body's values.
pub use super::{
    authenticate_operator, binding::BodyBinding, install_with_projection, read_retention,
    request_scope, schema::*,
};
pub use worth_query_host::facade::{
    application_installation::WorthQueryProgramOwner,
    declaration::application_operation::ApplicationMutationIdentities,
    primary_graph::{
        WorthQueryApplicationCommitDenialKind, WorthQueryApplicationCommitOutcome,
        WorthQueryPrincipalResolutionMode,
    },
    runtime::ExecutionAllocationPolicy,
};
