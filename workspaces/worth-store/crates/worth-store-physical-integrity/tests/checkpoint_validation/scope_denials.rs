use worth_store_physical_integrity::{
    validate_checkpoint_binding, validate_checkpoint_binding_compaction,
    validate_checkpoint_dirty_basis, validate_checkpoint_footer, validate_checkpoint_stream_header,
    CheckpointBindingCompactionIntegrityValidation, CheckpointBindingIntegrityValidation,
    CheckpointDirtyBasisIntegrityValidation, CheckpointFooterIntegrityValidation,
    CheckpointFooterValidationBasis, CheckpointStreamHeaderIntegrityValidation,
    PhysicalArtifactScope, PhysicalBlastRadius, PhysicalByteRange, PhysicalDamageCause,
    PhysicalFormatField, PhysicalIntegrityRejection, UntrustedPhysicalArtifact,
};

use super::literal_vectors::{BINDING, BINDING_COMPACTION, DIRTY_BASIS, FOOTER, HEADER};
use super::support::{
    assert_damage, binding_scope, compaction_scope, dirty_scope, field_range, footer_scope,
    header_scope_staged, identity, other_store, reseal_record, validate_binding,
    validate_compaction, validate_dirty, validate_header,
};

#[test]
fn every_validator_rejects_a_different_checkpoint_family_scope() {
    let identity = identity();
    let scopes = [
        PhysicalArtifactScope::checkpoint_dirty_basis(identity, range(0, HEADER.len() as u64)),
        PhysicalArtifactScope::checkpoint_binding_compaction(
            identity,
            range(164, DIRTY_BASIS.len() as u64),
        ),
        PhysicalArtifactScope::checkpoint_binding(
            identity,
            range(232, BINDING_COMPACTION.len() as u64),
        ),
        PhysicalArtifactScope::checkpoint_footer(identity, range(268, BINDING.len() as u64)),
        PhysicalArtifactScope::checkpoint_binding(identity, range(291, FOOTER.len() as u64)),
    ];
    let rejections = [
        header_rejection(&HEADER, scopes[0]),
        dirty_rejection(&DIRTY_BASIS, scopes[1]),
        compaction_rejection(&BINDING_COMPACTION, scopes[2]),
        binding_rejection(&BINDING, scopes[3]),
        footer_wrong_scope_rejection(scopes[4]),
    ];
    for (scope, rejection) in scopes.into_iter().zip(rejections) {
        assert_damage(
            rejection,
            scope,
            PhysicalDamageCause::FamilyMismatch,
            scope.byte_range(),
            None,
            PhysicalBlastRadius::CompleteArtifact,
        );
    }
}

#[test]
fn footer_store_substitution_is_distinct_from_sequence_substitution() {
    let identity = identity();
    let mut substituted = FOOTER;
    substituted[16..32].copy_from_slice(&other_store().bytes());
    reseal_record(&mut substituted);
    let scope = footer_scope(identity);
    let header = validate_header(&HEADER, header_scope_staged());
    let dirty = validate_dirty(&DIRTY_BASIS, dirty_scope(identity));
    let compaction = validate_compaction(&BINDING_COMPACTION, compaction_scope(identity));
    let binding = validate_binding(&BINDING, binding_scope(identity, BINDING.len() as u64));
    let basis = CheckpointFooterValidationBasis::new(
        &header,
        std::slice::from_ref(&dirty),
        &compaction,
        std::slice::from_ref(&binding),
    );
    let rejection = match validate_checkpoint_footer(
        UntrustedPhysicalArtifact::from_bounded_bytes(&substituted),
        scope,
        basis,
    )
    .0
    {
        CheckpointFooterIntegrityValidation::Rejected(rejection) => rejection,
        _ => panic!("foreign-store footer unexpectedly validated"),
    };
    assert_damage(
        rejection,
        scope,
        PhysicalDamageCause::StoreIdentityMismatch,
        field_range(scope, 16, 16),
        Some(PhysicalFormatField::StoreIdentity),
        PhysicalBlastRadius::CompleteArtifact,
    );
}

fn header_rejection(bytes: &[u8], scope: PhysicalArtifactScope) -> PhysicalIntegrityRejection {
    match validate_checkpoint_stream_header(
        UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
        scope,
    )
    .0
    {
        CheckpointStreamHeaderIntegrityValidation::Rejected(rejection) => rejection,
        _ => panic!("wrong-family header scope validated"),
    }
}

fn dirty_rejection(bytes: &[u8], scope: PhysicalArtifactScope) -> PhysicalIntegrityRejection {
    match validate_checkpoint_dirty_basis(
        UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
        scope,
    )
    .0
    {
        CheckpointDirtyBasisIntegrityValidation::Rejected(rejection) => rejection,
        _ => panic!("wrong-family dirty scope validated"),
    }
}

fn compaction_rejection(bytes: &[u8], scope: PhysicalArtifactScope) -> PhysicalIntegrityRejection {
    match validate_checkpoint_binding_compaction(
        UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
        scope,
    )
    .0
    {
        CheckpointBindingCompactionIntegrityValidation::Rejected(rejection) => rejection,
        _ => panic!("wrong-family compaction scope validated"),
    }
}

fn binding_rejection(bytes: &[u8], scope: PhysicalArtifactScope) -> PhysicalIntegrityRejection {
    match validate_checkpoint_binding(UntrustedPhysicalArtifact::from_bounded_bytes(bytes), scope).0
    {
        CheckpointBindingIntegrityValidation::Rejected(rejection) => rejection,
        _ => panic!("wrong-family binding scope validated"),
    }
}

fn footer_wrong_scope_rejection(scope: PhysicalArtifactScope) -> PhysicalIntegrityRejection {
    let identity = identity();
    let header = validate_header(&HEADER, header_scope_staged());
    let dirty = validate_dirty(&DIRTY_BASIS, dirty_scope(identity));
    let compaction = validate_compaction(&BINDING_COMPACTION, compaction_scope(identity));
    let binding = validate_binding(&BINDING, binding_scope(identity, BINDING.len() as u64));
    let basis = CheckpointFooterValidationBasis::new(
        &header,
        std::slice::from_ref(&dirty),
        &compaction,
        std::slice::from_ref(&binding),
    );
    match validate_checkpoint_footer(
        UntrustedPhysicalArtifact::from_bounded_bytes(&FOOTER),
        scope,
        basis,
    )
    .0
    {
        CheckpointFooterIntegrityValidation::Rejected(rejection) => rejection,
        _ => panic!("wrong-family footer scope validated"),
    }
}

fn range(offset: u64, length: u64) -> PhysicalByteRange {
    PhysicalByteRange::new(offset, length).unwrap()
}
