//! Attachment, original identity issuance and branch reservations have one order.
use super::composite::CompositePublicationAuthorization;
use super::*;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::DurableSignalBranchIdentity;

/// An executing callback never leaves a nameless reservation after unwinding.
pub(super) struct RegisteredSignalPublication<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord + 'static,
    T: Copy + Ord + 'static,
{
    pub(super) state: Arc<SignalDurablePublicationState<D, I, T>>,
    pub(super) original: SignalBranchAdvanceIdentity,
    pub(super) new_admission: DurableSignalAdmissionIdentity,
    pub(super) recorded_nodes: worth_execution::ExecutionOrderedMap<u32, u32>,
    pub(super) association: Option<worth_foundational::facade::CompositeAttemptAssociation>,
    armed: bool,
}
impl<D, I, T> Drop for RegisteredSignalPublication<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord + 'static,
    T: Copy + Ord + 'static,
{
    fn drop(&mut self) {
        if self.armed {
            // No media method is reachable while this reservation is Preparing.
            // Poison recovery here releases owned resources only; subsequent
            // operation entry still reports the poisoned publication state.
            let mut table = self
                .state
                .tasks
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let slot = table.tasks.remove(&self.original);
            if let Some(slot) = &slot {
                let guard = slot
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                table.branches.remove(&guard.branches.target());
                match guard.branches {
                    ReservedSignalBranches::Advanced(_) => {}
                    ReservedSignalBranches::Forked { source, .. } => {
                        table.branches.remove(&source);
                    }
                }
            }
            drop(table);
            drop(slot);
        }
    }
}

impl<D, I, T> RegisteredSignalPublication<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(super) fn seal(
        mut self,
        phase: TaskPhase<D, I, T>,
    ) -> Result<SignalDurableAdvanceTask, SignalDurableAdvanceDenial> {
        let slot = self.state.task(self.original)?;
        slot.lock()
            .map_err(|_| SignalDurableAdvanceDenial::PublicationStatePoisoned)?
            .phase = phase;
        self.armed = false;
        Ok(self.state.handle(self.original))
    }
}
impl<D, I, T> SignalDurablePublicationState<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(super) fn register_prepared(
        self: &Arc<Self>,
        execution: &ExecutionRequest<'_, '_>,
        branches: ReservedSignalBranches,
        authorization: Option<CompositePublicationAuthorization>,
    ) -> Result<RegisteredSignalPublication<D, I, T>, SignalDurableAdvanceDenial> {
        use SignalDurableAdvanceDenial as Dn;
        execution.consult().map_err(Dn::Request)?;
        if let Some(original) = *self
            .unsettled
            .lock()
            .map_err(|_| Dn::PublicationStatePoisoned)?
        {
            return Err(Dn::PublicationUnsettled { original });
        }
        let memory = task_storage::reserve::<D, I, T>(execution, branches)?;
        let installed = self.admit_publication_mode(authorization.as_ref())?;
        let mut table = self
            .tasks
            .lock()
            .map_err(|_| Dn::PublicationStatePoisoned)?;
        if table.capture_active {
            return Err(Dn::CheckpointCaptureInProgress {
                owner: self.binding().owner(),
            });
        }
        for branch in branches.identities() {
            if let Some(original) = table.branches.get(&branch) {
                return Err(Dn::BranchReserved {
                    original: *original,
                });
            }
        }
        let (association, composite_grant) = match authorization {
            Some(authorization) => (Some(authorization.association), Some(authorization.grant)),
            None => (None, None),
        };
        let mut source = self
            .source
            .lock()
            .map_err(|_| Dn::PublicationStatePoisoned)?;
        let (original, new_admission) = source
            .frontiers
            .issue_advance_and_admission()
            .map_err(Dn::Readmission)?;
        let recorded_nodes =
            source
                .frontiers
                .node_issuance_root(DurableSignalBranchIdentity::from_recorded(
                    self.binding().owner(),
                    branches.target().0,
                ));
        let slot = Arc::new(Mutex::new(TaskSlot {
            branches,
            resource: Some(LinearResource::mint(original, &self.authority)),
            phase: TaskPhase::Preparing,
            composite_grant,
            _memory: memory,
        }));
        for branch in branches.identities() {
            table.branches.insert(branch, original);
        }
        table.tasks.insert(original, slot);
        drop(source);
        drop(table);
        drop(installed);
        Ok(RegisteredSignalPublication {
            state: Arc::clone(self),
            original,
            new_admission,
            recorded_nodes,
            association,
            armed: true,
        })
    }
}
