//! Projection authority is encoded at fixed width without dropping identity.
use serde::{Serialize, Serializer};

pub(super) struct ProjectionAuthorityEncoding(pub(super) u64);
impl Serialize for ProjectionAuthorityEncoding {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(&self.0.to_be_bytes())
    }
}

mod tests {
    use super::super::*;

    use worth_query_declaration::facade::application_program::ApplicationComputationPartition;

    impl ApplicationComputationPartition for WorthQueryInvariantEntityIdentity<(), ()> {
        const IDENTITY: &'static str = "projection-identity-encoding-test";
    }
}
