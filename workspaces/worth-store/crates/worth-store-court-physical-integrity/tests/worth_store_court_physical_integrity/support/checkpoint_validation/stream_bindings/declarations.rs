pub(crate) use worth_store_physical_format::{
    CheckpointSelectiveRecordAggregate, PhysicalCheckpointIdentity, RecordArtifactFile,
};
pub(crate) use worth_store_physical_integrity::{
    validate_checkpoint_footer, CheckpointFooterIntegrityValidation,
    CheckpointFooterValidationBasis, PhysicalBlastRadius, PhysicalDamageCause, PhysicalFormatField,
    UntrustedPhysicalArtifact,
};

pub(crate) use crate::checkpoint_validation::support::{
    assert_damage, binding_scope, compaction_scope, dirty_scope, field_range, footer_scope,
    header_scope_staged, identity, reseal_record, validate_binding, validate_compaction,
    validate_dirty, validate_footer, validate_header, BINDING_OFFSET, DIRTY_OFFSET,
};
pub(crate) use crate::checksum_oracles::{crc32c, sha256};
pub(crate) use crate::literal_vectors::checkpoint_validation::{
    BINDING, BINDING_AGGREGATE, BINDING_COMPACTION, DIRTY_AGGREGATE, DIRTY_BASIS, FOOTER, HEADER,
};
pub(crate) use std::num::NonZeroU64;

#[derive(Clone, Copy)]
pub(crate) enum LaterRecord {
    Dirty,
    Compaction,
    Binding,
}

pub(crate) fn assert_later_identity_substitution(target: LaterRecord) {
    let identity = identity();
    let substituted = worth_store_physical_format::PhysicalCheckpointIdentity::new(
        identity.store_identity(),
        std::num::NonZeroU64::new(identity.sequence().get() + 1).unwrap(),
    );
    let header = validate_header(&HEADER, header_scope_staged());
    let scope = footer_scope(identity);
    let (rejection, substituted_scope) = match target {
        crate::checkpoint_validation::stream_bindings::LaterRecord::Dirty => {
            let dirty = validate_dirty(&DIRTY_BASIS, dirty_scope(substituted));
            let compaction = validate_compaction(&BINDING_COMPACTION, compaction_scope(identity));
            let binding = validate_binding(&BINDING, binding_scope(identity, BINDING.len() as u64));
            let substituted_scope = dirty.scope();
            let basis = worth_store_physical_integrity::CheckpointFooterValidationBasis::new(
                &header,
                std::slice::from_ref(&dirty),
                &compaction,
                std::slice::from_ref(&binding),
            );
            (footer_basis_rejection(scope, basis), substituted_scope)
        }
        crate::checkpoint_validation::stream_bindings::LaterRecord::Compaction => {
            let dirty = validate_dirty(&DIRTY_BASIS, dirty_scope(identity));
            let compaction =
                validate_compaction(&BINDING_COMPACTION, compaction_scope(substituted));
            let binding = validate_binding(&BINDING, binding_scope(identity, BINDING.len() as u64));
            let substituted_scope = compaction.scope();
            let basis = worth_store_physical_integrity::CheckpointFooterValidationBasis::new(
                &header,
                std::slice::from_ref(&dirty),
                &compaction,
                std::slice::from_ref(&binding),
            );
            (footer_basis_rejection(scope, basis), substituted_scope)
        }
        crate::checkpoint_validation::stream_bindings::LaterRecord::Binding => {
            let dirty = validate_dirty(&DIRTY_BASIS, dirty_scope(identity));
            let compaction = validate_compaction(&BINDING_COMPACTION, compaction_scope(identity));
            let binding =
                validate_binding(&BINDING, binding_scope(substituted, BINDING.len() as u64));
            let substituted_scope = binding.scope();
            let basis = worth_store_physical_integrity::CheckpointFooterValidationBasis::new(
                &header,
                std::slice::from_ref(&dirty),
                &compaction,
                std::slice::from_ref(&binding),
            );
            (footer_basis_rejection(scope, basis), substituted_scope)
        }
    };
    assert_damage(
        rejection,
        substituted_scope,
        worth_store_physical_integrity::PhysicalDamageCause::ArtifactIdentityMismatch,
        substituted_scope.byte_range(),
        None,
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );
}

pub(crate) fn footer_basis_rejection<'records, 'media>(
    scope: worth_store_physical_integrity::PhysicalArtifactScope,
    basis: CheckpointFooterValidationBasis<'records, 'media>,
) -> worth_store_physical_integrity::PhysicalIntegrityRejection {
    match validate_checkpoint_footer(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&FOOTER),
        scope,
        basis,
    )
    .0
    {
        worth_store_physical_integrity::CheckpointFooterIntegrityValidation::Rejected(
            rejection,
        ) => rejection,
        worth_store_physical_integrity::CheckpointFooterIntegrityValidation::Intact(_) => {
            panic!("substituted later-record identity unexpectedly validated")
        }
    }
}
