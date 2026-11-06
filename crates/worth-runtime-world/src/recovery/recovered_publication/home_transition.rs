//! One original recovery row owns every phase of its missing World occurrence.
mod advance;
mod movement;
mod native_occurrence;
mod phase_loan;
mod progress;
mod work;
use crate::history::WorldReadmittedPublication;
use work::*;
pub(crate) use work::{ContinuationCommon, PreparedOriginalContinuation};
use worth_foundational::facade::HomeProgramTransitionStepAssociation;

pub(crate) struct OriginalHomeTransitionContinuation {
    step: HomeProgramTransitionStepAssociation,
    plan: [u8; 32],
    association: [u8; 32],
    progress: Option<OriginalContinuationProgress>,
    _operation: crate::lifecycle::owner::RuntimeWorldOperationReservation,
    _publication_capacity: crate::lifecycle::owner::ReservedPublicationAttemptCapacity,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

pub(crate) enum OriginalContinuationAdvance {
    Completed(WorldReadmittedPublication),
    Pending,
    EvidenceRequired,
    Indeterminate,
    ReopenRequired,
}

impl std::fmt::Debug for OriginalHomeTransitionContinuation {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        out.debug_struct("OriginalHomeTransitionContinuation")
            .field("step", &self.step)
            .finish_non_exhaustive()
    }
}

impl OriginalHomeTransitionContinuation {
    pub(crate) fn reserve_backing(
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        Option<worth_execution::ExecutionMemoryReservation>,
        crate::durability::WorldDurableMediumDenial,
    > {
        let bytes = u64::try_from(std::mem::size_of::<Self>())
            .map_err(|_| crate::durability::WorldDurableFrameDenial::SizeLimit)?;
        request
            .reserve_retained_memory(bytes)
            .map_err(crate::durability::WorldDurableMediumDenial::Request)
    }

    pub(crate) fn admits(
        &self,
        plan: [u8; 32],
        step: HomeProgramTransitionStepAssociation,
        association: [u8; 32],
    ) -> bool {
        self.plan == plan && self.step == step && self.association == association
    }

    pub(crate) fn native_receipt(
        &self,
    ) -> &worth_relational::facade::history::RelationalCommitReceipt {
        match self
            .progress
            .as_ref()
            .expect("the consuming advance retains its phase loan")
        {
            OriginalContinuationProgress::Prepared(work) => {
                work.native.custody().effect().subject().receipt()
            }
            OriginalContinuationProgress::Submitted(work)
            | OriginalContinuationProgress::Acknowledged(work) => {
                work.work.native.custody().effect().subject().receipt()
            }
            OriginalContinuationProgress::Materialized(work) => work.record.semantic().receipt(),
            OriginalContinuationProgress::HistoryInstalled(work) => {
                work.record.semantic().receipt()
            }
            OriginalContinuationProgress::Transferred(work) => work.record.semantic().receipt(),
            OriginalContinuationProgress::Installed(work)
            | OriginalContinuationProgress::ReopenRequired(work) => {
                work.record.semantic().receipt()
            }
            OriginalContinuationProgress::Completed(publication) => publication.native_receipt(),
            OriginalContinuationProgress::Delivered => {
                unreachable!("completed delivery removes the original row")
            }
        }
    }
}
