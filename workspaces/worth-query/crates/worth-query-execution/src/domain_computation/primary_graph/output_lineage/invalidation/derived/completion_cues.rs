//! Wake actual native consumers only after completion's image is installed.

use super::super::mark_state::MarkState;
use super::RecordedSettlementIdentity;
use crate::domain_computation::primary_graph::application_output_demand::ReplacedRequiredWorkHint;
use crate::domain_computation::primary_graph::application_output_demand::RequiredWorkMembership;
use im::OrdSet;
use std::sync::Arc;
use worth_relational::facade::mvcc::CompanionPreflightStop;

type Cue = (Arc<RequiredWorkMembership>, Arc<RecordedSettlementIdentity>);

pub(super) struct PreparedCompletionCues {
    entries: Vec<Cue>,
    cleanup: Vec<ReplacedRequiredWorkHint>,
}

pub(super) struct PublishedCompletionCues {
    _cleanup: Vec<ReplacedRequiredWorkHint>,
}

impl PreparedCompletionCues {
    pub(super) fn prepare(
        state: &MarkState,
        affected: &OrdSet<Arc<RecordedSettlementIdentity>>,
    ) -> Result<Self, CompanionPreflightStop> {
        let mut entries = Vec::with_capacity(affected.len());
        let cleanup = Vec::with_capacity(affected.len());
        for identity in affected {
            if let Some(membership) = state
                .settlements
                .get(identity)
                .and_then(|row| row.work_membership.as_ref())
            {
                entries.push((Arc::clone(membership), Arc::clone(identity)));
            }
        }
        Ok(Self { entries, cleanup })
    }

    pub(super) fn publish(mut self) -> PublishedCompletionCues {
        // Both vectors were admitted before native installation. Native gate
        // failure drops them without sending any queue hint.
        for (membership, identity) in self.entries {
            self.cleanup.push(membership.mark_local_required(identity));
        }
        PublishedCompletionCues {
            _cleanup: self.cleanup,
        }
    }
}
