//! One selected candidate verified at the reader's own snapshot.

use std::sync::{Arc, OnceLock};

use worth_relational::facade::runtime::PositionedRelationalSnapshot;

use super::*;
use crate::domain_computation::primary_graph::output_lineage::{
    invalidation::FullVerificationReason, SealedNativeOutputWitness,
    WorthQueryCurrentOutputCandidate,
};

impl<'reader, 'runtime, Schema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'reader, 'runtime, Schema, Operation>
where
    Schema: ApplicationSchema,
{
    /// The candidate's output witness and its verification at the selected
    /// snapshot. `None` when the candidate has no proof a reader can compare.
    ///
    /// A restored output no demand has verified carries no witness yet. Its
    /// reader compares the checkpoint facts and the output in full, and the
    /// lineage row keeps the witness that comparison built.
    #[allow(clippy::type_complexity)]
    pub(super) fn verify_current_candidate(
        &mut self,
        candidate: &WorthQueryCurrentOutputCandidate,
        selected: &PositionedRelationalSnapshot,
        subject: &str,
    ) -> Result<
        Option<(
            Arc<OnceLock<SealedNativeOutputWitness>>,
            ConsumedOutputVerification,
        )>,
        WorthQueryCurrentOutputDenial,
    > {
        let sealed = candidate
            .native_output_witness
            .as_ref()
            .filter(|witness| witness.get().is_some());
        let restored = candidate.consumed_outputs.is_empty()
            && matches!(
                candidate.verification_requirement,
                Some(FullVerificationReason::CheckpointRestore)
            );
        if sealed.is_none() && !restored {
            return Ok(None);
        }
        // The verification's meter is the reader's remaining work.
        let Some(maximum_work) = std::num::NonZeroUsize::new(self.reader.work_budget.remaining())
        else {
            self.reader.work_budget.mark_exceeded();
            return Err(WorthQueryCurrentOutputDenial::new(
                WorthQueryCurrentOutputDenialKind::WorkBudgetExceeded,
                subject,
            ));
        };
        let mut admission = self
            .reader
            .invalidation_owner
            .edit_admission_within(maximum_work);
        let verification = match sealed {
            Some(witness) => ConsumedOutputEvidence::verify_candidate_at(
                &candidate.settlement_identity,
                &candidate.observed_source_facts,
                &candidate.consumed_outputs,
                candidate.verification_requirement,
                witness,
                selected,
                &self.reader.invalidation_owner,
                self.reader.runtime,
                self.reader.snapshot,
                selected,
                &mut admission,
            )
            .map(|verification| Some((Arc::clone(witness), verification))),
            None => ConsumedOutputEvidence::verify_restored_root_at(
                &candidate.correspondence,
                &candidate.observed_source_facts,
                self.reader.layout,
                &self.reader.invalidation_owner,
                self.reader.runtime,
                self.reader.snapshot,
                &mut admission,
            )
            .map(|witness| {
                let witness = self
                    .reader
                    .output_lineage
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .retain_restored_witness(&candidate.settlement_identity, witness?);
                Some((witness, ConsumedOutputVerification::Current))
            }),
        };
        let charged = usize::try_from(admission.charged_work()).unwrap_or(usize::MAX);
        self.reader.work_budget.consume(charged);
        self.reader.work.record_output_lineage_selection(charged);
        verification.map_err(|stop| {
            if stop == ConsumedOutputVerificationStop::WorkExhausted {
                self.reader.work_budget.mark_exceeded();
            }
            WorthQueryCurrentOutputDenial::new(
                match stop {
                    ConsumedOutputVerificationStop::WorkExhausted => {
                        WorthQueryCurrentOutputDenialKind::WorkBudgetExceeded
                    }
                    ConsumedOutputVerificationStop::PendingUpstream => {
                        WorthQueryCurrentOutputDenialKind::PendingUpstream
                    }
                    ConsumedOutputVerificationStop::RetryCurrentness(stop) => {
                        WorthQueryCurrentOutputDenialKind::CurrentnessRaced(stop)
                    }
                    ConsumedOutputVerificationStop::Unavailable => {
                        WorthQueryCurrentOutputDenialKind::OutputUnavailable
                    }
                },
                subject,
            )
        })
    }
}
