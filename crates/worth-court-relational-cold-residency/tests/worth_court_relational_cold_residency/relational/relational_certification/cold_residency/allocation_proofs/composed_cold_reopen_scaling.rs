//! Layer 3 composed cold-reopen scaling court for persistent naming: a
//! structural-identity name scheme, an Anchor group partition and forked
//! branches composed in one store, captured as a `CHK24` checkpoint and
//! reopened cold at `1x` (64 names and group members, 4 older forks) and
//! `10x` (640 and 40), with an active fork taken after the group; a
//! names-only `10x` (640 names, 4 forks) separates name from branch growth.
//!
//! Measured on the court binary's global allocator and the checkpoint
//! source's reads, after the cold reopen:
//! - the reopen reads no member at all (no table, group or history scan) and
//!   allocates no faster than the checkpoint manifest it decodes grows;
//! - a point resolve of a name and of its group faults at most four more
//!   naming pages, and allocates per faulted page byte as at `1x`;
//! - a one-member detach publishes the same delta frame, faults at most four
//!   more pages, and beyond the same commit without a group change allocates
//!   per faulted page byte as at `1x`: its work follows the pages (at most
//!   64 KiB each) it faults, never the group's 640 members;
//! - a fork reads nothing and allocates the same for 64 or 640 names.
//!
//! Allocation is compared per faulted page byte because a `1x` store's pages
//! are partly filled while `10x` pages are full: each faulted page costs its
//! decoded rows, bounded by the page size, not by the names it holds.
use std::sync::Arc;

use worth_execution::ExecutionRequest;
use worth_relational::facade::aggregate_identity::AggregateMemberRole;
use worth_relational::facade::durability::RelationalNativeCheckpoint;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::{EntityId, PartitionId};
use worth_relational::facade::lineage::{
    CommitNamingPlan, PersistentNameIdentity, PersistentNameKey, PersistentNameSelection,
};
use worth_relational::facade::mvcc::RelationalTransactionIntent;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::transactions::{
    EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent, WorkerIntentBatch,
};
use worth_testkit_relational::aspect_storage::publications::release_test_commit_snapshot;
use worth_testkit_relational::cold_recovery::{capture_external_checkpoint, immutable_source};
use worth_testkit_relational::durable_medium::immutable_members::ImmutableCheckpointMemberSource;
use worth_testkit_relational::durable_medium::TestDurableMedium;
use worth_testkit_relational::durable_publication::field_patch;

use super::super::bounded_naming_fixture::{
    create_named_entities, durable_runtime_with, naming_schema, observe_branch,
    publish_group_partition, reopen_with, ENTITY_SCHEME, GROUP_SCHEME,
};
use super::fixture::create_named_batch_in_partition;
use super::fork_and_layer_scaling::fork_branch;
use super::{ensure_tracker_installed, measure, AllocWindow};

/// Names and group members per `1x`, and older forks per `1x`.
const NAMES: usize = 64;
const OLDER_FORKS: usize = 4;

struct Composed {
    medium: Arc<TestDurableMedium>,
    checkpoint: RelationalNativeCheckpoint,
    source: Arc<ImmutableCheckpointMemberSource>,
    main: BranchId,
    seed: EntityId,
    sample: PersistentNameIdentity,
    last: PersistentNameIdentity,
}

/// One operation's allocations, faulted naming pages and their bytes, and
/// every other checkpoint member it read.
#[derive(Clone, Copy)]
struct Cost {
    window: AllocWindow,
    pages: usize,
    page_bytes: usize,
    other_reads: usize,
}

impl Cost {
    /// Allocated bytes beyond `beyond`, per faulted naming page byte.
    fn per_page_byte(&self, beyond: usize) -> f64 {
        self.window.alloc_bytes.saturating_sub(beyond) as f64 / self.page_bytes.max(1) as f64
    }
}

struct Costs {
    manifest: usize,
    reopen: Cost,
    resolve: Cost,
    control: Cost,
    detach: Cost,
    frame: usize,
    fork: Cost,
}

