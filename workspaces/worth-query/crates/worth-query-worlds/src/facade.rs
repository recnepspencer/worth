//! Worlds shared by the workspace courts; fixtures carry real platform boundaries.
#[allow(
    ambiguous_glob_reexports,
    hidden_glob_reexports,
    reason = "Fixture namespaces retain authored lexical imports; courts exercise concrete world definitions through these scopes."
)]
#[path = "certification/mod.rs"]
pub mod certification;
#[allow(
    ambiguous_glob_reexports,
    hidden_glob_reexports,
    reason = "Fixture namespaces retain authored lexical imports; courts exercise concrete world definitions through these scopes."
)]
#[path = "decl/mod.rs"]
pub mod decl;
#[allow(
    ambiguous_glob_reexports,
    hidden_glob_reexports,
    reason = "Fixture namespaces retain authored lexical imports; courts exercise concrete world definitions through these scopes."
)]
#[path = "host/mod.rs"]
pub mod host;
#[allow(
    ambiguous_glob_reexports,
    hidden_glob_reexports,
    reason = "Fixture namespaces retain authored lexical imports; courts exercise concrete world definitions through these scopes."
)]
#[path = "installation/mod.rs"]
pub mod installation;
#[allow(
    ambiguous_glob_reexports,
    hidden_glob_reexports,
    reason = "Fixture namespaces retain authored lexical imports; courts exercise concrete world definitions through these scopes."
)]
#[path = "package_archive/mod.rs"]
pub mod package_archive;
#[allow(
    ambiguous_glob_reexports,
    hidden_glob_reexports,
    reason = "Fixture namespaces retain authored lexical imports; courts exercise concrete world definitions through these scopes."
)]
#[path = "publication/mod.rs"]
pub mod publication;
#[allow(
    ambiguous_glob_reexports,
    hidden_glob_reexports,
    reason = "Fixture namespaces retain authored lexical imports; courts exercise concrete world definitions through these scopes."
)]
#[path = "query/mod.rs"]
pub mod query;
// The shared owner-unit oracle resolves its existing facade path here.
pub(crate) use worth_query_execution::facade::primary_graph;
