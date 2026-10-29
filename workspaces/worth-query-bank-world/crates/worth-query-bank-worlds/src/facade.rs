//! Worlds shared by the workspace courts; fixtures carry real platform boundaries.
#[allow(
    ambiguous_glob_reexports,
    hidden_glob_reexports,
    reason = "Fixture namespaces retain authored lexical imports; courts exercise concrete world definitions through these scopes."
)]
#[path = "binary_path.rs"]
mod binary_path;
#[allow(
    ambiguous_glob_reexports,
    hidden_glob_reexports,
    reason = "Fixture namespaces retain authored lexical imports; courts exercise concrete world definitions through these scopes."
)]
#[path = "courtroom/mod.rs"]
pub mod courtroom;
#[allow(
    ambiguous_glob_reexports,
    hidden_glob_reexports,
    reason = "Fixture namespaces retain authored lexical imports; courts exercise concrete world definitions through these scopes."
)]
#[path = "domain/mod.rs"]
pub mod domain;
#[allow(
    ambiguous_glob_reexports,
    hidden_glob_reexports,
    reason = "Fixture namespaces retain authored lexical imports; courts exercise concrete world definitions through these scopes."
)]
#[path = "estate_certification/mod.rs"]
pub mod estate_certification;
#[allow(
    ambiguous_glob_reexports,
    hidden_glob_reexports,
    reason = "Fixture namespaces retain authored lexical imports; courts exercise concrete world definitions through these scopes."
)]
#[path = "external_rail/mod.rs"]
pub mod external_rail;
#[allow(
    ambiguous_glob_reexports,
    hidden_glob_reexports,
    reason = "Fixture namespaces retain authored lexical imports; courts exercise concrete world definitions through these scopes."
)]
#[path = "http_adapter/mod.rs"]
pub mod http_adapter;
#[allow(
    ambiguous_glob_reexports,
    hidden_glob_reexports,
    reason = "Fixture namespaces retain authored lexical imports; courts exercise concrete world definitions through these scopes."
)]
#[path = "server/mod.rs"]
pub mod server;
#[allow(
    ambiguous_glob_reexports,
    hidden_glob_reexports,
    reason = "Fixture namespaces retain authored lexical imports; courts exercise concrete world definitions through these scopes."
)]
#[path = "user_node/mod.rs"]
pub mod user_node;
pub use binary_path::binary_path;
