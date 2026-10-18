//! Wake actual native consumers only after completion's image is installed.

use super::super::{
    admission::IndexAdmission, mark_state::MarkState, retention, InvalidationEditAdmission,
    SourceInvalidationOwner,
};
use super::RecordedSettlementIdentity;
use crate::domain_computation::primary_graph::application_output_demand::{
    ReplacedRequiredWorkHint, RequiredWorkMembership,
};
use im::OrdSet;
use std::sync::Arc;
use worth_relational::facade::mvcc::CompanionPreflightStop;

type Cue = (Arc<RequiredWorkMembership>, Arc<RecordedSettlementIdentity>);
type Capacity = Arc<
    crate::domain_computation::execution_runtime::source_invalidation::RetainedInvalidationCapacity,
>;

pub(super) struct PreparedCompletionCues {
    entries: Vec<Cue>,
    cleanup: Vec<ReplacedRequiredWorkHint>,
    capacity: Capacity,
}

pub(super) struct PublishedCompletionCues {
    _cleanup: Vec<ReplacedRequiredWorkHint>,
    _capacity: Capacity,
}

impl PreparedCompletionCues {
    pub(super) fn prepare(
        owner: &SourceInvalidationOwner,
        state: &MarkState,
        affected: &OrdSet<Arc<RecordedSettlementIdentity>>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Self, CompanionPreflightStop> {
        let bytes = affected
            .len()
            .checked_mul(
                std::mem::size_of::<Cue>() + std::mem::size_of::<ReplacedRequiredWorkHint>(),
            )
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        admission.bytes(bytes)?;
        let capacity = retention::reserve(&owner.resources, bytes, admission)?;
        let mut entries = Vec::with_capacity(affected.len());
        let cleanup = Vec::with_capacity(affected.len());
        for identity in affected {
            admission.work(1)?;
            admission.ordered_read(state.settlements.len())?;
            if let Some(membership) = state
                .settlements
                .get(identity)
                .and_then(|row| row.work_membership.as_ref())
            {
                entries.push((Arc::clone(membership), Arc::clone(identity)));
            }
        }
        Ok(Self {
            entries,
            cleanup,
            capacity,
        })
    }

    pub(super) fn publish(mut self) -> PublishedCompletionCues {
        // Both vectors were admitted before native installation. Native gate
        // failure drops them without sending any queue hint.
        for (membership, identity) in self.entries {
            self.cleanup.push(membership.mark_local_required(identity));
        }
        PublishedCompletionCues {
            _cleanup: self.cleanup,
            _capacity: self.capacity,
        }
    }
}
