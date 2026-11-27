//! Full comparison after a certified chain's terminal leaves the mark window.

use super::*;
use crate::domain_computation::primary_graph::output_lineage::invalidation::EqualityRecoveryImage;
use crate::domain_computation::primary_graph::output_lineage::invalidation::EqualityRecoveryRow;
use crate::domain_computation::primary_graph::output_lineage::invalidation::ObservedRows;
use crate::domain_computation::primary_graph::output_lineage::CertificationScope;

enum RecoveryStep {
    Visit(Arc<RecordedSettlementIdentity>, bool),
    Compared(EqualityRecoveryRow),
}

impl ConsumedOutputEvidence {
    pub fn compare_equal_chain(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        evidence: EvidenceView<'_>,
        owner: &SourceInvalidationOwner,
        runtime: &RelationalRuntime,
        basis: CertificationBasis<'_>,
        selected: &PositionedRelationalSnapshot,
        compared: &mut Vec<EqualityRecoveryRow>,
    ) -> Result<ConsumedOutputVerification, ConsumedOutputVerificationStop> {
        let image = owner
            .equality_recovery_image(selected)
            .map_err(map_verification_stop)?;
        let mut observed = ObservedRows::new();
        Self::compare_equal_chain_in(
            execution, evidence, &image, &mut observed, runtime, basis, selected, compared, false,
        )
    }

    /// Follow `evidence`'s equal chain in one aligned `image` with no marks
    /// read. A compared decision is fenced on `observed`, every row this walk
    /// (and any earlier walk sharing `observed`) read. An unlinked root keeps
    /// its direct fact ordinal, as the marked walk compares it directly.
    ///
    /// A `memoized` walk (a re-resolution's, which retains nothing) takes and
    /// records the [`CertificationScope`] closure memo (public #429): a row
    /// another walk in this frame completed `Current` is skipped with its
    /// closure, and that walk's rows join `observed`, so the fence covers it.
    #[allow(clippy::too_many_arguments)]
    pub fn compare_equal_chain_in(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        evidence: EvidenceView<'_>,
        image: &EqualityRecoveryImage,
        observed: &mut ObservedRows,
        runtime: &RelationalRuntime,
        basis: CertificationBasis<'_>,
        selected: &PositionedRelationalSnapshot,
        compared: &mut Vec<EqualityRecoveryRow>,
        memoized: bool,
    ) -> Result<ConsumedOutputVerification, ConsumedOutputVerificationStop> {
        let mut pending = Vec::new();
        let mut visited = OrdSet::new();
        let mut memoize = None;
        reserve_pending(&mut pending, 1)?;
        pending.push(RecoveryStep::Visit(Arc::clone(evidence.identity), true));
        while let Some(step) = pending.pop() {
            let (identity, root) = match step {
                RecoveryStep::Visit(identity, root) => (identity, root),
                RecoveryStep::Compared(row) => {
                    reserve_pending(compared, 1)?;
                    compared.push(row);
                    continue;
                }
            };
            let row = image
                .terminal_row(&identity, observed)
                .map_err(map_verification_stop)?;

            if visited.contains(&row.identity) {
                continue;
            }
            visited.insert(Arc::clone(&row.identity));
            if memoized {
                if let Some(rows) =
                    CertificationScope::memo_hit(basis, selected, image, &row.identity)
                {
                    *observed = std::mem::take(observed).union(rows);
                    continue;
                }
                // A completed walk of a row with an output projection is
                // independent of the root witness and of `direct`.
                if root && row.output_facts().is_some() {
                    memoize = Some(Arc::clone(&row.identity));
                }
            }
            if !at_observation::reads_at_or_after(selected, &row.read_basis) {
                return Ok(ConsumedOutputVerification::Unaligned);
            }
            let Some(facts) = row.facts.for_comparison() else {
                return Ok(ConsumedOutputVerification::ChangedUpstream);
            };
            let direct = root && Arc::ptr_eq(&row.identity, evidence.identity);
            let witness = evidence
                .native_output_witness
                .filter(|_| root)
                .map(Arc::as_ref);
            if row.output_facts().is_none() && witness.is_none() {
                return Err(ConsumedOutputVerificationStop::Unavailable);
            }
            // As `full_verification` takes them (public #358): a projection
            // holds through its own coordinate; the witness, used only at the
            // root without one, through the root evidence's own row.
            let output_historical = match row.output_facts() {
                Some(_) => row.output_historical(),
                None => evidence.historical.filter(|_| root),
            };
            let changed = Self::compare_facts(
                execution,
                &facts,
                row.output_facts(),
                witness,
                direct,
                runtime,
                basis,
                row.historical.as_ref(),
                output_historical,
            )?;
            if let Some(changed) = changed {
                image.fence(observed).map_err(map_verification_stop)?;
                return Ok(changed);
            }

            reserve_pending(&mut pending, row.upstream.len().saturating_add(1))?;
            let upstream = row.upstream.clone();
            pending.push(RecoveryStep::Compared(row));
            pending.extend(
                upstream
                    .iter()
                    .map(|upstream| RecoveryStep::Visit(Arc::clone(upstream), false)),
            );
        }
        image.fence(observed).map_err(map_verification_stop)?;
        if let Some(terminal) = memoize {
            CertificationScope::memo_record(basis, selected, image, terminal, observed);
        }
        Ok(ConsumedOutputVerification::Current)
    }
}
