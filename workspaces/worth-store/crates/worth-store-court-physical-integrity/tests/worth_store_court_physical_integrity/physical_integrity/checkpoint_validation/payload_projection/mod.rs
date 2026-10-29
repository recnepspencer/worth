use {std::num::NonZeroU64};

use {worth_store_physical_format::PhysicalCheckpointIdentity};
use {worth_store_physical_integrity::CheckpointBindingPayloadProjectionDenial, worth_store_physical_integrity::UntrustedPhysicalArtifact};

use {crate::literal_vectors::checkpoint_validation::BINDING};
use {crate::checkpoint_validation::support::binding_scope, crate::checkpoint_validation::support::identity, crate::checkpoint_validation::support::other_store, crate::checkpoint_validation::support::validate_binding};

#[test]
fn binding_payload_projection_is_exact_incarnation_and_checkpoint_scoped() {
    let checkpoint = identity();
    let bytes = BINDING.to_vec();
    let input = worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&bytes);
    let validated = validate_binding(&bytes, binding_scope(checkpoint, bytes.len() as u64));

    let projection = validated.project_payload(input, checkpoint).unwrap();

    assert_eq!(projection.checkpoint_identity(), checkpoint);
    assert_eq!(&bytes[projection.payload_range()], [0xaa, 0xbb, 0xcc]);

    let equal_copy = bytes.clone();
    assert_eq!(
        validated
            .project_payload(
                worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&equal_copy),
                checkpoint,
            )
            .unwrap_err(),
        worth_store_physical_integrity::CheckpointBindingPayloadProjectionDenial::InputIncarnationMismatch
    );
    let foreign = worth_store_physical_format::PhysicalCheckpointIdentity::new(other_store(), std::num::NonZeroU64::new(7).unwrap());
    assert_eq!(
        validated.project_payload(input, foreign).unwrap_err(),
        worth_store_physical_integrity::CheckpointBindingPayloadProjectionDenial::CheckpointIdentityMismatch
    );
}
