//! A workflow operation encodes its input once, at bind, and reports the
//! request's two identity derivations in its admission, first and replayed.

pub use super::*;
pub use worth_query_testkit_certification_canonical::facade::certification::canonical_identity::assert_request_identity_work;
pub use worth_query_testkit_document_retention::facade::document_retention_model::schema::SetRetentionInput;
