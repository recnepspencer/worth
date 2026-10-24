//! The oracle's workload: a seeded script of Relational operations, generated
//! once by a memory-only reference runtime. The reference's canonical
//! artifacts and per-branch commitments after each commit are the expected
//! result for any durable run of the same script.

use std::collections::BTreeMap;

use worth_execution::{
    CancellationSource, ExecutionAllocationPolicy, ExecutionRequest, SerialMemoryBudget,
    SerialRequest,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::mvcc::RelationalTransactionIntent;
use worth_relational::facade::runtime::{
    RelationalRuntime, RelationalRuntimeApi, RelationalRuntimeBuilder,
};
use worth_relational::facade::transactions::{
    CommitResult, RecordRef, TransactionCommitError, WorkerIntentBatch,
};

use super::batches;

/// One operation of a script.
#[derive(Clone, Debug)]
pub(crate) enum Step {
    Commit {
        branch: BranchId,
        batch: WorkerIntentBatch,
    },
    Fork {
        target: BranchId,
    },
    Checkpoint,
}

/// Every branch's records, rendered: for each identity the script ever
/// created, the record the branch head holds for it, or nothing.
pub(crate) type BranchStates = BTreeMap<String, Vec<String>>;

pub(crate) struct Script {
    pub(crate) steps: Vec<Step>,
    /// Artifact bytes by position (index 0 is position 1).
    pub(crate) artifacts: Vec<Vec<u8>>,
    /// Branch states after each prefix: index `k` holds the state after `k`
    /// commits.
    pub(crate) states: Vec<BranchStates>,
    pub(crate) records: Vec<RecordRef>,
}

pub(crate) const RUNTIME_NAME: &str = "relational-home-oracle";

pub(crate) fn runtime_builder() -> RelationalRuntimeBuilder {
    RelationalRuntimeApi::builder()
        .runtime_setup(|runtime| {
            runtime.runtime_name(RUNTIME_NAME);
        })
        .schema_setup(|schema| {
            schema.schema_registry(batches::schema_registry());
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
    /// Run `commits` seeded commits on a memory-only runtime, forking one
    /// branch partway and checkpointing every `checkpoint_every` commits.
    pub(crate) fn generate(seed: u64, commits: usize, checkpoint_every: usize) -> Self {
        let request = serial_request();
        let execution = ExecutionRequest::serial(&request);
        let runtime = runtime_builder().build();
        let mut generator = Xorshift(seed | 1);
        let main = BranchId("main".to_owned());
        let fork = BranchId("feature".to_owned());
        let mut live: BTreeMap<String, Vec<EntityId>> = BTreeMap::new();
        let mut steps = Vec::new();
        let mut records = Vec::new();
        let fork_at = commits / 3;
        for step in 0..commits {
            let branch = if step == fork_at {
                fork_branch(&runtime, &main, &fork);
                steps.push(Step::Fork {
                    target: fork.clone(),
                });
                let inherited = live.get(&main.0).cloned().unwrap_or_default();
                live.insert(fork.0.clone(), inherited);
                fork.clone()
            } else if step > fork_at && generator.next() % 3 == 0 {
                fork.clone()
            } else {
                main.clone()
            };
            let entities = live.entry(branch.0.clone()).or_default();
            let (batch, creates) = batches::next(&mut generator, entities, step);
            let outcome = commit(&runtime, &execution, &branch, batch.clone())
                .unwrap_or_else(|error| panic!("reference commit {step}: {error:?}"));
            if creates {
                entities.extend(
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
            steps.push(Step::Commit { branch, batch });
            if checkpoint_every > 0 && (step + 1) % checkpoint_every == 0 {
                steps.push(Step::Checkpoint);
            }
        }
        let artifacts = artifacts_of(&runtime);
        assert_eq!(
            artifacts.len(),
            commits,
            "every reference commit is retained"
        );
        records.sort();
        records.dedup();
        let states = replay_states(&steps, &records);
        Self {
            steps,
            artifacts,
            states,
            records,
        }
    }
}

/// Replay `steps` on a fresh reference runtime, recording every branch's
/// state after each commit. The replay allocates exactly as the generating
/// run did, or its commits fail.
fn replay_states(steps: &[Step], records: &[RecordRef]) -> Vec<BranchStates> {
    let request = serial_request();
    let execution = ExecutionRequest::serial(&request);
    let runtime = runtime_builder().build();
    let mut states = vec![states_of(&runtime, records)];
    for step in steps {
        match step {
            Step::Commit { branch, batch } => {
                commit(&runtime, &execution, branch, batch.clone())
                    .expect("the replayed reference commit");
                states.push(states_of(&runtime, records));
            }
            Step::Fork { target } => {
                fork_branch(&runtime, &BranchId("main".to_owned()), target);
            }
            Step::Checkpoint => {}
        }
    }
    states
}

pub(crate) fn fork_branch(runtime: &RelationalRuntime, source: &BranchId, target: &BranchId) {
    let (_, basis) = runtime
        .observe_fork_source(source)
        .expect("the fork source is observable");
    runtime
        .fork_branch(target.clone(), basis)
        .expect("the fork is admitted");
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

/// Every registered branch's head, read record by record.
pub(crate) fn states_of(runtime: &RelationalRuntime, records: &[RecordRef]) -> BranchStates {
    ["main", "feature"]
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
                            .read_entity(*entity)
                            .expect("entity read")
                            .effective()
                    ),
                    RecordRef::Relation(relation) => format!(
                        "{:?}",
                        reader
                            .read_relation(*relation)
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
