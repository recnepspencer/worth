//! A root whose routing tree contradicts itself is damaged media. None of it
//! is a limit the operator admitted too low.

use worth_store_physical_format::{
    durable_artifact_checksum, CurrentPhysicalRecordPlacement, DurablePhysicalRootManifest,
    ManifestBlockReference, PersistedRecordIdentity, PhysicalRootRoutingBlock, RecordArtifactFile,
};

use super::super::{
    ManifestEntryBudget, PageObservationFailure, ResidentAllowance, ResidentTraceDenial,
};
use super::{observe_routes_held, observe_routes_with_budget, RoutesFailure};
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
    let root_unit = budget.charge_root()?;
    observe_routes_with_budget(
        source.discovery,
        root,
        source.format,
        &root_unit,
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

#[test]
fn routes_their_holder_has_no_room_for_are_refused_as_held_and_never_as_an_entry_limit() {
    selected_world("routes-held", 4).read(|source| {
        let root = source.root;
        let mut held = |maximum: u64| {
            let mut budget = ManifestEntryBudget::new(AMPLE, 0);
            let mut trace = RecoveryIntegrityIngressTrace::default();
            let mut resident = ResidentAllowance::new(maximum);
            let root_unit = budget.charge_root().unwrap();
            let outcome = observe_routes_held(
                source.discovery,
                root,
                source.format,
                &root_unit,
                &mut budget,
                &mut trace,
                &mut resident,
            )
            .map(|routes| routes.len() as u64);
            let held = (resident.used(), trace.owned_heap_bytes());
            (outcome, resident.peak(), budget.refused_at(), held)
        };
        let (routed, need, _, (used, traced)) = held(1 << 30);
        assert!(matches!(routed, Ok(count) if count == root.record_count()));
        // One leaf block: its window, its trace slot and its routes are held.
        let page = u64::from(source.format.page_size().bytes());
        let route = std::mem::size_of::<CurrentPhysicalRecordPlacement>() as u64;
        assert_eq!(
            Some(used),
            traced.map(|traced| 4 * page + 512 + traced + root.record_count() * (2 * route + 64)),
        );
        assert!(need >= used);
        // Exactly what the traversal holds at once is enough.
        let (at_need, peak, refused_at, _) = held(need);
        assert!(matches!(at_need, Ok(count) if count == root.record_count()));
        assert_eq!((peak, refused_at), (need, None));
        // One byte less is the holder's refusal, and no entry was refused.
        for short in [need - 1, 0] {
            let (outcome, _, refused_at, _) = held(short);
            assert!(matches!(
                outcome,
                Err(RoutesFailure::Held(
                    ResidentTraceDenial::ResidentBoundExceeded
                ))
            ));
            assert_eq!(refused_at, None);
        }
    });
}
