use super::*;
use crate::history::data::canonical_commit_envelope::tests::envelope_with_patch_and_update;
use crate::identity::data::{EntityId, PartitionId};
use crate::schema::data::schema_authority_snapshot_digest_bytes;
use crate::transactions::data::primitives::RecordRef;

#[test]
fn completeness_recomputes_visibility_commitment() {
    let body = body(false);
    let (mut axes, descriptor, content) = binding(&body);
    assert!(original_is_complete(
        axes,
        &descriptor,
        content.accumulator,
        content,
        &body,
        true
    ));
    axes.visibility = RelationalBranchVisibilityCommitment::for_root(
        &self::body(true),
        axes.storage_root,
        axes.schema_root,
        axes.correctness_index,
    );
    assert!(
        !original_is_complete(axes, &descriptor, content.accumulator, content, &body, true),
        "completeness must reject a commitment to any different visible tuple"
    );
}

#[test]
fn completeness_rejects_each_reconstructed_binding_axis_mismatch() {
    let body = body(false);
    let (axes, descriptor, content) = binding(&body);
    for mutant in [
        RelationalBranchRootAxes {
            storage_version: 2,
            ..axes
        },
        RelationalBranchRootAxes {
            storage_root: [9; 32],
            ..axes
        },
        RelationalBranchRootAxes {
            schema_root: [9; 32],
            ..axes
        },
        RelationalBranchRootAxes {
            visibility: RelationalBranchVisibilityCommitment::from_checkpoint([9; 32]),
            ..axes
        },
    ] {
        assert!(!original_is_complete(
            mutant,
            &descriptor,
            content.accumulator,
            content,
            &body,
            true
        ));
    }
    for mutant in [
        RelationalBranchRootDescriptor::new([9; 32], axes.schema_root),
        RelationalBranchRootDescriptor::new(axes.storage_root, [9; 32]),
    ] {
        assert!(!original_is_complete(
            axes,
            &mutant,
            content.accumulator,
            content,
            &body,
            true
        ));
    }
    assert!(!original_is_complete(
        axes,
        &descriptor,
        [9; 32],
        content,
        &body,
        true
    ));
    assert!(!original_is_complete(
        axes,
        &descriptor,
        content.accumulator,
        content,
        &body,
        false
    ));
    let zero = ReconstructedContent {
        storage: CanonicalDigestId::new([0; 32]),
        ..content
    };
    let zero_axes = RelationalBranchRootAxes {
        storage_root: [0; 32],
        visibility: RelationalBranchVisibilityCommitment::for_root(
            &body,
            [0; 32],
            axes.schema_root,
            axes.correctness_index,
        ),
        ..axes
    };
    assert!(!original_is_complete(
        zero_axes,
        &RelationalBranchRootDescriptor::new([0; 32], axes.schema_root),
        zero.accumulator,
        zero,
        &body,
        true
    ));
}

fn binding(
    body: &CanonicalCommitEnvelope,
) -> (
    RelationalBranchRootAxes,
    RelationalBranchRootDescriptor,
    ReconstructedContent,
) {
    let schema = schema_authority_snapshot_digest_bytes(&body.schema_authority);
    let content = ReconstructedContent {
        storage: CanonicalDigestId::new([7; 32]),
        schema: CanonicalDigestId::new(schema),
        accumulator: [11; 32],
    };
    let correctness_index = super::super::RelationalRootCorrectnessIndex::AuthoritativeFallback;
    let axes = RelationalBranchRootAxes {
        storage_version: 1,
        storage_root: [7; 32],
        schema_root: schema,
        correctness_index,
        visibility: RelationalBranchVisibilityCommitment::for_root(
            body,
            [7; 32],
            schema,
            correctness_index,
        ),
    };
    (
        axes,
        RelationalBranchRootDescriptor::new([7; 32], schema),
        content,
    )
}

fn body(opaque: bool) -> CanonicalCommitEnvelope {
    let entity = EntityId::new(PartitionId::main(), 1, 1);
    let mut envelope = envelope_with_patch_and_update(RecordRef::Entity(entity), entity);
    envelope
        .body_mut_for_test()
        .patch
        .authoritative_record_patches[0]
        .contains_opaque_aspect = opaque;
    envelope
}
