use super::{
    CheckpointPublicationReadmission, CheckpointPublicationStabilityProof,
    CheckpointReadInterlockPlan, CheckpointRootEpochTransition, ReadDuringCheckpointVerdict,
};
use crate::stable_read_execution::stable_physical_read_receipt_for_certification_root;
use crate::{
    epoch::{manifest_epoch_from_entry_seed, root_epoch_from_entry_seed},
    CheckpointPublicationIdentity, CheckpointPublicationRoot, CheckpointPublicationRootBasis,
    CurrentPhysicalRoot, CurrentPhysicalRootBasis, PhysicalOrderingContract,
};
use std::num::NonZeroU64;
use worth_store_physical_format::{
    store_namespace::ProposedStoreIdentity, store_namespace::StoreNamespaceIdentityRecord,
    store_namespace::StoreNamespaceVersion, CheckpointBindingCompactionHeader, CheckpointRootBasis,
    CheckpointStreamEncoder, CheckpointWalSourceRange, PhysicalCheckpointIdentity,
    PhysicalCheckpointSource, CHECKPOINT_BINDING_COMPACTION_HEADER_RECORD_BYTES,
    CHECKPOINT_STREAM_FOOTER_RECORD_BYTES, CHECKPOINT_STREAM_HEADER_RECORD_BYTES,
};
use worth_store_physical_integrity::{
    validate_checkpoint_binding_compaction, validate_checkpoint_footer,
    validate_checkpoint_stream_header, CheckpointBindingCompactionIntegrityValidation,
    CheckpointFooterIntegrityValidation, CheckpointFooterValidationBasis,
    CheckpointStreamHeaderIntegrityValidation, CheckpointStreamHeaderScopeIdentity,
    PhysicalArtifactScope, PhysicalByteRange, UntrustedPhysicalArtifact, VerifiedCheckpointStream,
};

pub fn read_during_checkpoint_verdict_for_certification_test() -> ReadDuringCheckpointVerdict {
    let old_root = current_root_for_certification_test(41);
    let new_root = current_root_for_certification_test(42);
    let checkpoint = checkpoint_for_certification_test();
    let checkpoint_root = CheckpointPublicationRoot::from_checkpoint_publication(
        CheckpointPublicationRootBasis::new(new_root.epoch()),
        PhysicalOrderingContract::root_swap_acquire_release(),
        CheckpointPublicationIdentity::from_physical_checkpoint_identity(
            checkpoint.source().identity(),
        ),
    )
    .expect("certification checkpoint root should admit");
    let readmission =
        CheckpointPublicationReadmission::admit(checkpoint_root, new_root, &checkpoint)
            .expect("certification checkpoint readmission should admit");
    let transition = CheckpointRootEpochTransition::admit(old_root, readmission)
        .expect("certification checkpoint transition should admit");
    let pre_read = stable_physical_read_receipt_for_certification_root(old_root, 64);
    let post_read = stable_physical_read_receipt_for_certification_root(new_root, 64);
    let plan = CheckpointReadInterlockPlan::admit(pre_read, transition)
        .expect("certification checkpoint plan should admit");
    let proof =
        CheckpointPublicationStabilityProof::from_plan_and_post_publication_read(plan, post_read)
            .expect("certification checkpoint stability proof should admit");
    ReadDuringCheckpointVerdict::from_stability_proof(proof)
        .expect("certification checkpoint verdict should admit")
}

fn checkpoint_for_certification_test() -> VerifiedCheckpointStream {
    let store_identity = StoreNamespaceIdentityRecord::new(
        StoreNamespaceVersion::CURRENT,
        ProposedStoreIdentity::from_nonzero_bytes([7; 16]).expect("store identity"),
    )
    .published_identity();
    let source = PhysicalCheckpointSource::concurrent(
        PhysicalCheckpointIdentity::new(
            store_identity,
            NonZeroU64::new(1).expect("checkpoint sequence"),
        ),
        CheckpointWalSourceRange::new(10, 20).expect("checkpoint WAL range"),
        CheckpointRootBasis::new(1, 1),
        1,
    );
    let (encoder, mut bytes) = CheckpointStreamEncoder::begin(source);
    let (encoder, compaction_header) = encoder.begin_binding_compaction(
        CheckpointBindingCompactionHeader::new(1, 12).expect("compaction binding header"),
    );
    bytes.extend(compaction_header);
    let (_, footer) = encoder.finish();
    bytes.extend(footer);
    admit_empty_checkpoint_stream(&bytes, source)
}

fn admit_empty_checkpoint_stream(
    bytes: &[u8],
    source: PhysicalCheckpointSource,
) -> VerifiedCheckpointStream {
    let header_range = PhysicalByteRange::new(0, CHECKPOINT_STREAM_HEADER_RECORD_BYTES as u64)
        .expect("header range");
    let compaction_range = PhysicalByteRange::new(
        header_range.end_exclusive(),
        CHECKPOINT_BINDING_COMPACTION_HEADER_RECORD_BYTES as u64,
    )
    .expect("compaction range");
    let footer_range = PhysicalByteRange::new(
        compaction_range.end_exclusive(),
        CHECKPOINT_STREAM_FOOTER_RECORD_BYTES as u64,
    )
    .expect("footer range");
    let header_scope = PhysicalArtifactScope::checkpoint_stream_header(
        CheckpointStreamHeaderScopeIdentity::staged(source.identity().store_identity()),
        header_range,
    );
    let compaction_scope =
        PhysicalArtifactScope::checkpoint_binding_compaction(source.identity(), compaction_range);
    let footer_scope = PhysicalArtifactScope::checkpoint_footer(source.identity(), footer_range);
    let record = |range: PhysicalByteRange| {
        UntrustedPhysicalArtifact::from_bounded_bytes(
            &bytes[range.offset() as usize..range.end_exclusive() as usize],
        )
    };
    let CheckpointStreamHeaderIntegrityValidation::Intact(header) =
        validate_checkpoint_stream_header(record(header_range), header_scope).0
    else {
        panic!("certification checkpoint header must validate")
    };
    let CheckpointBindingCompactionIntegrityValidation::Intact(compaction) =
        validate_checkpoint_binding_compaction(record(compaction_range), compaction_scope).0
    else {
        panic!("certification checkpoint compaction must validate")
    };
    let CheckpointFooterIntegrityValidation::Intact(footer) = validate_checkpoint_footer(
        record(footer_range),
        footer_scope,
        CheckpointFooterValidationBasis::new(&header, &[], &compaction, &[]),
    )
    .0
    else {
        panic!("certification checkpoint footer must validate")
    };
    VerifiedCheckpointStream::assemble_from_validated_records(
        UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
        &header,
        &[],
        &compaction,
        &[],
        &footer,
    )
    .expect("certification checkpoint stream must assemble")
}

fn current_root_for_certification_test(seed: u64) -> CurrentPhysicalRoot {
    CurrentPhysicalRoot::from_physical_isolation_entry(
        CurrentPhysicalRootBasis::new(
            root_epoch_from_entry_seed(seed),
            manifest_epoch_from_entry_seed(seed),
            worth_store_physical_format::PhysicalStoreIdentity::physical_format_default()
                .authority_identity(),
        ),
        PhysicalOrderingContract::root_swap_acquire_release(),
    )
    .expect("certification root ordering should admit")
}
