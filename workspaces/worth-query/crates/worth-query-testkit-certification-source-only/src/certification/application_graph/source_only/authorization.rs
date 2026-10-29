//! Disclosure observation cannot execute a retained command or cross installations.
#[path = "fixture.rs"]
#[allow(
    dead_code,
    reason = "Shared custody fixture also exposes pending-publication helpers used by the other source courts."
)]
pub mod fixture;
#[path = "handler.rs"]
pub mod handler;
#[path = "host.rs"]
pub mod host;
#[path = "readback.rs"]
pub mod readback;
#[path = "schema.rs"]
pub mod schema;
pub use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};
pub use readback::{observe_head, read_retention};
pub use worth_query_host::facade::application_entry::*;
