use worth_store_security::StoreTenantScope;
use worth_store_tiering::ColdPlacementState;

use crate::corruption::test_support::quarantined_read_corruption;
use crate::placement::admission::test_support::admit_inline_placement;
use crate::publication::test_support::publish_generation_with_bytes_and_chunk_size;
use crate::test_support::{
    admitted_multichunk_sequence_for_scope, blob_scope, physical_payload_for_bytes,
};
use crate::{
    BlobAuthorityClassification, BlobCapsuleMaterializationAuthority, BlobCapsuleReadinessDenial,
    BlobCapsuleSliceDeclaration, BlobCapsuleSliceSelection, BlobGenerationRegistry,
    BlobGenerationRegistryAdmission, BlobObjectClassificationAdmission, BlobObjectId,
    BlobStreamingReadObservation, BlobStreamingReadObservedChunk, BlobStreamingReadWindow,
    BlobStreamingVerifiedRead,
};

#[test]
fn same_admitted_subset_materializes_same_readiness_evidence_and_counters() {
    let lane = capsule_lane("phase21.equivalence", b"abcdefghijklmno", 4);
    let authority = lane.authority();
    let declaration = BlobCapsuleSliceDeclaration::for_generation(lane.generation)
        .select(BlobCapsuleSliceSelection::chunk_ordinals([0, 2]).expect("selection"))
        .require_parent_root_basis();

    let first = materialize_readiness(&authority, &lane, declaration.clone(), &lane.scope, &[]);
    let second = materialize_readiness(&authority, &lane, declaration, &lane.scope, &[]);

    assert_eq!(first.readiness_digest(), second.readiness_digest());
    assert_eq!(first.counters(), second.counters());
    assert_eq!(first.selected_chunks(), second.selected_chunks());
    assert_eq!(first.declared_bytes(), second.declared_bytes());
}

#[test]
fn hostile_rows_digest_only_scope_quarantine_and_cold_unavailable_deny_before_readiness() {
    let lane = capsule_lane("phase21.denials", b"aaaabbbb", 4);
    let authority = lane.authority();
    let declaration = BlobCapsuleSliceDeclaration::for_generation(lane.generation)
        .select(BlobCapsuleSliceSelection::chunk_ordinals([0]).expect("selection"))
        .require_parent_root_basis();
    let planned = authority.plan_slice(declaration).expect("planned");

    match crate::reject_copied_capsule_row_as_capsule_readiness("copied-row") {
        BlobCapsuleReadinessDenial::CopiedCapsuleRow { .. } => {}
        other => panic!("expected copied-row denial, got {other:?}"),
    }
    match crate::reject_digest_only_chunk_reference_as_capsule_readiness("sha256:only") {
        BlobCapsuleReadinessDenial::DigestOnlyChunkReference { .. } => {}
        other => panic!("expected digest-only denial, got {other:?}"),
    }

    let stale_scope = blob_scope(
        "phase21.denials.stale",
        StoreTenantScope::MultiTenantPhysicalBoundary,
    );
    match authority.classify_slice_for_materialization(
        planned.clone(),
        &stale_scope,
        &lane.placement,
        &[],
    ) {
        Err(BlobCapsuleReadinessDenial::StaleSecurityScope { .. }) => {}
        other => panic!("expected stale-scope denial, got {other:?}"),
    }

    let quarantine = quarantined_read_corruption(
        "phase21.denials.quarantine",
        &lane.published,
        lane.visible.clone(),
    );
    match authority.classify_slice_for_materialization(
        planned.clone(),
        &lane.scope,
        &lane.placement,
        &[quarantine],
    ) {
        Err(BlobCapsuleReadinessDenial::QuarantinedChunk { .. }) => {}
        other => panic!("expected quarantine denial, got {other:?}"),
    }

    let mut cold_unavailable = lane.placement.clone();
    cold_unavailable.cold_state = Some(ColdPlacementState::ColdUnavailable);
    match authority.classify_slice_for_materialization(planned, &lane.scope, &cold_unavailable, &[])
    {
        Err(BlobCapsuleReadinessDenial::ColdPlacementUnavailable { .. }) => {}
        other => panic!("expected cold-unavailable denial, got {other:?}"),
    }
}

