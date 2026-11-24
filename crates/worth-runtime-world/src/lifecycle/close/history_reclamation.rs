//! Durable history retirement publication and pin reclamation helpers.
use crate::history::{HistoryReclamationDenial, PendingDurableHistoryRetirement};
use crate::identity::CompositeCommitIdentity;
use crate::lifecycle::owner::RuntimeWorldOwnerRoot;

/// Keep each `HistoryRetired` frame well under the 64 KiB artifact ceiling.
const HISTORY_RETIRED_CHUNK_SIZE: usize = 1000;

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    pub(super) fn retire_changed_history_inner(
        &self,
        caller_execution: Option<&worth_execution::ExecutionRequest<'_, '_>>,
    ) -> Result<Vec<CompositeCommitIdentity>, HistoryReclamationDenial> {
        let _operation = self
            .reserve_recovery_operation_if_open_and_bootstrapped()
            .map_err(|_| {
                HistoryReclamationDenial::OwnerUnavailable(
                    crate::lifecycle::RuntimeWorldOwnerUnavailable::new(),
                )
            })?;
        if let Some(binding) = self.state.durable.as_ref() {
            let pending = self.state.history.begin_durable_retire_changed_history();
            let (reclaimed, _) =
                self.drive_durable_history_retirement(binding, pending, caller_execution)?;
            Ok(reclaimed)
        } else {
            let retired = self.state.history.retire_changed_history();
            Ok(self.reclaim_retired_entries(retired))
        }
    }

    pub(super) fn release_retired_branch_history_inner(
        &self,
        from: &CompositeCommitIdentity,
        caller_execution: Option<&worth_execution::ExecutionRequest<'_, '_>>,
    ) -> Result<crate::history::RetiredBranchHistoryRelease, HistoryReclamationDenial> {
        let _operation = self
            .reserve_recovery_operation_if_open_and_bootstrapped()
            .map_err(|_| {
                HistoryReclamationDenial::OwnerUnavailable(
                    crate::lifecycle::RuntimeWorldOwnerUnavailable::new(),
                )
            })?;
        if let Some(binding) = self.state.durable.as_ref() {
            let pending = self
                .state
                .history
                .begin_durable_release_retired_branch_history(from)
                .map_err(HistoryReclamationDenial::Catalog)?;
            let (reclaimed, held) =
                self.drive_durable_history_retirement(binding, pending, caller_execution)?;
            Ok(crate::history::RetiredBranchHistoryRelease::new(
                reclaimed, held,
            ))
        } else {
            let (released, held) = self
                .state
                .history
                .release_retired_branch_history(from)
                .map_err(HistoryReclamationDenial::Catalog)?;
            Ok(crate::history::RetiredBranchHistoryRelease::new(
                self.reclaim_retired_entries(released),
                held,
            ))
        }
    }

    pub(super) fn drive_durable_history_retirement(
        &self,
        binding: &std::sync::Arc<crate::durability::WorldDurableMediumBinding>,
        pending: PendingDurableHistoryRetirement,
        caller_execution: Option<&worth_execution::ExecutionRequest<'_, '_>>,
    ) -> Result<(Vec<CompositeCommitIdentity>, Option<CompositeCommitIdentity>), HistoryReclamationDenial>
    {
        self.drive_durable_history_retirement_with_chunk_size_and_request(
            binding,
            pending,
            HISTORY_RETIRED_CHUNK_SIZE,
            caller_execution,
        )
    }

    #[cfg(test)]
    pub(crate) fn drive_durable_history_retirement_with_chunk_size(
        &self,
        binding: &std::sync::Arc<crate::durability::WorldDurableMediumBinding>,
        pending: PendingDurableHistoryRetirement,
        chunk_size: usize,
    ) -> Result<(Vec<CompositeCommitIdentity>, Option<CompositeCommitIdentity>), HistoryReclamationDenial>
    {
        self.drive_durable_history_retirement_with_chunk_size_and_request(
            binding,
            pending,
            chunk_size,
            None,
        )
    }

    fn drive_durable_history_retirement_with_chunk_size_and_request(
        &self,
        binding: &std::sync::Arc<crate::durability::WorldDurableMediumBinding>,
        mut pending: PendingDurableHistoryRetirement,
        chunk_size: usize,
        caller_execution: Option<&worth_execution::ExecutionRequest<'_, '_>>,
    ) -> Result<(Vec<CompositeCommitIdentity>, Option<CompositeCommitIdentity>), HistoryReclamationDenial>
    {
        use crate::durability::{
            WorldArtifactInput, WorldDurableArtifact, WorldDurablePublicationStop as Stop,
        };
        use worth_foundational::facade::DurableCompositeCommitIdentity;
        let chunk_size = chunk_size.max(1);
        if pending.remaining().is_empty() {
            let held = pending.finish_all();
            return Ok((Vec::new(), held));
        }
        let owner = binding.binding().owner();
        let retiring_incarnation = binding.binding().incarnation();
        let policy = self.state.execution.request_policy();
        let fallback_serial = worth_execution::SerialRequest::from_policy(
            &policy,
            worth_execution::CancellationSource::new().token(),
            None,
        );
        let execution = caller_execution
            .copied()
            .unwrap_or_else(|| worth_execution::ExecutionRequest::serial(&fallback_serial));
        let mut reclaimed = Vec::with_capacity(pending.remaining().len());
        while !pending.remaining().is_empty() {
            let chunk_len = pending.remaining().len().min(chunk_size);
            let durable_commits: Vec<DurableCompositeCommitIdentity> = pending.remaining()
                [..chunk_len]
                .iter()
                .map(|commit| DurableCompositeCommitIdentity::from_recorded(owner, commit.ordinal()))
                .collect();
            let subject = match WorldDurableArtifact::from_input(
                WorldArtifactInput::HistoryRetired {
                    owner,
                    commits: &durable_commits,
                    retiring_incarnation,
                },
                &execution,
            ) {
                Ok(subject) => subject,
                Err(_) => {
                    let resume_from = pending.next_resume_commit();
                    pending.release_remaining_proven_no_effect();
                    return Err(frame_failure_denial(reclaimed, resume_from, false));
                }
            };
            loop {
                match binding.publish(&subject, &execution) {
                    Ok(_committed) => {
                        let entries = pending.commit_next(chunk_len);
                        reclaimed.extend(self.reclaim_retired_entries(entries));
                        break;
                    }
                    Err(Stop::Pending) => std::thread::yield_now(),
                    Err(Stop::Refused(_) | Stop::Busy | Stop::BlockedByPriorBarrier { .. }) => {
                        let resume_from = pending.next_resume_commit();
                        pending.release_remaining_proven_no_effect();
                        return Err(frame_failure_denial(reclaimed, resume_from, false));
                    }
                    Err(Stop::Unresolved | Stop::EvidenceRequired) => {
                        let resume_from = pending.next_resume_commit();
                        pending.fence_indeterminate_and_release_suffix(chunk_len);
                        return Err(frame_failure_denial(reclaimed, resume_from, true));
                    }
                }
            }
        }
        let held = pending.finish_all();
        Ok((reclaimed, held))
    }

    /// Drops removed history entries, releases any durable checkpoint origins
    /// for those commits, and reclaims the pins only they held.
    pub(super) fn reclaim_retired_entries(
        &self,
        retired: crate::history::RetiredHistoryEntries,
    ) -> Vec<crate::identity::CompositeCommitIdentity> {
        let mut keys: Vec<_> = retired
            .iter()
            .filter_map(|slot| slot.get())
            .flat_map(|entry| entry.retention_keys())
            .collect();
        let identities: Vec<_> = retired
            .iter()
            .filter_map(|slot| slot.get())
            .map(|entry| entry.commit().identity().clone())
            .collect();
        let commit_results: Vec<_> = retired
            .iter()
            .filter_map(|slot| slot.get())
            .filter_map(|entry| entry.live_relational_commit_result())
            .collect();
        let ordinals: std::collections::BTreeSet<u64> =
            identities.iter().map(|identity| identity.ordinal()).collect();
        drop(retired);
        if let Some(relational) = &self.state.relational {
            let basis_port = relational.basis_port();
            for result in &commit_results {
                let _ = basis_port.release_commit_result_snapshot(result);
            }
        }
        drop(commit_results);
        if let Some(binding) = &self.state.durable {
            keys.extend(binding.dispose_checkpoint_commits(&ordinals));
        }
        let mut seen = std::collections::HashSet::with_capacity(keys.len());
        keys.retain(|key| seen.insert(key.key.clone()));
        self.state
            .retention
            .reclaim_keys(&keys, keys.len())
            .expect("retired history pins belong to this owner");
        identities
    }
}

fn frame_failure_denial(
    reclaimed: Vec<CompositeCommitIdentity>,
    resume_from: Option<CompositeCommitIdentity>,
    indeterminate: bool,
) -> HistoryReclamationDenial {
    if reclaimed.is_empty() {
        HistoryReclamationDenial::OwnerUnavailable(
            crate::lifecycle::RuntimeWorldOwnerUnavailable::new(),
        )
    } else {
        HistoryReclamationDenial::PartialPublication {
            reclaimed,
            resume_from,
            indeterminate,
        }
    }
}
