//! One selected candidate verified at the reader's own snapshot.

use std::sync::{Arc, OnceLock};

use worth_relational::facade::runtime::PositionedRelationalSnapshot;

use super::*;
use crate::domain_computation::primary_graph::output_lineage::{
    invalidation::FullVerificationReason, SealedNativeOutputWitness,
    WorthQueryCurrentOutputCandidate,
};

pub(super) enum VerifiedCandidate {
    Current {
        facts: crate::domain_computation::primary_graph::output_lineage::ComparableSourceFacts,
        witness: Arc<OnceLock<SealedNativeOutputWitness>>,
    },
    Changed(ConsumedOutputVerification),
}

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
    pub(super) fn verify_current_candidate(
        &mut self,
        candidate: &WorthQueryCurrentOutputCandidate,
        selected: &PositionedRelationalSnapshot,
        subject: &str,
    ) -> Result<Option<VerifiedCandidate>, WorthQueryCurrentOutputDenial> {
        let Some(facts) = candidate.observed_source_facts.for_comparison() else {
            return Ok(Some(VerifiedCandidate::Changed(
                ConsumedOutputVerification::ChangedUpstream,
            )));
        };
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
            Some(witness) => ConsumedOutputEvidence::verify_at_observation(
                &candidate.settlement_identity,
                &facts,
                &candidate.consumed_outputs,
                candidate.verification_requirement,
                witness,
                &self.reader.invalidation_owner,
                self.reader.runtime,
                self.reader.snapshot,
                selected,
                &mut admission,
            )
            .map(|verification| {
                Some(match verification {
                    ConsumedOutputVerification::Current => VerifiedCandidate::Current {
                        facts,
                        witness: Arc::clone(witness),
                    },
                    changed => VerifiedCandidate::Changed(changed),
                })
            }),
            None => ConsumedOutputEvidence::verify_restored_root_at(
                &candidate.correspondence,
                &facts,
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
                Some(VerifiedCandidate::Current { facts, witness })
            }),
        };
        let charged = usize::try_from(admission.charged_work()).unwrap_or(usize::MAX);
        self.reader.work_budget.consume(charged);
        self.reader.work.record_output_lineage_selection(charged);
        verification.map_err(|stop| {
            if stop == ConsumedOutputVerificationStop::WorkExhausted {
                self.reader.work_budget.mark_exceeded();
            }
            self.reader.retention_exhausted |=
                stop == ConsumedOutputVerificationStop::CapacityExhausted;
            let mut denial = super::verification_denial::from_stop(stop, subject);
            if stop == ConsumedOutputVerificationStop::PendingUpstream {
                match self.retain_requested_output(candidate, selected, subject) {
                    Ok(read) => denial.requested_output = Some(read),
                    Err(stopped) => return stopped,
                }
            }
            denial
        })
    }
    pub(super) fn retain_requested_output(
        &mut self,
        candidate: &WorthQueryCurrentOutputCandidate,
        selected: &PositionedRelationalSnapshot,
        subject: &str,
    ) -> Result<
        crate::domain_computation::primary_graph::invariant_projection::RequestedOutputRead,
        WorthQueryCurrentOutputDenial,
    > {
        use crate::domain_computation::primary_graph::invariant_projection::RequestedOutputRead;
        let before = self.reader.work_budget.remaining();
        let maximum = std::num::NonZeroUsize::new(before).ok_or_else(|| {
            self.reader.work_budget.mark_exceeded();
            WorthQueryCurrentOutputDenial::new(
                WorthQueryCurrentOutputDenialKind::WorkBudgetExceeded,
                subject,
            )
        })?;
        let mut admission = self
            .reader
            .invalidation_owner
            .edit_admission_within(maximum);
        let bytes = std::mem::size_of::<RequestedOutputRead>() as u64;
        let result = admission
            .charge_external_work(bytes + selected.branch_id().0.len() as u64)
            .and_then(|_| {
                self.reader.invalidation_owner.retain_consumed_output(
                    &[],
                    selected,
                    bytes,
                    &mut admission,
                )
            });
        let charged = usize::try_from(admission.charged_work()).unwrap_or(usize::MAX);
        self.reader.work_budget.consume(charged);
        self.reader.work.record_output_lineage_selection(charged);
        let capacity = result.map_err(|stop| {
            let stop = crate::domain_computation::primary_graph::invariant_projection::consumed_output::map_admission_stop(stop);
            if stop == ConsumedOutputVerificationStop::WorkExhausted {
                self.reader.work_budget.mark_exceeded();
            }
            self.reader.retention_exhausted |= stop == ConsumedOutputVerificationStop::CapacityExhausted;
            super::verification_denial::from_stop(stop, subject)
        })?;
        Ok(RequestedOutputRead::new(
            Arc::clone(&candidate.settlement_identity),
            selected.clone(),
            capacity,
        ))
    }
}
