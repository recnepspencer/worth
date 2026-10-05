//! A root whose routing tree contradicts itself is damaged media. None of it
//! is a limit the operator admitted too low.

use worth_store_physical_format::{
    durable_artifact_checksum, DurablePhysicalRootManifest, ManifestBlockReference,
    PersistedRecordIdentity, PhysicalRootRoutingBlock, RecordArtifactFile,
};

use super::super::{ManifestEntryBudget, PageObservationFailure};
use super::observe_routes_with_budget;
use crate::integrity_ingress::RecoveryIntegrityIngressTrace;
use crate::orchestration::planning::selected_world_fixture::{selected_world, SelectedSource};

/// Entries no walk here comes near.
const AMPLE: u64 = 4_096;

/// How many records the walk of `root` routes.
fn routed(
    source: &mut SelectedSource<'_>,
    root: &DurablePhysicalRootManifest,
) -> Result<u64, PageObservationFailure> {
    let mut budget = ManifestEntryBudget::new(AMPLE, 0);
    let mut trace = RecoveryIntegrityIngressTrace::default();
    observe_routes_with_budget(
        source.discovery,
        root,
        source.format,
        &mut budget,
        &mut trace,
    )
    .map(|routes| routes.len() as u64)
}

/// The selected root's tree under another record count and routing root.
fn rerooted(
    root: &DurablePhysicalRootManifest,
    record_count: u64,
    next_block: u64,
    routing_root: ManifestBlockReference,
) -> DurablePhysicalRootManifest {
    DurablePhysicalRootManifest::builder(
        root.generation(),
        root.tree_identity(),
        root.node_capacity(),
        root.free_space_checksum(),
    )
    .record_count(record_count)
    .next_block(next_block)
    .routing_root(Some(routing_root))
    .admit()
    .unwrap()
}

#[test]
fn a_root_that_counts_a_record_its_tree_does_not_route_is_damaged() {
    selected_world("routes-record-count", 4).read(|mut source| {
        let root = source.root;
        let leaf = root.routing_root().unwrap();
        assert_eq!(routed(&mut source, root), Ok(root.record_count()));
        let miscounted = rerooted(root, root.record_count() + 1, root.next_block(), leaf);
        assert_eq!(
            routed(&mut source, &miscounted),
            Err(PageObservationFailure::InvalidManifest {
                target: None,
                artifact: RecordArtifactFile::RootManifest {
                    generation: root.generation(),
                },
            }),
        );
    });
}

#[test]
fn a_routing_block_two_references_reach_is_damaged() {
    selected_world("routes-block-reached-twice", 4).read(|mut source| {
        let root = source.root;
        let leaf = root.routing_root().unwrap();
        // A second reference to the leaf's block, over a later record range.
        let beyond =
            PersistedRecordIdentity::new(leaf.last().allocation_epoch(), leaf.last().ordinal() + 1)
                .unwrap();
        let twin = ManifestBlockReference::new(
            leaf.generation(),
            leaf.block(),
            leaf.level(),
            leaf.checksum(),
            beyond,
            beyond,
        )
        .unwrap();
        let branch = PhysicalRootRoutingBlock::branch(
            root.tree_identity(),
            root.generation(),
            root.next_block(),
            leaf.level() + 1,
            vec![leaf, twin],
            root.node_capacity(),
        )
        .unwrap();
        let bytes = branch.encode(source.format);
        let written = RecordArtifactFile::RootRoutingBlock {
            generation: root.generation(),
            block: root.next_block(),
        };
        std::fs::write(
            source
                .store
                .join("families/records/roots")
                .join(written.file_name()),
            &bytes,
        )
        .unwrap();
        let branched = rerooted(
            root,
            u64::from(root.node_capacity()) + 1,
            root.next_block() + 1,
            branch.reference(durable_artifact_checksum(&bytes)),
        );
        assert_eq!(
            routed(&mut source, &branched),
            Err(PageObservationFailure::InvalidManifest {
                target: None,
                artifact: RecordArtifactFile::RootRoutingBlock {
                    generation: leaf.generation(),
                    block: leaf.block(),
                },
            }),
        );
    });
}
