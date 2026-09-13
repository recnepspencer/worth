use std::num::NonZeroU64;

use worth_store_physical_format::PhysicalCheckpointIdentity;
use worth_store_physical_integrity::{
    CheckpointBindingPayloadProjectionDenial, UntrustedPhysicalArtifact,
};

use super::literal_vectors::BINDING;
use super::support::{binding_scope, identity, other_store, validate_binding};

#[test]
fn binding_payload_projection_is_exact_incarnation_and_checkpoint_scoped() {
    let checkpoint = identity();
    let bytes = BINDING.to_vec();
    let input = UntrustedPhysicalArtifact::from_bounded_bytes(&bytes);
    let validated = validate_binding(&bytes, binding_scope(checkpoint, bytes.len() as u64));

    let projection = validated.project_payload(input, checkpoint).unwrap();

    assert_eq!(projection.checkpoint_identity(), checkpoint);
    assert_eq!(&bytes[projection.payload_range()], [0xaa, 0xbb, 0xcc]);

    let equal_copy = bytes.clone();
    assert_eq!(
        validated
            .project_payload(
                UntrustedPhysicalArtifact::from_bounded_bytes(&equal_copy),
                checkpoint,
            )
            .unwrap_err(),
        CheckpointBindingPayloadProjectionDenial::InputIncarnationMismatch
    );
    let foreign = PhysicalCheckpointIdentity::new(other_store(), NonZeroU64::new(7).unwrap());
    assert_eq!(
        validated.project_payload(input, foreign).unwrap_err(),
        CheckpointBindingPayloadProjectionDenial::CheckpointIdentityMismatch
    );
}