/// `names` times 64 names, all in one Anchor group, over `forks` times 4
/// older forks. The names fill their own partition, so the commits touching
/// `PartitionId::main()`'s seed entity edit a partition of fixed size.
fn build_composed(request: &ExecutionRequest<'_, '_>, names: usize, forks: usize) -> Composed {
    let medium = TestDurableMedium::new();
    let producer = durable_runtime_with(&medium, u64::MAX / 4, naming_schema(true));
    let main = producer.main_branch_identity().branch_id().clone();
    let (_, seed, _) = create_named_entities(request, &producer, &main, &["composed-seed".into()]);
    for i in 0..OLDER_FORKS * forks {
        fork_branch(request, &producer, &main, format!("older-fork-{i:03}"));
    }
    let mut named = Vec::new();
    for batch in 0..names {
        let labels: Vec<String> = (0..NAMES)
            .map(|i| format!("composed-{batch:02}-{i:02}"))
            .collect();
        let created =
            create_named_batch_in_partition(request, &producer, &main, PartitionId(1), &labels);
        named.extend(created.1);
    }
    let structural = PersistentNameIdentity::of_pending_birth(
        ENTITY_SCHEME,
        "occ.composed-00-00",
        "authored",
        &[],
        &[],
        true,
    );
    assert_eq!(
        named[0], structural,
        "the scheme mints structural identities"
    );
    let group = named
        .iter()
        .enumerate()
        .map(|(i, &name)| match i {
            0 => (name, AggregateMemberRole::Anchor),
            _ => (name, AggregateMemberRole::Ordinary),
        })
        .collect();
    publish_group_partition(request, &producer, &main, seed[0], "whole", vec![group]);
    fork_branch(request, &producer, &main, "active-fork".into());
    let (checkpoint, _) = capture_external_checkpoint(request, &producer);
    let source = immutable_source(&checkpoint, &medium);
    Composed {
        medium,
        checkpoint,
        source,
        main,
        seed: seed[0],
        sample: named[0],
        last: *named.last().unwrap(),
    }
}

/// The detach's commit without its group change: the seed's field edit
/// under an empty naming plan.
fn touch_seed(request: &ExecutionRequest<'_, '_>, runtime: &RelationalRuntime, built: &Composed) {
    let basis = observe_branch(runtime, &built.main);
    let mut tx = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    let edit = UpdateEntityFieldsIntent {
        entity_id: built.seed,
        fields: field_patch("name", "name", "control"),
    };
    let batch = WorkerIntentBatch::new("control").push(MutationIntent::Entity(
        EntityMutationIntent::UpdateFields(edit),
    ));
    tx.push_batch(batch, request.allocation_policy()).unwrap();
    tx.attach_naming_plan(CommitNamingPlan::default());
    let committed = tx.commit(request, runtime).unwrap();
    release_test_commit_snapshot(runtime, &committed);
}

fn other_reads(source: &ImmutableCheckpointMemberSource) -> usize {
    source.member_reads.lock().unwrap().len()
        + source.commit_page_reads.lock().unwrap().len()
        + source.paged_reads.load(std::sync::atomic::Ordering::SeqCst)
}

/// Runs `operation` measured, with what it read from the checkpoint.
fn faulting<T>(built: &Composed, operation: impl FnOnce() -> T) -> (T, Cost) {
    let source = &built.source;
    let pages_before = source.naming_page_reads.lock().unwrap().len();
    let others_before = other_reads(source);
    let (value, window) = measure(operation);
    let pages = source.naming_page_reads.lock().unwrap()[pages_before..].to_vec();
    let page_bytes = pages
        .iter()
        .map(|page| source.members[page].bytes().len())
        .sum();
    let cost = Cost {
        window,
        pages: pages.len(),
        page_bytes,
        other_reads: other_reads(source) - others_before,
    };
    (value, cost)
}

