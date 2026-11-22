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
    pub(super) fn drive_durable_history_retirement(
        &self,
        binding: &std::sync::Arc<crate::durability::WorldDurableMediumBinding>,
        pending: PendingDurableHistoryRetirement,
    ) -> Result<(Vec<CompositeCommitIdentity>, Option<CompositeCommitIdentity>), HistoryReclamationDenial>
    {
        self.drive_durable_history_retirement_with_chunk_size(
            binding,
            pending,
            HISTORY_RETIRED_CHUNK_SIZE,
        )
    }

    pub(crate) fn drive_durable_history_retirement_with_chunk_size(
        &self,
        binding: &std::sync::Arc<crate::durability::WorldDurableMediumBinding>,
        mut pending: PendingDurableHistoryRetirement,
        chunk_size: usize,
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
        let serial = worth_execution::SerialRequest::from_policy(
            &policy,
            worth_execution::CancellationSource::new().token(),
            None,
        );
        let execution = worth_execution::ExecutionRequest::serial(&serial);
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
                    pending.release_remaining_proven_no_effect();
                    return Err(frame_failure_denial(reclaimed, false));
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
                    Err(Stop::Refused(_) | Stop::Busy) => {
                        pending.release_remaining_proven_no_effect();
                        return Err(frame_failure_denial(reclaimed, false));
                    }
                    Err(Stop::Unresolved | Stop::EvidenceRequired) => {
                        pending.fence_indeterminate_and_release_suffix(chunk_len);
                        return Err(frame_failure_denial(reclaimed, true));
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
    indeterminate: bool,
) -> HistoryReclamationDenial {
    if reclaimed.is_empty() {
        HistoryReclamationDenial::OwnerUnavailable(
            crate::lifecycle::RuntimeWorldOwnerUnavailable::new(),
        )
    } else {
        HistoryReclamationDenial::PartialPublication {
            reclaimed,
            indeterminate,
        }
    }
}
