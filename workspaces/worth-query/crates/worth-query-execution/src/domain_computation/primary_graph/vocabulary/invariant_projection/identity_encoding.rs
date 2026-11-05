//! Moved whole from `domain_computation::primary_graph::invariant_projection::identity_encoding`: the items below and every impl they own.
use serde::Serialize;
use serde::Serializer;

pub(in crate::domain_computation::primary_graph) struct ProjectionAuthorityEncoding(
    pub(in crate::domain_computation::primary_graph) u64,
);

impl Serialize for ProjectionAuthorityEncoding {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(&self.0.to_be_bytes())
    }
}
