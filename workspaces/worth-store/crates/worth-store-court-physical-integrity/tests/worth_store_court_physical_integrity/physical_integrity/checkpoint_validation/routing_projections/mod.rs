use {worth_store_physical_format::CHECKPOINT_BINDING_RECORD_PREFIX_BYTES};
use {worth_store_physical_integrity::project_checkpoint_binding_frame_length, worth_store_physical_integrity::validate_checkpoint_footer_envelope, worth_store_physical_integrity::CheckpointFooterEnvelopeIntegrityValidation, worth_store_physical_integrity::UntrustedPhysicalArtifact};

use {crate::literal_vectors::checkpoint_validation::BINDING, crate::literal_vectors::checkpoint_validation::FOOTER};
use {crate::checkpoint_validation::support::binding_scope, crate::checkpoint_validation::support::footer_scope, crate::checkpoint_validation::support::identity};

#[test]
fn footer_envelope_routes_only_after_exact_record_admission() {
    let scope = footer_scope(identity());
    let validation = validate_checkpoint_footer_envelope(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&FOOTER),
        scope,
    )
    .0;
    let worth_store_physical_integrity::CheckpointFooterEnvelopeIntegrityValidation::Intact(validated) = validation else {
        panic!("canonical footer envelope was rejected");
    };
    let projection = validated.routing_projection();
    assert_eq!(
        projection.footer(),
        worth_store_physical_format::CheckpointStreamFooter::decode_record(&FOOTER).unwrap()
    );
    assert_eq!(projection.footer_offset(), scope.byte_range().offset());

    let mut corrupt = FOOTER;
    *corrupt.last_mut().unwrap() ^= 0x80;
    assert!(matches!(
        validate_checkpoint_footer_envelope(
            worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&corrupt),
            scope,
        )
        .0,
        worth_store_physical_integrity::CheckpointFooterEnvelopeIntegrityValidation::Rejected(_)
    ));
}

#[test]
fn binding_frame_length_is_projected_by_integrity_not_an_owner_decoder() {
    let prefix = &BINDING[..CHECKPOINT_BINDING_RECORD_PREFIX_BYTES];
    let scope = binding_scope(identity(), prefix.len() as u64);
    let projection = project_checkpoint_binding_frame_length(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(prefix),
        scope,
    )
    .unwrap();
    assert_eq!(projection.encoded_bytes(), BINDING.len() as u64);

    let mut wrong_kind = prefix.to_vec();
    wrong_kind[9] = 2;
    assert!(project_checkpoint_binding_frame_length(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&wrong_kind),
        scope,
    )
    .is_err());
}
