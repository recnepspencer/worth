//! The oracle's workload: a seeded script of Relational operations, generated
//! once by a memory-only reference runtime. The reference's canonical
//! artifacts and per-branch record states after each commit are the expected
//! result for any durable run of the same script: canonical artifacts are
//! runtime-neutral (D11), so a durable run, recovered or not, must reproduce
//! the reference's artifacts byte for byte at every position.

use std::collections::BTreeMap;

use worth_execution::{
    CancellationSource, ExecutionAllocationPolicy, ExecutionRequest, SerialMemoryBudget,
    SerialRequest,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::mvcc::RelationalTransactionIntent;
use worth_relational::facade::runtime::{
    RelationalRuntime, RelationalRuntimeApi, RelationalRuntimeBuilder,
};
use worth_relational::facade::schema::SchemaVersionId;
use worth_relational::facade::transactions::{
    CommitResult, RecordRef, TransactionCommitError, WorkerIntentBatch,
};

use super::{batches, schema_transition};

/// One operation of a script.
#[derive(Clone, Debug)]
pub(crate) enum Step {
    Commit {
        branch: BranchId,
        batch: WorkerIntentBatch,
    },
    /// A commit that also moves its branch from schema v1 to v2.
    Transition {
        branch: BranchId,
        batch: WorkerIntentBatch,
    },
    Fork {
        target: BranchId,
    },
    Archive {
        branch: BranchId,
    },
    RequestDeletion {
        branch: BranchId,
    },
    Delete {
        branch: BranchId,
    },
    Checkpoint,
}

impl Step {
    pub(crate) fn is_commit(&self) -> bool {
        matches!(self, Self::Commit { .. } | Self::Transition { .. })
    }

    /// Execute a commit step on `runtime`. Panics on a fork or checkpoint.
    pub(crate) fn commit(
        &self,
        runtime: &RelationalRuntime,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<CommitResult, TransactionCommitError> {
        match self {
            Self::Commit { branch, batch } => commit(runtime, execution, branch, batch.clone()),
            Self::Transition { branch, batch } => {
                schema_transition::commit(runtime, execution, branch, batch.clone())
            }
            _ => panic!("not a commit step"),
        }
    }
}

/// Every branch's records, rendered: for each identity the script ever
/// created, the record the branch head holds for it, or nothing.
pub(crate) type BranchStates = BTreeMap<String, Vec<String>>;

pub(crate) struct Script {
    pub(crate) steps: Vec<Step>,
    /// Branch states after each prefix: index `k` holds the state after `k`
    /// commits.
    pub(crate) states: Vec<BranchStates>,
    pub(crate) records: Vec<RecordRef>,
    /// The reference's canonical artifact bytes; index `k` is position `k + 1`.
    pub(crate) artifacts: Vec<Vec<u8>>,
    pub(crate) transitions: Vec<Vec<u8>>,
    pub(crate) prefixes: super::branch_lifecycle::ReferencePrefixes,
}

pub(crate) const RUNTIME_NAME: &str = "relational-home-oracle";

/// The oracle runtime, declaring the schema registry at `schema`.
pub(crate) fn runtime_builder(schema: SchemaVersionId) -> RelationalRuntimeBuilder {
    RelationalRuntimeApi::builder()
        .runtime_setup(|runtime| {
            runtime.runtime_name(RUNTIME_NAME);
        })
        .schema_setup(move |setup| {
            setup.schema_registry(schema_transition::registry(schema));
        })
}

pub(crate) fn serial_request() -> SerialRequest {
    SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    )
}

impl Script {
    /// Run `commits` seeded commits on a memory-only runtime, exercising all
    /// lifecycle transitions and checkpointing every `checkpoint_every` commits.
    pub(crate) fn generate(seed: u64, commits: usize, checkpoint_every: usize) -> Self {
        let request = serial_request();
        let execution = ExecutionRequest::serial(&request);
        let runtime = runtime_builder(schema_transition::SOURCE).build();
        let mut generator = Xorshift(seed | 1);
        let main = BranchId("main".to_owned());
        let fork = BranchId("feature".to_owned());
        let mut live: BTreeMap<String, batches::BranchLive> = BTreeMap::new();
        let mut steps = Vec::new();
        let mut records = Vec::new();
        let fork_at = commits / 3;
        let transition_at = 2 * commits / 3;
        for step in 0..commits {
            let branch = if step == transition_at {
                main.clone()
            } else if step == fork_at {
                let transition = Step::Fork {
                    target: fork.clone(),
                };
                transition
                    .lifecycle(&runtime, &execution)
                    .expect("reference fork");
                steps.push(transition);
                let inherited = live.get(&main.0).cloned().unwrap_or_default();
                live.insert(fork.0.clone(), inherited);
                fork.clone()
            } else if step > fork_at && generator.next() % 3 == 0 {
                fork.clone()
            } else {
                main.clone()
            };
            if step == commits / 2 {
                for lifecycle in super::branch_lifecycle::steps() {
                    lifecycle
                        .lifecycle(&runtime, &execution)
                        .expect("reference lifecycle");
                    steps.push(lifecycle);
                }
            }
            let branch_live = live.entry(branch.0.clone()).or_default();
            let (next, creates) = if step == transition_at {
                let batch = batches::create(step);
                (Step::Transition { branch, batch }, true)
            } else {
                let (batch, creates) = batches::next(&mut generator, branch_live, step);
                (Step::Commit { branch, batch }, creates)
            };
            let outcome = next
                .commit(&runtime, &execution)
                .unwrap_or_else(|error| panic!("reference commit {step}: {error:?}"));
            if creates {
                branch_live
                    .entities
                    .extend(
                        outcome
                            .changed_records
                            .iter()
                            .filter_map(|record| match record {
                                RecordRef::Entity(entity) => Some(*entity),
                                _ => None,
                            }),
                    );
            }
            records.extend(outcome.changed_records.iter().cloned());
            steps.push(next);
            if checkpoint_every > 0 && (step + 1) % checkpoint_every == 0 {
                steps.push(Step::Checkpoint);
            }
        }
        let artifacts = artifacts_of(&runtime);
        let transitions = super::branch_lifecycle::artifacts_of(&runtime);
        assert_eq!(
            artifacts.len(),
            commits,
            "every reference commit is retained"
        );
        records.sort();
        records.dedup();
        let states = replay_states(&steps, &records);
        let prefixes = super::branch_lifecycle::reference_prefixes(&steps, &records);
        Self {
            steps,
            states,
            records,
            artifacts,
            transitions,
            prefixes,
        }
    }
}

/// Replay `steps` on a fresh reference runtime, recording every branch's
/// state after each commit. The replay allocates exactly as the generating
/// run did, or its commits fail.
fn replay_states(steps: &[Step], records: &[RecordRef]) -> Vec<BranchStates> {
    let request = serial_request();
    let execution = ExecutionRequest::serial(&request);
    let runtime = runtime_builder(schema_transition::SOURCE).build();
    let mut states = vec![states_of(&runtime, records)];
    for step in steps {
        match step {
            Step::Commit { .. } | Step::Transition { .. } => {
                step.commit(&runtime, &execution)
                    .expect("the replayed reference commit");
                states.push(states_of(&runtime, records));
            }
            Step::Checkpoint => {}
            _ => {
                step.lifecycle(&runtime, &execution)
                    .expect("reference lifecycle");
            }
        }
    }
    states
}

pub(crate) fn commit(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    branch: &BranchId,
    batch: WorkerIntentBatch,
) -> Result<CommitResult, TransactionCommitError> {
    let identity = runtime
        .branch_identity(branch)
        .expect("the branch is registered");
    let (_, basis) = runtime
        .observe_branch(&identity)
        .expect("the branch is owner-observable");
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .expect("owner-admitted transaction");
    transaction
        .push_batch(batch, ExecutionAllocationPolicy::SystemAllocation)
        .expect("the batch stages");
    transaction.commit(execution, runtime)
}

/// Every retained canonical commit, as artifact bytes in position order.
pub(crate) fn artifacts_of(runtime: &RelationalRuntime) -> Vec<Vec<u8>> {
    runtime
        .durability()
        .canonical_commit_artifacts(ExecutionAllocationPolicy::SystemAllocation)
        .map(|artifact| {
            artifact
                .expect("retained commits encode")
                .as_bytes()
                .to_vec()
        })
        .collect()
}

/// Inspect only newly performed positions while Store may be indeterminate.
/// The acknowledged prefix is retained by the oracle before arming the fault;
/// full byte agreement is checked independently after reopening the home.
pub(crate) fn append_live_artifacts(
    runtime: &RelationalRuntime,
    before: &[Vec<u8>],
) -> Vec<Vec<u8>> {
    let artifacts = runtime
        .durability()
        .canonical_commit_artifacts(ExecutionAllocationPolicy::SystemAllocation);
    assert!(
        artifacts.len() >= before.len(),
        "live selection lost an acknowledged index prefix"
    );
    let mut now = before.to_vec();
    now.extend(
        artifacts
            .after_position(worth_relational::facade::publication::PatchStreamPosition(
                before.len() as u64,
            ))
            .map(|artifact| {
                artifact
                    .expect("newly performed artifact encodes")
                    .as_bytes()
                    .to_vec()
            }),
    );
    now
}

/// Every registered branch's head, read record by record.
pub(crate) fn states_of(runtime: &RelationalRuntime, records: &[RecordRef]) -> BranchStates {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    super::branch_lifecycle::BRANCHES
        .into_iter()
        .filter_map(|name| {
            let identity = runtime.branch_identity(&BranchId(name.to_owned())).ok()?;
            let (_, basis) = runtime.observe_branch(&identity).ok()?;
            let mut reader = runtime
                .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
                .expect("a read transaction");
            let rendered = records
                .iter()
                .map(|record| match record {
                    RecordRef::Entity(entity) => format!(
                        "{:?}",
                        reader
                            .read_entity(execution, *entity)
                            .expect("entity read")
                            .effective()
                    ),
                    RecordRef::Relation(relation) => format!(
                        "{:?}",
                        reader
                            .read_relation(execution, *relation)
                            .expect("relation read")
                            .effective()
                    ),
                })
                .collect();
            Some((name.to_owned(), rendered))
        })
        .collect()
}

/// A small deterministic generator, so a failing seed reproduces exactly.
pub(crate) struct Xorshift(pub(crate) u64);

impl Xorshift {
    pub(crate) fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    pub(crate) fn index(&mut self, length: usize) -> usize {
        (self.next() % length as u64) as usize
    }
}
