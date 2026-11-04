//! Source-scope ownership crosses the same real codec-transition fixture.
//! These baseline cases preserve native source identity and decoded values.
//! Genuine codec adoption still requires an independent full stored authored
//! field-byte/carrier oracle, alongside identity and value preservation.
#[path = "canonical_value/binding.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "this fixture compiles the authored grammar and declarations under its distinct transition schema"
)]
pub mod binding;
#[path = "canonical_transition/capture_child.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "the source-owned transition installs the same fixture vocabulary under distinct schema types"
)]
pub mod capture_child;
#[path = "canonical_transition/derived_query.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "the source-owned transition installs the same fixture vocabulary under distinct schema types"
)]
pub mod derived_query;
#[path = "canonical_transition/edit.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "the source-owned transition installs the same fixture vocabulary under distinct schema types"
)]
pub mod edit;
#[path = "canonical_transition/host.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "the source-owned transition installs the same fixture vocabulary under distinct schema types"
)]
pub mod host;
#[path = "canonical_value/model.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "this fixture compiles the authored grammar and declarations under its distinct transition schema"
)]
pub mod model;
#[path = "canonical_transition/output.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "the source-owned transition installs the same fixture vocabulary under distinct schema types"
)]
pub mod output;
#[path = "canonical_transition/producer.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "the source-owned transition installs the same fixture vocabulary under distinct schema types"
)]
pub mod producer;
#[path = "canonical_transition/program.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "the source-owned transition installs the same fixture vocabulary under distinct schema types"
)]
pub mod program;
#[path = "canonical_value/query.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "this fixture compiles the authored grammar and declarations under its distinct transition schema"
)]
pub mod query;
#[path = "canonical_transition/readiness.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "the source-owned transition installs the same fixture vocabulary under distinct schema types"
)]
pub mod readiness;
#[path = "canonical_transition/schema.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "the source-owned transition installs the same fixture vocabulary under distinct schema types"
)]
pub mod schema;
#[path = "canonical_transition/successor.rs"]
#[expect(
    clippy::duplicate_mod,
    reason = "the source-owned transition installs the same fixture vocabulary under distinct schema types"
)]
pub mod successor;

pub use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};
pub use installation::WorthQueryCheckpointCapturePolicy as CapturePolicy;
pub use worth_query_host::facade::application_installation::ApplicationBuildIdentity;
pub use worth_query_host::facade::{
    application_entry::*, application_installation as installation,
};

pub const OWNERSHIP: worth_query_host::facade::application_contribution::ProducerPublicationOwnership = worth_query_host::facade::application_contribution::ProducerPublicationOwnership::SourceScopeCascade;
