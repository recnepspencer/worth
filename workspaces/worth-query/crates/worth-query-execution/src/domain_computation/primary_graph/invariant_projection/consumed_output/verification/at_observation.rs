//! A settlement verified for a reader at a retained observation.
//!
//! The marks a reader uses are the ones recorded at its own position: marks
//! from later commits do not apply to it. A reader older than the positions
//! the mark image retains has none to read. It compares every source fact
//! and the output witness of the settlement, and of everything that
//! settlement consumed, at its own snapshot.

use super::*;

/// Whether a reader at `selected` reads at or after the position `observed`
/// was read at, on the same branch of the same runtime.
pub(super) fn reads_at_or_after(
    selected: &PositionedRelationalSnapshot,
    observed: &PositionedRelationalSnapshot,
) -> bool {
    selected.runtime_instance_id() == observed.runtime_instance_id()
        && selected.branch_id() == observed.branch_id()
        && selected.version_id() >= observed.version_id()
        && selected.position() >= observed.position()
}

impl ConsumedOutputEvidence {
    /// Compare one settlement's source facts and its output witness at
    /// `snapshot`. `None` when all of them are unchanged there.
    pub(super) fn compare_own(
        evidence: EvidenceView<'_>,
        direct: bool,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<ConsumedOutputVerification>, ConsumedOutputVerificationStop> {
        Self::compare_facts(
            evidence.source_facts,
            None,
            evidence.native_output_witness.map(Arc::as_ref),
            direct,
            runtime,
            snapshot,
            admission,
        )
    }

    pub(super) fn compare_facts(
        source_facts: &crate::domain_computation::primary_graph::output_lineage::ComparableSourceFacts,
        output_facts: Option<&[WorthQueryApplicationObservedFact]>,
        witness: Option<&OnceLock<SealedNativeOutputWitness>>,
        direct: bool,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<ConsumedOutputVerification>, ConsumedOutputVerificationStop> {
        for (ordinal, fact) in source_facts.iter().enumerate() {
            if !fact_is_current(fact, runtime, snapshot, admission)? {
                return Ok(Some(if direct {
                    ConsumedOutputVerification::ChangedDirectFact(ordinal)
                } else {
                    ConsumedOutputVerification::ChangedUpstream
                }));
            }
        }
        if let Some(output_facts) = output_facts {
            for fact in output_facts {
                if !fact_is_current(fact, runtime, snapshot, admission)? {
                    return Ok(Some(ConsumedOutputVerification::ChangedUpstream));
                }
            }
        } else if let Some(witness) = witness {
            let witness = witness
                .get()
                .ok_or(ConsumedOutputVerificationStop::Unavailable)?;
            if !witness
                .unchanged_in(runtime, snapshot, admission)
                .map_err(map_admission_stop)?
            {
                return Ok(Some(ConsumedOutputVerification::ChangedUpstream));
            }
        }
        Ok(None)
    }

    /// Verify the settlement a reader at a retained observation selected, at
    /// that reader's own position. The lineage row it selected may be older
    /// than the row recorded at its observation, which retention has since
    /// released; the reader never takes it on its position alone. Both the
    /// marked and the full comparison are paid from the reader's meter.
    pub(in crate::domain_computation::primary_graph) fn verify_at_observation(
        identity: &Arc<RecordedSettlementIdentity>,
        source_facts: &crate::domain_computation::primary_graph::output_lineage::ComparableSourceFacts,
        upstream: &[ConsumedOutputEvidence],
        verification_requirement: Option<FullVerificationReason>,
        native_output_witness: &Arc<OnceLock<SealedNativeOutputWitness>>,
        owner: &SourceInvalidationOwner,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        selected: &PositionedRelationalSnapshot,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<ConsumedOutputVerification, ConsumedOutputVerificationStop> {
        let marked = Self::verify_candidate_at(
            identity,
            source_facts,
            upstream,
            verification_requirement,
            native_output_witness,
            selected,
            owner,
            runtime,
            snapshot,
            selected,
            admission,
        );
        if marked != Err(ConsumedOutputVerificationStop::Unavailable) {
            return marked;
        }
        // No marks are retained for this position.
        Self::compare_in_full(
            EvidenceView {
                identity,
                source_facts,
                upstream,
                verification_requirement,
                native_output_witness: Some(native_output_witness),
                selected_native_root: selected,
            },
            runtime,
            snapshot,
            selected,
            admission,
        )
    }

    pub(super) fn compare_in_full<'a>(
        root: EvidenceView<'a>,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        selected: &PositionedRelationalSnapshot,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<ConsumedOutputVerification, ConsumedOutputVerificationStop> {
        let mut pending = Vec::new();
        let mut visited = OrdSet::new();
        charge_external(admission, 1)?;
        reserve_pending(&mut pending, 1, admission)?;
        pending.push((root, true));
        while let Some((evidence, direct)) = pending.pop() {
            admission
                .admit_visited_settlement(visited.len())
                .map_err(map_admission_stop)?;
            if visited.contains(evidence.identity) {
                continue;
            }
            visited.insert(Arc::clone(evidence.identity));
            if !reads_at_or_after(selected, evidence.selected_native_root) {
                return Ok(ConsumedOutputVerification::ChangedUpstream);
            }
            let changed = Self::compare_own(evidence, direct, runtime, snapshot, admission)?;
            if let Some(changed) = changed {
                return Ok(changed);
            }
            charge_external(admission, evidence.upstream.len())?;
            reserve_pending(&mut pending, evidence.upstream.len(), admission)?;
            pending.extend(
                evidence
                    .upstream
                    .iter()
                    .map(|upstream| (EvidenceView::from(upstream), false)),
            );
        }
        Ok(ConsumedOutputVerification::Current)
    }
}
