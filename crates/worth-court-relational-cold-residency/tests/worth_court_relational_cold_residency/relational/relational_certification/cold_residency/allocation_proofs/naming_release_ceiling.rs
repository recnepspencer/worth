//! The persistent-naming release ceiling run (`persistent-naming.md`,
//! milestone closeout): one composed durable store at 100,000 named
//! entities, built over 10,000 commits of ten structural-identity births,
//! with 1,000 Anchor groups of 100 members and an older fork every 160
//! commits, captured as a `CHK24` checkpoint and reopened cold.
//!
//! Manual and ignored: run it in `--release` with `--ignored --nocapture`.
//! It prints one `[7B-CEILING]` line: cold reopen latency, its peak live heap
//! and the process's resident set around it, and the latencies of forking a
//! branch, resolving a name with its group, detaching one group member and
//! committing one birth after the reopen (the first, and the median of the
//! following ones on other names, groups and branch names; births all land
//! in one populated partition, so the first also faults its cold records).
use std::sync::Arc;
use std::time::{Duration, Instant};

use worth_execution::ExecutionRequest;
use worth_relational::facade::aggregate_identity::AggregateMemberRole;
use worth_relational::facade::durability::RelationalNativeCheckpoint;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::{EntityId, PartitionId};
use worth_relational::facade::lineage::{
    PersistentNameIdentity, PersistentNameKey, PersistentNameSelection,
};
use worth_relational::facade::runtime::RelationalRuntime;
use worth_testkit_relational::cold_recovery::{capture_external_checkpoint, immutable_source};
use worth_testkit_relational::durable_medium::immutable_members::ImmutableCheckpointMemberSource;
use worth_testkit_relational::durable_medium::TestDurableMedium;

use super::super::bounded_naming_fixture::{
    create_named_entities, durable_runtime_with, naming_pages, naming_schema, observe_branch,
    publish_group_partition, reopen_with, GROUP_SCHEME,
};
use super::fixture::create_named_batch_in_partition;
use super::fork_and_layer_scaling::fork_branch;
use super::{ensure_tracker_installed, measure};

const COMMITS: usize = 10_000;
const NAMES_PER_COMMIT: usize = 10;
const PARTITIONS: usize = 100;
const GROUP_MEMBERS: usize = 100;
/// Commits between group declarations: each declares the names they made.
const GROUPED_EVERY: usize = 100;
const FORK_EVERY: usize = 160;
/// Operations timed after the first of each kind.
const REPEATS: usize = 9;

/// The process's resident set, in MiB.
fn resident_mib() -> f64 {
    let pid = std::process::id().to_string();
    let out = std::process::Command::new("ps")
        .args(["-o", "rss=", "-p", &pid])
        .output()
        .expect("ps reports the resident set");
    let kib: f64 = String::from_utf8_lossy(&out.stdout).trim().parse().unwrap();
    kib / 1024.0
}

fn timed<T>(operation: impl FnOnce() -> T) -> (T, Duration) {
    let start = Instant::now();
    let value = operation();
    (value, start.elapsed())
}

/// The first latency and the median of the rest, in microseconds.
fn first_and_median(mut times: Vec<Duration>) -> (u128, u128) {
    let first = times.remove(0).as_micros();
    times.sort_unstable();
    (first, times[times.len() / 2].as_micros())
}

/// The checkpointed store, what the timed operations act on, and what its
/// build cost.
struct CeilingStore {
    medium: Arc<TestDurableMedium>,
    checkpoint: RelationalNativeCheckpoint,
    source: Arc<ImmutableCheckpointMemberSource>,
    main: BranchId,
    seed: EntityId,
    /// The first name of every commit, and the last member of every group.
    sample: Vec<PersistentNameIdentity>,
    detachable: Vec<PersistentNameIdentity>,
    forks: usize,
    build: Duration,
    rss_built_mib: f64,
}

/// `names` split into Anchor groups of `GROUP_MEMBERS`, each led by its first.
fn anchor_groups(
    names: &[PersistentNameIdentity],
) -> Vec<Vec<(PersistentNameIdentity, AggregateMemberRole)>> {
    let role = |(i, &name): (usize, &PersistentNameIdentity)| match i {
        0 => (name, AggregateMemberRole::Anchor),
        _ => (name, AggregateMemberRole::Ordinary),
    };
    names
        .chunks(GROUP_MEMBERS)
        .map(|members| members.iter().enumerate().map(role).collect())
        .collect()
}

fn build_ceiling_store(request: &ExecutionRequest<'_, '_>) -> CeilingStore {
    let medium = TestDurableMedium::new();
    let producer = durable_runtime_with(&medium, u64::MAX / 4, naming_schema(true));
    let main = producer.main_branch_identity().branch_id().clone();
    let started = Instant::now();
    let (_, seed, _) = create_named_entities(request, &producer, &main, &["ceiling-seed".into()]);
    let (mut pending, mut sample, mut detachable, mut forks) =
        (Vec::new(), Vec::new(), Vec::new(), 0);
    for commit in 0..COMMITS {
        let labels: Vec<String> = (0..NAMES_PER_COMMIT)
            .map(|i| format!("ceiling-{commit:05}-{i}"))
            .collect();
        let partition = PartitionId(1 + (commit % PARTITIONS) as u32);
        let (_, names) =
            create_named_batch_in_partition(request, &producer, &main, partition, &labels);
        sample.push(names[0]);
        pending.extend(names);
        if (commit + 1) % GROUPED_EVERY == 0 {
            let groups = anchor_groups(&pending);
            detachable.extend(groups.iter().map(|group| group.last().unwrap().0));
            publish_group_partition(request, &producer, &main, seed[0], "groups", groups);
            pending.clear();
        }
        if (commit + 1) % FORK_EVERY == 0 {
            fork_branch(request, &producer, &main, format!("older-fork-{forks:03}"));
            forks += 1;
        }
    }
    fork_branch(request, &producer, &main, "active-fork".into());
    let (checkpoint, _) = capture_external_checkpoint(request, &producer);
    let source = immutable_source(&checkpoint, &medium);
    let (build, rss_built_mib) = (started.elapsed(), resident_mib());
    CeilingStore {
        medium,
        checkpoint,
        source,
        main,
        seed: seed[0],
        sample,
        detachable,
        forks,
        build,
        rss_built_mib,
    }
}

