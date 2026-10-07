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
        if let Some(ordinal) =
            Self::first_changed_own_fact(evidence.source_facts, runtime, snapshot, admission)?
        {
            return Ok(Some(if direct {
                ConsumedOutputVerification::ChangedDirectFact(ordinal)
            } else {
                ConsumedOutputVerification::ChangedUpstream
            }));
        }
        if let Some(witness) = evidence.native_output_witness {
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

    /// A changed accepted consumer may disclose a new dependency set before
    /// following its old pending edges. Consumed native content remains the
    /// child's evidence; only a disjoint changed fact can reject those edges.
    /// This comparison grants no currentness.
    pub(in crate::domain_computation::primary_graph) fn own_evidence_is_current(
        facts: &[WorthQueryApplicationObservedFact],
        consumed: &[ConsumedOutputEvidence],
        witness: &SealedNativeOutputWitness,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, ConsumedOutputVerificationStop> {
        for fact in facts {
            let mut child_content = false;
            for child in consumed {
                charge_external(admission, 1)?;
                let witness = child
                    .native_output_witness
                    .as_ref()
                    .and_then(|cell| cell.get())
                    .ok_or(ConsumedOutputVerificationStop::Unavailable)?;
                if witness
                    .covers_content_fact(fact, admission)
                    .map_err(map_admission_stop)?
                {
                    child_content = true;
                    break;
                }
            }
            if !child_content && !fact_is_current(fact, runtime, snapshot, admission)? {
                return Ok(false);
            }
        }
        witness
            .unchanged_in(runtime, snapshot, admission)
            .map_err(map_admission_stop)
    }

    fn first_changed_own_fact(
        facts: &[WorthQueryApplicationObservedFact],
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<usize>, ConsumedOutputVerificationStop> {
        for (ordinal, fact) in facts.iter().enumerate() {
            if !fact_is_current(fact, runtime, snapshot, admission)? {
                return Ok(Some(ordinal));
            }
        }
        Ok(None)
    }

    /// Verify the settlement a reader at a retained observation selected, at
    /// that reader's own position. The lineage row it selected may be older
    /// than the row recorded at its observation, which retention has since
    /// released; the reader never takes it on its position alone.
    pub(in crate::domain_computation::primary_graph) fn verify_at_observation(
        identity: &Arc<RecordedSettlementIdentity>,
        source_facts: &[WorthQueryApplicationObservedFact],
        upstream: &[ConsumedOutputEvidence],
        verification_requirement: Option<FullVerificationReason>,
        native_output_witness: &Arc<OnceLock<SealedNativeOutputWitness>>,
        owner: &SourceInvalidationOwner,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        selected: &PositionedRelationalSnapshot,
        remaining_work: &mut usize,
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
            remaining_work,
        );
        if marked != Err(ConsumedOutputVerificationStop::Unavailable) {
            return marked;
        }
        // No marks are retained for this position.
        let mut admission = owner.read_admission(*remaining_work);
        let result = Self::compare_in_full(
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
            &mut admission,
        );
        debit_wrapper_work(&admission, remaining_work)?;
        result
    }

    fn compare_in_full<'a>(
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
