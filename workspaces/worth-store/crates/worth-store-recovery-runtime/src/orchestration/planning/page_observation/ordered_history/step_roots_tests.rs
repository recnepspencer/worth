use super::super::test_inventory::{inventory, root, root_at};
use super::*;
use crate::orchestration::planning::selected_world_fixture::{selected_world, SelectedSource};

/// What the member of the step under test declares.
const STEP: u64 = 5;

fn observed<'a>(
    root: &'a DurablePhysicalRootManifest,
    inventory: &'a RecoverySelectedSourceInventory,
) -> Observed<'a> {
    Observed {
        root,
        inventory,
        routes: &[],
    }
}

/// A node capacity other than this one.
const fn another(capacity: u16) -> u16 {
    if capacity == 64 {
        128
    } else {
        64
    }
}

/// The world's own inventory, read apart from the step under test.
fn inventory_of(world: &mut SelectedSource<'_>) -> RecoverySelectedSourceInventory {
    selected_source_inventory::observe(world.discovery, world.root, world.format, 4_096)
        .0
        .unwrap()
}

/// One run of the real step from `source` under `entries` manifest entries:
/// the generation it reread, the artifacts it read and the entries it left.
fn step(
    world: &mut SelectedSource<'_>,
    source: Observed<'_>,
    selected: Observed<'_>,
    entries: u64,
) -> (Result<Option<u64>, WalkFailure>, u64, u64) {
    let mut budget = ManifestEntryBudget::new(entries, 0);
    let before = world.discovery.counters().addressed_artifacts_read;
    let reread = charge_and_reread(
        world.discovery,
        &mut budget,
        &mut RecoveryIntegrityIngressTrace::default(),
        world.format,
        4_096,
        STEP as usize,
        source,
        selected,
    )
    .map(|reread| reread.map(|reread| reread.root.generation()));
    let read = world.discovery.counters().addressed_artifacts_read - before;
    (reread, read, budget.remaining())
}

const LIMIT: Result<Option<u64>, WalkFailure> = Err(WalkFailure::ManifestEntryLimit);

#[test]
fn a_step_that_changes_node_capacity_is_charged_the_whole_tree_of_its_result() {
    let (small_root, small) = (root(64, 2), inventory(64, 1, 1));
    let (large_root, large) = (root(128, 5), inventory(128, 3, 7));
    let grown = StepRoots {
        source: observed(&small_root, &small),
        result: observed(&large_root, &large),
    };
    // Five routes, three segment pages and seven free entries: the result's.
    assert_eq!(grown.whole_tree_rewrite(), Some(15));
    let shrunk = StepRoots {
        source: grown.result,
        result: grown.source,
    };
    assert_eq!(shrunk.whole_tree_rewrite(), Some(4));
    let same = StepRoots {
        source: grown.source,
        result: grown.source,
    };
    assert_eq!(same.whole_tree_rewrite(), Some(0));
    // Only free space changed its capacity: only its entries are charged.
    let (free_root, free_only) = (root(64, 5), inventory(128, 3, 7));
    let free_grown = StepRoots {
        source: grown.source,
        result: observed(&free_root, &free_only),
    };
    assert_eq!(free_grown.whole_tree_rewrite(), Some(7));
}

#[test]
fn a_step_one_entry_short_of_its_charge_reads_nothing() {
    selected_world("step-charge-precedes-reads", 4).read(|mut world| {
        let generation = world.root.generation();
        let real = inventory_of(&mut world);
        // The world's root is the result of a source of its own node
        // capacities, so the step rewrote no tree whole. Any root but the
        // world's stands for the selected one, so the step rereads it.
        let (source_root, source) = (
            root_at(generation - 1, world.root.node_capacity(), 1),
            inventory(real.free_space.node_capacity(), 1, 1),
        );
        let (far_root, far) = (root_at(generation + 1, 64, 1), inventory(64, 1, 1));
        let (source, far) = (observed(&source_root, &source), observed(&far_root, &far));
        let (short, read, _) = step(&mut world, source, far, STEP - 1);
        assert_eq!(short, LIMIT);
        assert_eq!(read, 0, "the step is charged before its first read");
        let (exact, read, left) = step(&mut world, source, far, STEP);
        assert_eq!(exact, Ok(Some(generation)));
        assert!(read > 2, "the whole result root was reread");
        assert_eq!(left, 0);
    });
}

#[test]
fn a_tree_the_step_rewrote_whole_is_charged_before_the_trees_after_it_are_read() {
    selected_world("step-whole-tree-charge", 4).read(|mut world| {
        let generation = world.root.generation();
        let real = inventory_of(&mut world);
        let (routes, free, pages) = (
            world.root.record_count(),
            real.free_space.entry_count(),
            real.segment_pages.len() as u64,
        );
        assert!(
            routes > 0 && free > 0 && pages > 0,
            "every tree holds entries"
        );
        let (root_capacity, free_capacity) =
            (world.root.node_capacity(), real.free_space.node_capacity());
        let segment_blocks = real.segment_topology.len() as u64;
        let (far_root, far) = (root_at(generation + 1, 64, 1), inventory(64, 1, 1));
        let far = observed(&far_root, &far);
        let before = |root_capacity, free_capacity| {
            (
                root_at(generation - 1, root_capacity, 1),
                inventory(free_capacity, 1, 1),
            )
        };

        // Both capacities changed: the headers count the records and the free
        // entries, and only the segment tree counts its own pages.
        let (source_root, source) = before(another(root_capacity), another(free_capacity));
        let source = observed(&source_root, &source);
        let counted_by_headers = STEP + routes + free;
        let (short, read, _) = step(&mut world, source, far, counted_by_headers - 1);
        assert_eq!(short, LIMIT);
        assert_eq!(read, 2, "only the root and the free-space header");
        let (short, read, _) = step(&mut world, source, far, counted_by_headers + pages - 1);
        assert_eq!(short, LIMIT);
        assert_eq!(read, 2 + segment_blocks, "and then only the segment tree");
        let (exact, _, left) = step(&mut world, source, far, counted_by_headers + pages);
        assert_eq!((exact, left), (Ok(Some(generation)), 0));

        // The same step onto the selected root, which the walk already holds.
        let selected = observed(world.root, &real);
        let (short, read, _) = step(&mut world, source, selected, counted_by_headers + pages - 1);
        assert_eq!((short, read), (LIMIT, 0));
        let (exact, read, left) = step(&mut world, source, selected, counted_by_headers + pages);
        assert_eq!((exact, read, left), (Ok(None), 0, 0));

        // Each capacity alone charges only the trees that take it.
        for (source_capacities, need) in [
            (
                (another(root_capacity), free_capacity),
                STEP + routes + pages,
            ),
            ((root_capacity, another(free_capacity)), STEP + free),
            ((root_capacity, free_capacity), STEP),
        ] {
            let (source_root, source) = before(source_capacities.0, source_capacities.1);
            let source = observed(&source_root, &source);
            assert_eq!(step(&mut world, source, far, need - 1).0, LIMIT);
            let (exact, _, left) = step(&mut world, source, far, need);
            assert_eq!((exact, left), (Ok(Some(generation)), 0));
        }
    });
}
