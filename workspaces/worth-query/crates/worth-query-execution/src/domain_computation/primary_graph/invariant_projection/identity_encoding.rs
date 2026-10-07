//! Projection authority is encoded at fixed width without dropping identity.
use serde::{Serialize, Serializer};

pub(super) struct ProjectionAuthorityEncoding(pub(super) u64);
impl Serialize for ProjectionAuthorityEncoding {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(&self.0.to_be_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use worth_query_declaration::facade::application_operation::application_computation_partition_identity;
    use worth_query_declaration::facade::application_program::ApplicationComputationPartition;
    use worth_relational::facade::identity::PartitionId;

    impl ApplicationComputationPartition for WorthQueryInvariantEntityIdentity<(), ()> {
        const IDENTITY: &'static str = "projection-identity-encoding-test";
    }

    #[test]
    fn projection_authority_encoding_has_equal_work_across_varint_boundaries() {
        let mut previous = None;
        for authority in [0, 127, 128, 16383, 16384, u64::MAX] {
            // A private serialization value, never issued read authority.
            let value = WorthQueryInvariantEntityIdentity::<(), ()> {
                entity_id: EntityId::new(PartitionId::main(), 1, 1),
                kind: KindId(1),
                entity: Arc::from("Entry"),
                authority_identity: authority,
                _marker: PhantomData,
            };
            let identity =
                application_computation_partition_identity(&value, &mut |_| Ok::<(), ()>(()))
                    .unwrap();
            if let Some(previous) = previous {
                let previous: worth_query_declaration::facade::application_operation::ApplicationComputationPartitionIdentity = previous;
                assert_eq!(
                    identity.work(),
                    previous.work(),
                    "authority values must not change a binding's encoding work"
                );
                assert_ne!(
                    identity.digest(),
                    previous.digest(),
                    "different authorities remain different bindings"
                );
            }
            previous = Some(identity);
        }
    }
}
