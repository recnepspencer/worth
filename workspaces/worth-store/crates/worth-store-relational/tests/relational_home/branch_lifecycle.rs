//! The oracle's independent lifecycle script and two-stream reference prefixes.

use std::collections::BTreeMap;
use worth_execution::{ExecutionAllocationPolicy, ExecutionRequest};
use worth_relational::facade::branch::{
    RelationalBranchArchiveDenial, RelationalBranchBasisDenial, RelationalBranchDeleteDenial,
    RelationalBranchDeletionOutcome, RelationalBranchTransitionStop, RelationalForkDenial,
};
use worth_relational::facade::durability::RelationalDurabilityUnresolved;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::mvcc::RelationalTransactionIntent;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::transactions::RecordRef;

use super::workload::{self, BranchStates, Step};

pub(crate) const BRANCHES: [&str; 5] = ["main", "feature", "archived", "deleting", "deleted"];

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Posture {
    Absent,
    Live,
    Archived,
    Deleting,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct State {
    records: BranchStates,
    postures: BTreeMap<String, Posture>,
}

pub(crate) type ReferencePrefixes = BTreeMap<(usize, usize), State>;

#[derive(Debug)]
pub(crate) enum LifecycleStop {
    Denied(LifecycleDenial),
    RequestStopped(worth_execution::WorkCeilingDenial),
    Unresolved(RelationalDurabilityUnresolved),
}

#[derive(Debug)]
pub(crate) enum LifecycleDenial {
    Fork(RelationalForkDenial),
    Archive(RelationalBranchArchiveDenial),
    Delete(RelationalBranchDeleteDenial),
}

impl LifecycleDenial {
    pub(crate) fn publication_refused(&self) -> bool {
        matches!(
            self,
            Self::Fork(RelationalForkDenial::DurablePublicationRefused(_))
                | Self::Archive(RelationalBranchArchiveDenial::DurablePublicationRefused(_))
                | Self::Delete(RelationalBranchDeleteDenial::DurablePublicationRefused(_))
        )
    }
}

fn stop<D>(
    stop: RelationalBranchTransitionStop<D>,
    denial: fn(D) -> LifecycleDenial,
) -> LifecycleStop {
    match stop {
        RelationalBranchTransitionStop::DurabilityUnresolved(fate) => {
            LifecycleStop::Unresolved(fate)
        }
        RelationalBranchTransitionStop::Denied(reason) => LifecycleStop::Denied(denial(reason)),
        RelationalBranchTransitionStop::RequestStopped(reason) => {
            LifecycleStop::RequestStopped(reason)
        }
    }
}

impl Step {
    pub(crate) fn is_lifecycle(&self) -> bool {
        !self.is_commit() && !matches!(self, Self::Checkpoint)
    }

    pub(crate) fn lifecycle(
        &self,
        runtime: &RelationalRuntime,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(), LifecycleStop> {
        if let Self::Fork { target } = self {
            let (_, basis) = runtime
                .observe_fork_source(&BranchId("main".into()))
                .unwrap();
            return runtime
                .fork_branch(execution, target.clone(), basis)
                .map(|_| ())
                .map_err(|reason| stop(reason, LifecycleDenial::Fork));
        }
        let (Self::Archive { branch } | Self::RequestDeletion { branch } | Self::Delete { branch }) =
            self
        else {
            panic!("not a lifecycle step")
        };
        let identity = runtime.branch_identity(branch).unwrap();
        let port = runtime.owner_component_services().lifecycle_port();
        match self {
            Self::Archive { .. } => port
                .archive_branch(execution, &identity)
                .map(|_| ())
                .map_err(|reason| stop(reason, LifecycleDenial::Archive)),
            Self::RequestDeletion { .. } => {
                let (_, basis) = runtime.observe_branch(&identity).unwrap();
                let transaction = runtime
                    .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
                    .unwrap();
                let result = port.delete_branch(execution, &identity);
                if let Ok(outcome) = &result {
                    assert!(matches!(
                        outcome,
                        RelationalBranchDeletionOutcome::WaitingForActiveOperations(_)
                    ));
                }
                drop(transaction);
                result
                    .map(|_| ())
                    .map_err(|reason| stop(reason, LifecycleDenial::Delete))
            }
            Self::Delete { .. } => port
                .delete_branch(execution, &identity)
                .map(|outcome| {
                    assert!(matches!(
                        outcome,
                        RelationalBranchDeletionOutcome::Deleted(_)
                    ))
                })
                .map_err(|reason| stop(reason, LifecycleDenial::Delete)),
            _ => unreachable!(),
        }
    }
}

pub(crate) fn steps() -> Vec<Step> {
    let branch = |name: &str| BranchId(name.into());
    vec![
        Step::Fork {
            target: branch("archived"),
        },
        Step::Archive {
            branch: branch("archived"),
        },
        Step::Fork {
            target: branch("deleting"),
        },
        Step::RequestDeletion {
            branch: branch("deleting"),
        },
        Step::Fork {
            target: branch("deleted"),
        },
        Step::Delete {
            branch: branch("deleted"),
        },
    ]
}

pub(crate) fn artifacts_of(runtime: &RelationalRuntime) -> Vec<Vec<u8>> {
    runtime
        .durability()
        .branch_transition_artifacts(ExecutionAllocationPolicy::SystemAllocation)
        .map(|artifact| artifact.unwrap().as_bytes().to_vec())
        .collect()
}

/// Historical artifacts are read from their authority, not retained in the
/// runtime checkpoint. Independently compare the live tail to that journal.
pub(crate) fn artifacts_of_home(
    home: &worth_store_relational::RelationalStoreHome,
) -> Vec<Vec<u8>> {
    let journal = home.certification_serving().branch_transitions().unwrap();
    let head = journal.head().unwrap().map_or(0, |entry| entry.position());
    let durable = (1..=head)
        .map(|sequence| journal.read(sequence, 1 << 20).unwrap().into_bytes())
        .collect::<Vec<_>>();
    let live = home
        .runtime()
        .durability()
        .branch_transition_artifacts(ExecutionAllocationPolicy::SystemAllocation);
    let covered = live.covered_sequence().0 as usize;
    let tail = live
        .map(|artifact| artifact.unwrap().as_bytes().to_vec())
        .collect::<Vec<_>>();
    assert_eq!(
        tail,
        durable[covered..],
        "live transition tail differs from durable history"
    );
    durable
}

pub(crate) fn live_count(runtime: &RelationalRuntime) -> usize {
    let tail = runtime
        .durability()
        .branch_transition_artifacts(ExecutionAllocationPolicy::SystemAllocation);
    tail.covered_sequence().0 as usize + tail.len()
}

pub(crate) fn append_live(runtime: &RelationalRuntime, acknowledged: &[Vec<u8>]) -> Vec<Vec<u8>> {
    let tail = runtime
        .durability()
        .branch_transition_artifacts(ExecutionAllocationPolicy::SystemAllocation);
    let mut combined = acknowledged[..tail.covered_sequence().0 as usize].to_vec();
    combined.extend(tail.map(|artifact| artifact.unwrap().as_bytes().to_vec()));
    combined
}

pub(crate) fn state_of(runtime: &RelationalRuntime, records: &[RecordRef]) -> State {
    let postures = BRANCHES
        .into_iter()
        .map(|name| {
            let posture = match runtime.branch_identity(&BranchId(name.into())) {
                Err(_) => Posture::Absent,
                Ok(identity) => match runtime.observe_branch(&identity) {
                    Ok(_) => Posture::Live,
                    Err(RelationalBranchBasisDenial::ArchivedBranch(_)) => Posture::Archived,
                    Err(RelationalBranchBasisDenial::DeletingBranch(_)) => Posture::Deleting,
                    Err(error) => panic!("cannot observe {name}: {error:?}"),
                },
            };
            (name.to_owned(), posture)
        })
        .collect();
    State {
        records: workload::states_of(runtime, records),
        postures,
    }
}

pub(crate) fn reference_prefixes(steps: &[Step], records: &[RecordRef]) -> ReferencePrefixes {
    let request = workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let runtime = workload::runtime_builder(super::schema_transition::SOURCE).build();
    let mut prefixes = BTreeMap::new();
    let (mut commits, mut transitions) = (0, 0);
    prefixes.insert((commits, transitions), state_of(&runtime, records));
    for step in steps {
        if step.is_commit() {
            step.commit(&runtime, &execution).unwrap();
            commits += 1;
        } else if step.is_lifecycle() {
            step.lifecycle(&runtime, &execution).unwrap();
            transitions += 1;
        }
        prefixes.insert((commits, transitions), state_of(&runtime, records));
    }
    prefixes
}

pub(crate) fn steps_after(
    script: &workload::Script,
    commits: usize,
    transitions: usize,
) -> &[Step] {
    let (mut seen_commits, mut seen_transitions) = (0, 0);
    for (index, step) in script.steps.iter().enumerate() {
        if seen_commits == commits
            && seen_transitions == transitions
            && !matches!(step, Step::Checkpoint)
        {
            return &script.steps[index..];
        }
        seen_commits += usize::from(step.is_commit());
        seen_transitions += usize::from(step.is_lifecycle());
    }
    assert_eq!((seen_commits, seen_transitions), (commits, transitions));
    &[]
}
