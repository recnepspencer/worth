//! Canonical Query diagnostic identity protocol.
//!
//! Scope and scheme remain part of every identity. Encoding and Bridge
//! translation describe evidence; neither admits inspection, publication,
//! execution, or correction. Those owners require their original artifacts.

mod artifact;
mod consumer_scope_strings;
mod encoder;
mod foundational;
mod graph_application_scope_strings;
mod installed_domain_scope_strings;
mod key;
mod scheme;
mod scope;
mod scope_strings;
mod sealed;
mod tag;

#[cfg(test)]
mod tests;

pub use artifact::{WorthQueryEvidenceIdentity, WorthQueryEvidenceIdentityComparisonError};
pub use encoder::WorthQueryEvidenceIdentityEncoder;
pub use key::WorthQueryEvidenceIdentityKey;
pub use scheme::WorthQueryEvidenceIdentityScheme;
pub use scope::WorthQueryEvidenceScope;
pub use tag::WorthQueryEvidenceTag;

pub use encoder::worth_query_evidence_identity;
