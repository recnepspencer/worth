//! The oracle's independent lifecycle script and two-stream reference prefixes.

pub(crate) use {std::collections::BTreeMap};
pub(crate) use {worth_execution::ExecutionAllocationPolicy, worth_execution::ExecutionRequest};
pub(crate) use {worth_relational::facade::branch::RelationalBranchArchiveDenial, worth_relational::facade::branch::RelationalBranchBasisDenial, worth_relational::facade::branch::RelationalBranchDeleteDenial, worth_relational::facade::branch::RelationalBranchDeletionOutcome, worth_relational::facade::branch::RelationalBranchTransitionStop, worth_relational::facade::branch::RelationalForkDenial};
pub(crate) use {worth_relational::facade::durability::RelationalDurabilityUnresolved};
pub(crate) use {worth_relational::facade::history::BranchId};
pub(crate) use {worth_relational::facade::mvcc::RelationalTransactionIntent};
pub(crate) use {worth_relational::facade::runtime::RelationalRuntime};
pub(crate) use {worth_relational::facade::transactions::RecordRef};

pub(crate) use {crate::workload, crate::workload::BranchStates, crate::workload::Step};

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
    pub(crate) records: BranchStates,
    pub(crate) postures: BTreeMap<String, Posture>,
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
            Self::Fork(worth_relational::facade::branch::RelationalForkDenial::DurablePublicationRefused(_))
                | Self::Archive(worth_relational::facade::branch::RelationalBranchArchiveDenial::DurablePublicationRefused(_))
                | Self::Delete(worth_relational::facade::branch::RelationalBranchDeleteDenial::DurablePublicationRefused(_))
        )
    }
}

pub(crate) fn stop<D>(
    stop: RelationalBranchTransitionStop<D>,
    denial: fn(D) -> LifecycleDenial,
) -> LifecycleStop {
    match stop {
        worth_relational::facade::branch::RelationalBranchTransitionStop::DurabilityUnresolved(fate) => {
            crate::branch_lifecycle::LifecycleStop::Unresolved(fate)
        }
        worth_relational::facade::branch::RelationalBranchTransitionStop::Denied(reason) => crate::branch_lifecycle::LifecycleStop::Denied(denial(reason)),
        worth_relational::facade::branch::RelationalBranchTransitionStop::RequestStopped(reason) => {
            crate::branch_lifecycle::LifecycleStop::RequestStopped(reason)
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
                .map_err(|reason| stop(reason, crate::branch_lifecycle::LifecycleDenial::Fork));
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
                .map_err(|reason| stop(reason, crate::branch_lifecycle::LifecycleDenial::Archive)),
            Self::RequestDeletion { .. } => {
                let (_, basis) = runtime.observe_branch(&identity).unwrap();
                let transaction = runtime
                    .begin_branch_transaction(&basis, worth_relational::facade::mvcc::RelationalTransactionIntent::ordinary())
                    .unwrap();
                let result = port.delete_branch(execution, &identity);
                if let Ok(outcome) = &result {
                    assert!(matches!(
                        outcome,
                        worth_relational::facade::branch::RelationalBranchDeletionOutcome::WaitingForActiveOperations(_)
                    ));
                }
                drop(transaction);
                result
                    .map(|_| ())
                    .map_err(|reason| stop(reason, crate::branch_lifecycle::LifecycleDenial::Delete))
            }
            Self::Delete { .. } => port
                .delete_branch(execution, &identity)
                .map(|outcome| {
                    assert!(matches!(
                        outcome,
                        worth_relational::facade::branch::RelationalBranchDeletionOutcome::Deleted(_)
                    ))
                })
                .map_err(|reason| stop(reason, crate::branch_lifecycle::LifecycleDenial::Delete)),
            _ => unreachable!(),
        }
    }
}

pub(crate) fn steps() -> Vec<Step> {
    let branch = |name: &str| BranchId(name.into());
    vec![
        crate::workload::Step::Fork {
            target: branch("archived"),
        },
        crate::workload::Step::Archive {
            branch: branch("archived"),
        },
        crate::workload::Step::Fork {
            target: branch("deleting"),
        },
        crate::workload::Step::RequestDeletion {
            branch: branch("deleting"),
        },
        crate::workload::Step::Fork {
            target: branch("deleted"),
        },
        crate::workload::Step::Delete {
            branch: branch("deleted"),
        },
    ]
}

pub(crate) fn artifacts_of(runtime: &RelationalRuntime) -> Vec<Vec<u8>> {
    runtime
        .durability()
        .branch_transition_artifacts(worth_execution::ExecutionAllocationPolicy::SystemAllocation)
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
        .branch_transition_artifacts(worth_execution::ExecutionAllocationPolicy::SystemAllocation);
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
        .branch_transition_artifacts(worth_execution::ExecutionAllocationPolicy::SystemAllocation);
    tail.covered_sequence().0 as usize + tail.len()
}

pub(crate) fn append_live(runtime: &RelationalRuntime, acknowledged: &[Vec<u8>]) -> Vec<Vec<u8>> {
    let tail = runtime
        .durability()
        .branch_transition_artifacts(worth_execution::ExecutionAllocationPolicy::SystemAllocation);
    let mut combined = acknowledged[..tail.covered_sequence().0 as usize].to_vec();
    combined.extend(tail.map(|artifact| artifact.unwrap().as_bytes().to_vec()));
    combined
}

pub(crate) fn state_of(runtime: &RelationalRuntime, records: &[RecordRef]) -> State {
    let postures = BRANCHES
        .into_iter()
        .map(|name| {
            let posture = match runtime.branch_identity(&BranchId(name.into())) {
                Err(_) => crate::branch_lifecycle::Posture::Absent,
                Ok(identity) => match runtime.observe_branch(&identity) {
                    Ok(_) => crate::branch_lifecycle::Posture::Live,
                    Err(worth_relational::facade::branch::RelationalBranchBasisDenial::ArchivedBranch(_)) => crate::branch_lifecycle::Posture::Archived,
                    Err(worth_relational::facade::branch::RelationalBranchBasisDenial::DeletingBranch(_)) => crate::branch_lifecycle::Posture::Deleting,
                    Err(error) => panic!("cannot observe {name}: {error:?}"),
                },
            };
            (name.to_owned(), posture)
        })
        .collect();
    State {
        records: crate::workload::states_of(runtime, records),
        postures,
    }
}

pub(crate) fn reference_prefixes(steps: &[Step], records: &[RecordRef]) -> ReferencePrefixes {
    let request = crate::workload::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let runtime = crate::workload::runtime_builder(crate::schema_transition::SOURCE)
        .build()
        .expect("valid native reference runtime configuration");
    let mut prefixes = std::collections::BTreeMap::new();
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
    script: &crate::workload::Script,
    commits: usize,
    transitions: usize,
) -> &[Step] {
    let (mut seen_commits, mut seen_transitions) = (0, 0);
    for (index, step) in script.steps.iter().enumerate() {
        if seen_commits == commits
            && seen_transitions == transitions
            && !matches!(step, crate::workload::Step::Checkpoint)
        {
            return &script.steps[index..];
        }
        seen_commits += usize::from(step.is_commit());
        seen_transitions += usize::from(step.is_lifecycle());
    }
    assert_eq!((seen_commits, seen_transitions), (commits, transitions));
    &[]
}