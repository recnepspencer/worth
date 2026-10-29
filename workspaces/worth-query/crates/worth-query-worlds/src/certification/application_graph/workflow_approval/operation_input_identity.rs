//! A workflow operation encodes its input once, at bind, and reports the
//! request's two identity derivations in its admission, first and replayed.

pub use super::*;
pub use crate::facade::certification::canonical_identity::assert_request_identity_work;
pub use crate::facade::certification::document_retention_model::schema::SetRetentionInput;
