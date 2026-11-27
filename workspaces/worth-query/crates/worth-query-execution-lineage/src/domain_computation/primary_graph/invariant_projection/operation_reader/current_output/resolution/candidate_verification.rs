//! One selected candidate verified at the reader's own snapshot.

use std::sync::Arc;

use worth_relational::facade::runtime::PositionedRelationalSnapshot;

use super::*;
use crate::domain_computation::primary_graph::output_lineage::family_selection::{candidate_verdict, CandidateVerdict};
use crate::domain_computation::primary_graph::output_lineage::WorthQueryCurrentOutputCandidate;

impl<'reader, 'runtime, Schema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'reader, 'runtime, Schema, Operation>
where
    Schema: ApplicationSchema,
{
    /// The candidate's verdict at the selected snapshot, through the one
    /// [`candidate_verdict`] a family selection's re-resolution also uses.
    ///
    /// A restored output no demand has verified carries no witness yet. Its
    /// reader compares the checkpoint facts and the output in full; the read
    /// retains the witness that comparison built (`retain_restored_witness`).
    pub(crate) fn verify_current_candidate(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        candidate: &WorthQueryCurrentOutputCandidate,
        producer: worth_relational::facade::identity::EntityId,
        native: &worth_relational::facade::runtime::RelationalRuntime,
        selected: &PositionedRelationalSnapshot,
        subject: &str,
    ) -> Result<CandidateVerdict, WorthQueryCurrentOutputDenial> {
        enum Stop {
            Denial(WorthQueryCurrentOutputDenial),
            Verification(ConsumedOutputVerificationStop),
        }
        let product = self.selected_product;
        let reader = &self.reader;
        let basis = || {
            product
                .map(|product| {
                    crate::domain_computation::primary_graph::output_lineage::CertificationBasis::of(
                        product.lifecycle_incarnation(),
                        product.reference_generation(),
                        reader.snapshot,
                    )
                })
                .ok_or_else(|| {
                    Stop::Denial(WorthQueryCurrentOutputDenial::new(
                        WorthQueryCurrentOutputDenialKind::FamilyUnavailable,
                        subject,
                    ))
                })
        };
        let verdict = candidate_verdict(
            candidate,
            producer,
            |facts, witness| {
                ConsumedOutputEvidence::verify_at_observation(
                    execution,
                    &candidate.settlement_identity,
                    facts,
                    &candidate.consumed_outputs_at(selected),
                    candidate.verification_requirement,
                    witness,
                    &reader.invalidation_owner,
                    native,
                    basis()?,
                    selected,
                    candidate.historical(),
                )
                .map_err(Stop::Verification)
            },
            |facts| {
                basis()?;
                ConsumedOutputEvidence::verify_restored_root_at(
                    execution,
                    &candidate.correspondence,
                    facts,
                    reader.layout,
                    native,
                    reader.snapshot,
                )
                .map_err(Stop::Verification)
            },
        );
        let stop = match verdict {
            Ok(CandidateVerdict::RestoredCurrent { facts, witness }) => {
                let witness = self
                    .reader
                    .output_lineage
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .retain_restored_witness(&candidate.settlement_identity, witness);
                return Ok(CandidateVerdict::Current { facts, witness });
            }
            Ok(verdict) => return Ok(verdict),
            Err(Stop::Denial(denial)) => return Err(denial),
            Err(Stop::Verification(stop)) => stop,
        };
        if stop == ConsumedOutputVerificationStop::WorkExhausted {
            self.reader.work_budget.mark_exceeded();
        }
        self.reader.retention_exhausted |=
            matches!(stop, ConsumedOutputVerificationStop::CapacityExhausted(_));
        let pending = stop == ConsumedOutputVerificationStop::PendingUpstream;
        let mut denial = super::verification_denial::from_stop(stop, subject);
        if pending {
            denial.requested_output = Some(self.retain_requested_output(candidate, selected)?);
        }
        Err(denial)
    }
    pub(crate) fn retain_requested_output(
        &mut self,
        candidate: &WorthQueryCurrentOutputCandidate,
        selected: &PositionedRelationalSnapshot,
    ) -> Result<
        crate::domain_computation::primary_graph::invariant_projection::RequestedOutputRead,
        WorthQueryCurrentOutputDenial,
    > {
        use crate::domain_computation::primary_graph::invariant_projection::RequestedOutputRead;
        Ok(RequestedOutputRead::new(
            Arc::clone(&candidate.settlement_identity),
            selected.clone(),
        ))
    }
}