#[test]
fn subset_without_basis_cross_scope_and_reachability_drift_produce_distinct_outcomes() {
    let lane = capsule_lane("phase21.slice", b"abcdefghijklmno", 4);
    let authority = lane.authority();

    match authority.plan_slice(
        BlobCapsuleSliceDeclaration::for_generation(lane.generation)
            .select(BlobCapsuleSliceSelection::chunk_ordinals([1]).expect("selection")),
    ) {
        Err(BlobCapsuleReadinessDenial::MissingParentRootBasis { .. }) => {}
        other => panic!("expected basis denial, got {other:?}"),
    }

    let planned = authority
        .plan_slice(
            BlobCapsuleSliceDeclaration::for_generation(lane.generation)
                .select(BlobCapsuleSliceSelection::chunk_ordinals([0, 1]).expect("selection"))
                .require_parent_root_basis(),
        )
        .expect("planned");
    let cross_scope_sequence = admitted_multichunk_sequence_for_scope(
        blob_scope(
            "phase21.cross-scope",
            StoreTenantScope::MultiTenantPhysicalBoundary,
        ),
        b"abcdefghijklmno",
        4,
    );
    let mut cross_scope_placement = lane.placement.clone();
    cross_scope_placement.security_metadata = cross_scope_sequence
        .proof_frontier()
        .first_leaf()
        .security_metadata();
    match authority.classify_slice_for_materialization(
        planned.clone(),
        &lane.scope,
        &cross_scope_placement,
        &[],
    ) {
        Err(BlobCapsuleReadinessDenial::CrossScopeSharedChunk { .. }) => {}
        other => panic!("expected cross-scope denial, got {other:?}"),
    }

    let classified = authority
        .classify_slice_for_materialization(planned, &lane.scope, &lane.placement, &[])
        .expect("classified");
    let prepared = authority
        .admit_materialized_capsule_read(
            &classified,
            lane.verified_read.clone(),
            lane.observations.clone(),
        )
        .expect("prepared");
    match authority.materialize_capsule_bundle(classified, &lane.changed_reachability, prepared) {
        Err(BlobCapsuleReadinessDenial::ReachabilityChangedDuringCreation { .. }) => {}
        other => panic!("expected reachability-drift denial, got {other:?}"),
    }
}

pub(crate) fn materialize_readiness(
    authority: &BlobCapsuleMaterializationAuthority,
    lane: &CapsuleLane,
    declaration: BlobCapsuleSliceDeclaration,
    scope: &crate::BlobChunkSecurityScope,
    quarantines: &[crate::BlobChunkQuarantine],
) -> crate::BlobCapsuleReadinessWitness {
    let planned = authority.plan_slice(declaration).expect("planned");
    let classified = authority
        .classify_slice_for_materialization(planned, scope, &lane.placement, quarantines)
        .expect("classified");
    let prepared = authority
        .admit_materialized_capsule_read(
            &classified,
            lane.verified_read.clone(),
            lane.observations.clone(),
        )
        .expect("prepared");
    let materialized = authority
        .materialize_capsule_bundle(classified, &lane.reachability, prepared)
        .expect("materialized");
    authority
        .publish_capsule_readiness(materialized)
        .expect("readiness")
}

pub(crate) struct CapsuleLane {
    pub(crate) published: crate::BlobGenerationPublished,
    pub(crate) visible: crate::BlobVisibleGeneration,
    pub(crate) generation: crate::BlobGeneration,
    pub(crate) object_id: BlobObjectId,
    pub(crate) ordered_leaves: &'static [crate::BlobChunkProofLeaf],
    pub(crate) reachability: crate::BlobChunkReachabilityProofSet,
    pub(crate) changed_reachability: crate::BlobChunkReachabilityProofSet,
    pub(crate) placement: crate::AdmittedBlobPlacement,
    pub(crate) scope: crate::BlobChunkSecurityScope,
    pub(crate) verified_read: BlobStreamingVerifiedRead,
    pub(crate) observations: Vec<BlobStreamingReadObservation>,
    pub(crate) registry: &'static BlobGenerationRegistry,
}

impl CapsuleLane {
    pub(crate) fn authority(&self) -> BlobCapsuleMaterializationAuthority {
        let observation = self
            .registry
            .observe_registered_generation(&self.object_id, self.generation)
            .expect("registered generation");
        BlobCapsuleMaterializationAuthority::from_generation_observation(
            &observation,
            self.ordered_leaves,
        )
        .expect("capsule authority")
    }
}

