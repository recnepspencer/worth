//! Full comparison after a certified chain's terminal leaves the mark window.

use super::*;
use crate::domain_computation::primary_graph::output_lineage::invalidation::EqualityRecoveryRow;

enum RecoveryStep {
    Visit(Arc<RecordedSettlementIdentity>, bool),
    Compared(EqualityRecoveryRow),
}

impl ConsumedOutputEvidence {
    pub(super) fn compare_equal_chain(
        evidence: EvidenceView<'_>,
        owner: &SourceInvalidationOwner,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        selected: &PositionedRelationalSnapshot,
        compared: &mut Vec<EqualityRecoveryRow>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<ConsumedOutputVerification, ConsumedOutputVerificationStop> {
        let image = owner
            .equality_recovery_image(selected, admission)
            .map_err(map_verification_stop)?;
        let mut pending = Vec::new();
        let mut visited = OrdSet::new();
        reserve_pending(&mut pending, 1, admission)?;
        pending.push(RecoveryStep::Visit(Arc::clone(evidence.identity), true));
        while let Some(step) = pending.pop() {
            let (identity, root) = match step {
                RecoveryStep::Visit(identity, root) => (identity, root),
                RecoveryStep::Compared(row) => {
                    reserve_pending(compared, 1, admission)?;
                    compared.push(row);
                    continue;
                }
            };
            let row = image
                .terminal_row(&identity, admission)
                .map_err(map_verification_stop)?;
            admission
                .admit_visited_settlement(visited.len())
                .map_err(map_admission_stop)?;
            if visited.contains(&row.identity) {
                continue;
            }
            visited.insert(Arc::clone(&row.identity));
            if !at_observation::reads_at_or_after(selected, &row.read_basis) {
                return Ok(ConsumedOutputVerification::ChangedUpstream);
            }
            let Some(facts) = row.facts.for_comparison() else {
                return Ok(ConsumedOutputVerification::ChangedUpstream);
            };
            let witness = evidence
                .native_output_witness
                .filter(|_| root)
                .map(Arc::as_ref);
            if row.output_facts().is_none() && witness.is_none() {
                return Err(ConsumedOutputVerificationStop::Unavailable);
            }
            let changed = Self::compare_facts(
                &facts,
                row.output_facts(),
                witness,
                false,
                runtime,
                snapshot,
                admission,
            )?;
            if let Some(changed) = changed {
                image.fence(admission).map_err(map_verification_stop)?;
                return Ok(changed);
            }
            charge_external(admission, row.upstream.len())?;
            reserve_pending(
                &mut pending,
                row.upstream.len().saturating_add(1),
                admission,
            )?;
            let upstream = row.upstream.clone();
            pending.push(RecoveryStep::Compared(row));
            pending.extend(
                upstream
                    .iter()
                    .map(|upstream| RecoveryStep::Visit(Arc::clone(upstream), false)),
            );
        }
        image.fence(admission).map_err(map_verification_stop)?;
        Ok(ConsumedOutputVerification::Current)
    }
}
