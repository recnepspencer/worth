//! Projection authority is encoded at fixed width without dropping identity.
use serde::{Serialize, Serializer};

pub struct ProjectionAuthorityEncoding(pub(super) u64);
impl Serialize for ProjectionAuthorityEncoding {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(&self.0.to_be_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectionAuthorityEncoding;

    use worth_query_declaration::facade::application_operation::{
        application_computation_partition_identity, ApplicationComputationPartitionIdentity,
    };
    use worth_query_declaration::facade::application_program::ApplicationComputationPartition;

    impl ApplicationComputationPartition for ProjectionAuthorityEncoding {
        const IDENTITY: &'static str = "projection-identity-encoding-test";
    }

    #[test]
    fn projection_authority_encoding_has_equal_work_across_varint_boundaries() {
        let mut previous: Option<ApplicationComputationPartitionIdentity> = None;
        for authority in [0, 127, 128, 16383, 16384, u64::MAX] {
            let identity = application_computation_partition_identity(
                &ProjectionAuthorityEncoding(authority),
                &mut |_| Ok::<(), ()>(()),
            )
            .unwrap();
            if let Some(previous) = previous {
                assert_eq!(identity.work(), previous.work(), "encoded work is fixed");
                assert_ne!(
                    identity.digest(),
                    previous.digest(),
                    "authority bits are retained"
                );
            }
            previous = Some(identity);
        }
    }
}
