#[allow(unused_imports)] use crate::checkpoint_validation::validation_records::*;
use {std::num::NonZeroU64};

use {worth_store_physical_format::PhysicalCheckpointIdentity};
use {worth_store_physical_integrity::CheckpointFooterValidationBasis, worth_store_physical_integrity::PhysicalArtifactScope, worth_store_physical_integrity::PhysicalByteRange, worth_store_physical_integrity::PhysicalIntegrityValidationRecord};

use {crate::literal_vectors::checkpoint_validation::BINDING, crate::literal_vectors::checkpoint_validation::BINDING_COMPACTION, crate::literal_vectors::checkpoint_validation::DIRTY_BASIS, crate::literal_vectors::checkpoint_validation::FOOTER, crate::literal_vectors::checkpoint_validation::HEADER};
use {crate::checkpoint_validation::support::binding_scope, crate::checkpoint_validation::support::compaction_scope, crate::checkpoint_validation::support::dirty_scope, crate::checkpoint_validation::support::footer_scope, crate::checkpoint_validation::support::header_scope_known, crate::checkpoint_validation::support::header_scope_staged, crate::checkpoint_validation::support::identity, crate::checkpoint_validation::support::other_store, crate::checkpoint_validation::support::validate_binding, crate::checkpoint_validation::support::validate_compaction, crate::checkpoint_validation::support::validate_dirty, crate::checkpoint_validation::support::validate_footer, crate::checkpoint_validation::support::validate_header};

#[test]
fn validation_records_bind_every_kind_to_store_sequence_family_and_range() {
    let identity = identity();
    let foreign_sequence = worth_store_physical_format::PhysicalCheckpointIdentity::new(
        identity.store_identity(),
        std::num::NonZeroU64::new(identity.sequence().get() + 1).unwrap(),
    );
    let foreign_store = worth_store_physical_format::PhysicalCheckpointIdentity::new(other_store(), identity.sequence());
    let header_scope = header_scope_staged();
    let dirty_own = dirty_scope(identity);
    let compaction_own = compaction_scope(identity);
    let binding_own = binding_scope(identity, BINDING.len() as u64);
    let footer_own = footer_scope(identity);
    let header = validate_header(&HEADER, header_scope);
    let dirty = validate_dirty(&DIRTY_BASIS, dirty_own);
    let compaction = validate_compaction(&BINDING_COMPACTION, compaction_own);
    let binding = validate_binding(&BINDING, binding_own);
    let footer = validate_footer(
        &FOOTER,
        footer_own,
        worth_store_physical_integrity::CheckpointFooterValidationBasis::new(
            &header,
            std::slice::from_ref(&dirty),
            &compaction,
            std::slice::from_ref(&binding),
        ),
    );
    let records = [
        (header.into_validation_record(), header_scope),
        (dirty.into_validation_record(), dirty_own),
        (compaction.into_validation_record(), compaction_own),
        (binding.into_validation_record(), binding_own),
        (footer.into_validation_record(), footer_own),
    ];
    let foreign_identities = [
        header_scope_known(foreign_sequence),
        dirty_scope(foreign_sequence),
        compaction_scope(foreign_sequence),
        binding_scope(foreign_sequence, BINDING.len() as u64),
        footer_scope(foreign_sequence),
    ];
    let foreign_stores = [
        header_scope_known(foreign_store),
        dirty_scope(foreign_store),
        compaction_scope(foreign_store),
        binding_scope(foreign_store, BINDING.len() as u64),
        footer_scope(foreign_store),
    ];
    let foreign_families = [
        dirty_scope(identity),
        header_scope_known(identity),
        header_scope_known(identity),
        header_scope_known(identity),
        header_scope_known(identity),
    ];
    for (index, (record, own_scope)) in records.into_iter().enumerate() {
        assert!(record.matches_scope(own_scope));
        assert!(!record.matches_scope(foreign_identities[index]));
        assert!(!record.matches_scope(foreign_stores[index]));
        assert!(!record.matches_scope(shifted(own_scope)));
        assert!(!record.matches_scope(foreign_families[index]));
    }
}

#[test]
fn scope_digest_changes_while_same_inspected_bytes_digest_does_not() {
    let identity = identity();
    let staged = validate_header(&HEADER, header_scope_staged()).into_validation_record();
    let known = validate_header(&HEADER, header_scope_known(identity)).into_validation_record();
    assert_digest_partition(staged, known);

    let dirty = dirty_scope(identity);
    let shifted_dirty = shifted(dirty);
    let original = validate_dirty(&DIRTY_BASIS, dirty).into_validation_record();
    let shifted = validate_dirty(&DIRTY_BASIS, shifted_dirty).into_validation_record();
    assert_digest_partition(original, shifted);
}
