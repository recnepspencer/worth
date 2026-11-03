//! Projection authority is encoded at fixed width without dropping identity.
use serde::{Serialize, Serializer};

pub(super) struct ProjectionAuthorityEncoding(pub(super) u64);
impl Serialize for ProjectionAuthorityEncoding {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(&self.0.to_be_bytes())
    }
}

pub mod tests {
    use super::super::*;
    use worth_query_declaration::facade::application_operation::application_computation_partition_identity;
    use worth_query_declaration::facade::application_program::ApplicationComputationPartition;
    use worth_relational::facade::identity::PartitionId;
    impl ApplicationComputationPartition for WorthQueryInvariantEntityIdentity<(), ()> {
        const IDENTITY: &'static str = "projection-identity-encoding-test";
    }
}