/// `REPEATS + 1` items of `items`, spread across it.
fn spread<T: Copy>(items: &[T]) -> impl Iterator<Item = T> + '_ {
    items
        .iter()
        .copied()
        .step_by(items.len() / (REPEATS + 1))
        .take(REPEATS + 1)
}

fn time_resolves(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    store: &CeilingStore,
) -> Vec<Duration> {
    spread(&store.sample)
        .map(|name| {
            let head = observe_branch(runtime, &store.main);
            timed(|| {
                runtime
                    .with_persistent_names_at_observation(request, &head.observation(), |view| {
                        let key = PersistentNameKey::Entity(name);
                        assert!(view
                            .resolve(key, &PersistentNameSelection::Exact)
                            .result
                            .is_ok());
                        let group = view.group_of_member(GROUP_SCHEME, &name).expect("grouped");
                        assert!(view
                            .resolve(group, &PersistentNameSelection::Exact)
                            .result
                            .is_ok());
                    })
                    .unwrap();
            })
            .1
        })
        .collect()
}

fn time_forks(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    store: &CeilingStore,
) -> Vec<Duration> {
    (0..=REPEATS)
        .map(|i| {
            timed(|| fork_branch(request, runtime, &store.main, format!("after-reopen-{i}"))).1
        })
        .collect()
}

fn time_detaches(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    store: &CeilingStore,
) -> Vec<Duration> {
    spread(&store.detachable)
        .map(|member| {
            let alone = vec![vec![(member, AggregateMemberRole::Anchor)]];
            timed(|| {
                publish_group_partition(request, runtime, &store.main, store.seed, "detach", alone)
            })
            .1
        })
        .collect()
}

/// Births into one populated partition: the first faults that partition's
/// cold records in, the rest find it resident.
fn time_births(
    request: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    store: &CeilingStore,
) -> Vec<Duration> {
    (0..=REPEATS)
        .map(|i| {
            let label = [format!("after-reopen-birth-{i}")];
            timed(|| {
                create_named_batch_in_partition(
                    request,
                    runtime,
                    &store.main,
                    PartitionId(1),
                    &label,
                )
            })
            .1
        })
        .collect()
}

#[test]
#[ignore = "manual release ceiling benchmark (100k names)"]
fn persistent_naming_release_ceiling_at_100k_names_records_reopen_fork_resolve_and_commit_costs() {
    ensure_tracker_installed();
    let host = worth_testkit_relational::shared_relational_certification_test_execution_policy::serial_request();
    let request = &ExecutionRequest::serial(&host);
    let store = build_ceiling_store(request);
    let pages = naming_pages(&store.source).len();
    let member_bytes: usize = store.source.members.values().map(|b| b.bytes().len()).sum();
    let manifest = store.checkpoint.bytes().len();

    let fresh = durable_runtime_with(&store.medium, u64::MAX / 4, naming_schema(true));
    let rss_before = resident_mib();
    let (checkpoint, source) = (store.checkpoint.clone(), store.source.clone());
    let ((runtime, window), reopen) =
        timed(|| measure(|| reopen_with(request, checkpoint, source, fresh)));
    let rss_after = resident_mib();

    let (resolve_first, resolve_median) =
        first_and_median(time_resolves(request, &runtime, &store));
    let (fork_first, fork_median) = first_and_median(time_forks(request, &runtime, &store));
    let (detach_first, detach_median) = first_and_median(time_detaches(request, &runtime, &store));
    let (birth_first, birth_median) = first_and_median(time_births(request, &runtime, &store));
    println!(
        "[7B-CEILING] names={} commits={} groups={} older_forks={} naming_pages={pages} checkpoint_member_bytes={member_bytes} manifest_bytes={manifest} build_s={:.1} rss_built_mib={:.1} reopen_ms={:.2} reopen_peak_live_mib={:.2} reopen_alloc_mib={:.2} rss_before_reopen_mib={rss_before:.1} rss_after_reopen_mib={rss_after:.1} resolve_first_us={resolve_first} resolve_median_us={resolve_median} fork_first_us={fork_first} fork_median_us={fork_median} detach_first_us={detach_first} detach_median_us={detach_median} birth_first_us={birth_first} birth_median_us={birth_median}",
        COMMITS * NAMES_PER_COMMIT,
        COMMITS + COMMITS / GROUPED_EVERY,
        COMMITS * NAMES_PER_COMMIT / GROUP_MEMBERS,
        store.forks,
        store.build.as_secs_f64(),
        store.rss_built_mib,
        reopen.as_secs_f64() * 1e3,
        window.peak_live_delta as f64 / 1048576.0,
        window.alloc_bytes as f64 / 1048576.0,
    );
}