pub(crate) fn capsule_lane(case: &str, bytes: &'static [u8], chunk_size: u64) -> CapsuleLane {
    let scope = blob_scope(case, StoreTenantScope::TenantPhysicalBoundary);
    let sequence = admitted_multichunk_sequence_for_scope(
        blob_scope(case, StoreTenantScope::TenantPhysicalBoundary),
        bytes,
        chunk_size,
    );
    let ordered_leaves = Box::leak(
        sequence
            .proof_frontier()
            .ordered_leaves()
            .to_vec()
            .into_boxed_slice(),
    );
    let (publication, stored_digest) =
        crate::lifecycle::generation_registry_test_support::root_publication_with_bytes_and_chunk_size(
            case, bytes, chunk_size,
        );
    let object_id = crate::BlobObjectId::from_declared_digest(
        crate::lifecycle::generation_registry_test_support::digest(&format!(
            "sha256:{case}.object"
        )),
    );
    let generation = crate::BlobGeneration::published(1);
    let lifecycle =
        crate::lifecycle::generation_registry_test_support::lifecycle_receipt_for_publication_with_identity(
            crate::lifecycle::generation_registry_test_support::PublicationIdentityCase::new(case, case),
            1,
            publication.chunk_tree_root().clone(),
            publication.logical_content_digest().clone(),
            stored_digest,
            BlobAuthorityClassification::StoreOwnedPhysicalBlob,
            bytes,
        );
    let reachability = lifecycle.reachability().clone();
    let changed_publication = crate::lifecycle::generation_registry_test_support::root_publication_with_bytes_and_chunk_size(
        &format!("{case}.changed"),
        b"abcdefghijklmno!",
        chunk_size,
    );
    let changed_lifecycle =
        crate::lifecycle::generation_registry_test_support::lifecycle_receipt_for_publication_with_identity(
            crate::lifecycle::generation_registry_test_support::PublicationIdentityCase::new(&format!("{case}.changed"), &format!("{case}.changed")),
            1,
            changed_publication.0.chunk_tree_root().clone(),
            changed_publication.0.logical_content_digest().clone(),
            changed_publication.1,
            BlobAuthorityClassification::StoreOwnedPhysicalBlob,
            b"abcdefghijklmno!",
        );
    let changed_reachability = changed_lifecycle.reachability().clone();
    let placement = admit_inline_placement(&reachability);
    let mut registry = BlobGenerationRegistry::new();
    let classification = BlobObjectClassificationAdmission::from_executed_lifecycle(&lifecycle);
    BlobGenerationRegistryAdmission::from_executed_lifecycle(
        publication.clone(),
        lifecycle,
        classification,
    )
    .publish(
        &mut registry,
        crate::lifecycle::generation_registry_test_support::registry_authority(case),
    )
    .expect("registry publication");
    let registry = Box::leak(Box::new(registry));
    let (published, visible) = publish_generation_with_bytes_and_chunk_size(
        &format!("{case}.published"),
        bytes,
        chunk_size,
    );
    let observations = bytes
        .chunks(chunk_size as usize)
        .enumerate()
        .map(|(index, chunk)| {
            BlobStreamingReadObservation::from_chunk(
                BlobStreamingReadObservedChunk::from_store_payload(
                    ordinal(index as u64),
                    index as u64 * chunk_size,
                    physical_payload_for_bytes(chunk),
                    BlobStreamingReadWindow::bounded(8).expect("window"),
                )
                .expect("observed chunk"),
            )
        })
        .collect::<Vec<_>>();
    let verified_read = BlobStreamingVerifiedRead::for_content_comparison_test(
        object_id.clone(),
        generation,
        publication.chunk_tree_root().clone(),
        publication.logical_content_digest().clone(),
        bytes.len() as u64,
    );

    CapsuleLane {
        published,
        visible,
        generation,
        object_id,
        ordered_leaves,
        reachability,
        changed_reachability,
        placement,
        scope,
        verified_read,
        observations,
        registry,
    }
}

fn ordinal(value: u64) -> crate::BlobChunkOrdinal {
    let mut ordinal = crate::BlobChunkOrdinal::first();
    for _ in 0..value {
        ordinal = ordinal.next();
    }
    ordinal
}
