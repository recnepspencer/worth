//! A genuine applied Source loan may join only its own actual publication.
use super::*;
use crate::{runtime::RelationalRuntime, snapshots::data::SnapshotHandle};

pub struct ManagedIndexAppliedSourcePublication {
    affinity: Arc<AppliedSourceAffinity>,
}
impl ManagedIndexAppliedSourcePublication {
    pub(super) fn from_loan(loan: &ManagedIndexAppliedSourceRead<'_>) -> Self {
        Self {
            affinity: Arc::clone(&loan.affinity),
        }
    }
    /// Read the installed physical Source at the genuine final snapshot. No
    /// applied pin, proposal id or caller-supplied position can supply it.
    pub fn observe_published(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        after: &SnapshotHandle,
    ) -> Result<crate::mvcc::ManagedIndexSourceObservation, Denial> {
        if runtime.runtime_instance_id() != self.affinity.candidate.proposal().runtime_instance_id()
        {
            return Err(Denial::ForeignSource);
        }
        // A receipt retains issued affinity, not its proposal's live interpreter.
        // Registered Source facts can themselves be retained by this lifecycle.
        // Read its actual final runtime registration without closing that cycle.
        let generation = {
            let publication = runtime.publication_binding();
            let epoch = publication
                .companion_epoch()
                .map_err(Denial::Registration)?;
            epoch
                .active()
                .map_err(Denial::Registration)?
                .map(|(generation, _)| generation)
                .ok_or(Denial::MissingParticipant)?
        };
        if generation != self.affinity.generation {
            return Err(Denial::Publication(
                crate::mvcc::CompanionPreflightStop::RegistrationChanged,
            ));
        }
        let observed = runtime.read_truth().observe_managed_index_source(
            execution,
            after,
            &self.affinity.program,
            self.affinity.scope,
        )?;
        if !observed.matches_applied_publication(self.affinity.candidate.proposal()) {
            return Err(Denial::ForeignSource);
        }
        Ok(observed)
    }
}
