use std::marker::PhantomData;
use std::sync::Arc;
use worth_query_declaration::facade::application_operation::application_computation_partition_identity;
use worth_query_declaration::facade::application_program::ApplicationComputationPartition;
use worth_query_execution::facade::domain_computation::WorthQueryInvariantEntityIdentity;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::identity::KindId;
use worth_relational::facade::identity::PartitionId;

#[test]
fn projection_authority_encoding_has_equal_work_across_varint_boundaries() {
    let mut previous = None;
    for authority in [0, 127, 128, 16383, 16384, u64::MAX] {
        // A private serialization value, never issued read authority.
        let value = WorthQueryInvariantEntityIdentity::<(), ()>::new_for_test(
            EntityId::new(PartitionId::main(), 1, 1),
            KindId(1),
            Arc::from("Entry"),
            authority,
            PhantomData,
        );
        let identity =
            application_computation_partition_identity(&value, &mut |_| Ok::<(), ()>(())).unwrap();
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