fn costs_after_cold_reopen(request: &ExecutionRequest<'_, '_>, built: &Composed) -> Costs {
    let fresh = durable_runtime_with(&built.medium, u64::MAX / 4, naming_schema(true));
    let (checkpoint, source) = (built.checkpoint.clone(), built.source.clone());
    let (runtime, reopen) = faulting(built, || reopen_with(request, checkpoint, source, fresh));
    let head = observe_branch(&runtime, &built.main);
    let ((), resolve) = faulting(built, || {
        runtime
            .with_persistent_names_at_observation(request, &head.observation(), |view| {
                let name = PersistentNameKey::Entity(built.sample);
                assert!(view
                    .resolve(name, &PersistentNameSelection::Exact)
                    .result
                    .is_ok());
                let group = view
                    .group_of_member(GROUP_SCHEME, &built.sample)
                    .expect("the group survives the cold reopen");
                assert!(view
                    .resolve(group, &PersistentNameSelection::Exact)
                    .result
                    .is_ok());
            })
            .unwrap();
    });
    // The first commit after reopen faults the commit path's own state; the
    // second is the warm baseline the detach adds its group change to.
    touch_seed(request, &runtime, built);
    let ((), control) = faulting(built, || touch_seed(request, &runtime, built));
    let detached = vec![vec![(built.last, AggregateMemberRole::Anchor)]];
    let (committed, detach) = faulting(built, || {
        publish_group_partition(
            request,
            &runtime,
            &built.main,
            built.seed,
            "detach",
            detached,
        )
    });
    let (_, fork) = faulting(built, || {
        fork_branch(request, &runtime, &built.main, "fork-after-reopen".into())
    });
    Costs {
        manifest: built.checkpoint.bytes().len(),
        reopen,
        resolve,
        control,
        detach,
        frame: committed.group_lineage_frame().len(),
        fork,
    }
}

fn ratio(large: &Cost, small: &Cost) -> f64 {
    large.window.alloc_bytes as f64 / small.window.alloc_bytes.max(1) as f64
}

fn report(label: &str, small: &Costs, large: &Costs) {
    for (op, s, l) in [
        ("reopen", &small.reopen, &large.reopen),
        ("resolve", &small.resolve, &large.resolve),
        ("control commit", &small.control, &large.control),
        ("detach", &small.detach, &large.detach),
        ("fork", &small.fork, &large.fork),
    ] {
        eprintln!(
            "[7B-COMPOSED {label}] {op}: 1x={} B ({} naming pages, {} B; {} other reads) vs 10x={} B ({} naming pages, {} B; {} other reads) -> alloc_ratio={:.4}x",
            s.window.alloc_bytes,
            s.pages,
            s.page_bytes,
            s.other_reads,
            l.window.alloc_bytes,
            l.pages,
            l.page_bytes,
            l.other_reads,
            ratio(l, s)
        );
    }
    eprintln!(
        "[7B-COMPOSED {label}] manifest 1x={} B vs 10x={} B; detach frame 1x={} B vs 10x={} B",
        small.manifest, large.manifest, small.frame, large.frame
    );
}

#[test]
fn composed_naming_groups_and_forks_cold_reopen_cost_stays_flat_from_1x_to_10x() {
    ensure_tracker_installed();
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    // Warm up on an independent store before measuring any scale.
    let _ = costs_after_cold_reopen(request, &build_composed(request, 1, 1));
    let small = costs_after_cold_reopen(request, &build_composed(request, 1, 1));
    let names_only = costs_after_cold_reopen(request, &build_composed(request, 10, 1));
    let large = costs_after_cold_reopen(request, &build_composed(request, 10, 10));
    report("names-only", &small, &names_only);
    report("composed", &small, &large);
    for costs in [&small, &names_only, &large] {
        assert_eq!(
            (costs.reopen.pages, costs.reopen.other_reads),
            (0, 0),
            "a cold reopen reads no checkpoint member"
        );
        assert_eq!(
            (costs.fork.pages, costs.fork.other_reads),
            (0, 0),
            "a fork after reopen reads no checkpoint member"
        );
    }
    let manifest_ratio = large.manifest as f64 / small.manifest as f64;
    assert!(
        ratio(&large.reopen, &small.reopen) <= manifest_ratio,
        "cold reopen allocations follow the manifest it decodes, never the names"
    );
    assert!(
        large.resolve.pages <= small.resolve.pages + 4
            && large.resolve.per_page_byte(0) <= small.resolve.per_page_byte(0) * 1.25,
        "a point resolve after reopen costs the pages it faults"
    );
    assert!(
        large.frame as f64 <= small.frame as f64 * 1.05,
        "the detach frame records the delta alone"
    );
    let detach_work = |costs: &Costs| costs.detach.per_page_byte(costs.control.window.alloc_bytes);
    assert!(
        large.detach.pages <= small.detach.pages + 4
            && detach_work(&large) <= detach_work(&small) * 1.25,
        "a one-member detach after reopen costs the pages it faults, never the group's size"
    );
    assert!(
        ratio(&names_only.fork, &small.fork) <= 1.05,
        "a fork after reopen shares every naming table whatever their size"
    );
}
