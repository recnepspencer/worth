use std::sync::{Arc, OnceLock};

use im::OrdSet;
use worth_relational::facade::mvcc::{CompanionCellEditStop, CompanionPreflightStop};
use worth_relational::facade::runtime::{PositionedRelationalSnapshot, RelationalRuntime};
use worth_relational::facade::snapshots::SnapshotHandle;

use super::ConsumedOutputEvidence;

mod at_observation;
mod replaced;
mod restored_root;
mod work_budget;
use crate::domain_computation::primary_graph::{
    application_attempt::{
        Movement, WorthQueryApplicationObservedFact, WorthQuerySourceCurrentnessFailure,
    },
    output_lineage::{
        invalidation::{
            ConsumedOutputCurrentness, DirtyReverification, FullVerificationReason,
            InvalidationEditAdmission, SettlementVerificationStop, SourceSettlementCurrentness,
        },
        RecordedSettlementIdentity, SealedNativeOutputWitness,
    },
    SourceInvalidationOwner,
};
pub(super) use work_budget::map_admission_stop;
use work_budget::{charge_external, fact_is_current, map_verification_stop, reserve_pending};

#[derive(Clone, Copy)]
pub(in crate::domain_computation::primary_graph) struct EvidenceView<'a> {
    pub(super) identity: &'a Arc<RecordedSettlementIdentity>,
    pub(super) source_facts: &'a [WorthQueryApplicationObservedFact],
    pub(super) upstream: &'a [ConsumedOutputEvidence],
    pub(super) verification_requirement: Option<FullVerificationReason>,
    pub(super) native_output_witness: Option<&'a Arc<OnceLock<SealedNativeOutputWitness>>>,
    pub(super) selected_native_root: &'a PositionedRelationalSnapshot,
}

#[cfg(feature = "certification-invalidation-equivalence")]
impl<'a> EvidenceView<'a> {
    pub(in crate::domain_computation::primary_graph) fn identity(
        self,
    ) -> &'a Arc<RecordedSettlementIdentity> {
        self.identity
    }
    pub(in crate::domain_computation::primary_graph) fn source_facts(
        self,
    ) -> &'a [WorthQueryApplicationObservedFact] {
        self.source_facts
    }
    pub(in crate::domain_computation::primary_graph) fn upstream(
        self,
    ) -> &'a [ConsumedOutputEvidence] {
        self.upstream
    }
    pub(in crate::domain_computation::primary_graph) fn selected_native_root(
        self,
    ) -> &'a PositionedRelationalSnapshot {
        self.selected_native_root
    }
    pub(in crate::domain_computation::primary_graph) fn native_output_witness(
        self,
    ) -> Option<&'a SealedNativeOutputWitness> {
        self.native_output_witness.and_then(|witness| witness.get())
    }
}

impl<'a> From<&'a ConsumedOutputEvidence> for EvidenceView<'a> {
    fn from(evidence: &'a ConsumedOutputEvidence) -> Self {
        Self {
            identity: &evidence.identity,
            source_facts: &evidence.source_facts,
            upstream: &evidence.upstream,
            verification_requirement: evidence.verification_requirement,
            native_output_witness: evidence.native_output_witness.as_ref(),
            selected_native_root: &evidence.selected_native_root,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum ConsumedOutputVerification {
    Current,
    ChangedDirectFact(usize),
    ChangedUpstream,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum ConsumedOutputVerificationStop {
    Unavailable,
    PendingUpstream,
    RetryCurrentness(CompanionCellEditStop),
    WorkExhausted,
}

impl ConsumedOutputEvidence {
    /// The source owner decides whether this settlement is clean, which exact
    /// fact ordinals are dirty, or whether retained evidence must be verified.
    /// Every actor lookup and fact comparison is paid from the caller's meter.
    /// The iterative worklist visits each exact settlement at most once.
    pub(in crate::domain_computation::primary_graph::invariant_projection) fn verify_candidate_at(
        identity: &Arc<RecordedSettlementIdentity>,
        source_facts: &[WorthQueryApplicationObservedFact],
        upstream: &[ConsumedOutputEvidence],
        verification_requirement: Option<FullVerificationReason>,
        native_output_witness: &Arc<OnceLock<SealedNativeOutputWitness>>,
        observed: &PositionedRelationalSnapshot,
        owner: &SourceInvalidationOwner,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        selected: &PositionedRelationalSnapshot,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<ConsumedOutputVerification, ConsumedOutputVerificationStop> {
        Self::verify_views(
            std::iter::once(EvidenceView {
                identity,
                source_facts,
                upstream,
                verification_requirement,
                native_output_witness: Some(native_output_witness),
                selected_native_root: observed,
            }),
            owner,
            runtime,
            snapshot,
            selected,
            admission,
        )
    }

    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn verify_candidate_with_admission(
        identity: &Arc<RecordedSettlementIdentity>,
        source_facts: &[WorthQueryApplicationObservedFact],
        upstream: &[ConsumedOutputEvidence],
        verification_requirement: Option<FullVerificationReason>,
        observed: &PositionedRelationalSnapshot,
        owner: &SourceInvalidationOwner,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        selected: &PositionedRelationalSnapshot,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<ConsumedOutputVerification, ConsumedOutputVerificationStop> {
        Self::verify_views(
            std::iter::once(EvidenceView {
                identity,
                source_facts,
                upstream,
                verification_requirement,
                native_output_witness: None,
                selected_native_root: observed,
            }),
            owner,
            runtime,
            snapshot,
            selected,
            admission,
        )
    }

    pub(in crate::domain_computation::primary_graph) fn verify_many_with_admission(
        roots: &[Self],
        owner: &SourceInvalidationOwner,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        selected: &PositionedRelationalSnapshot,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<ConsumedOutputVerification, ConsumedOutputVerificationStop> {
        Self::verify_views(
            roots.iter().map(EvidenceView::from),
            owner,
            runtime,
            snapshot,
            selected,
            admission,
        )
    }

    fn verify_views<'a>(
        roots: impl ExactSizeIterator<Item = EvidenceView<'a>> + Clone,
        owner: &SourceInvalidationOwner,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        selected: &PositionedRelationalSnapshot,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<ConsumedOutputVerification, ConsumedOutputVerificationStop> {
        #[cfg(feature = "certification-invalidation-equivalence")]
        if roots.len() == 0 {
            return Ok(ConsumedOutputVerification::Current);
        }
        #[cfg(feature = "certification-invalidation-equivalence")]
        let image = owner.capture_full_verification(runtime, snapshot, selected, admission);
        let mut verified = Vec::new();
        let mut marked = Vec::new();
        let result = Self::verify_marked_views(
            roots.clone(),
            owner,
            runtime,
            snapshot,
            selected,
            &mut verified,
            &mut marked,
            admission,
        );
        #[cfg(feature = "certification-invalidation-equivalence")]
        let result = super::equivalence::check(roots, image, result, admission);
        if result == Ok(ConsumedOutputVerification::Current) {
            // Each of these rows had its facts and output compared in full.
            // Consumed rows come last, so they are re-established first and
            // their consumers find them clean. A row that cannot be
            // re-established keeps requiring verification. The caller's meter
            // pays for recording it too; the answer is already decided, so a
            // meter that runs out here leaves the row unrecorded.
            for (evidence, _) in verified.into_iter().rev() {
                let _ = owner.reestablish_verified(
                    runtime,
                    snapshot,
                    selected,
                    evidence.identity,
                    evidence.source_facts,
                    admission,
                );
            }
            // Rows answered from their marks are verified through this image.
            let _ = owner.carry_read_basis(runtime, snapshot, selected, &marked, admission);
        }
        result
    }

    fn verify_marked_views<'a>(
        roots: impl ExactSizeIterator<Item = EvidenceView<'a>>,
        owner: &SourceInvalidationOwner,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        selected: &PositionedRelationalSnapshot,
        verified: &mut Vec<(EvidenceView<'a>, bool)>,
        marked: &mut Vec<Arc<RecordedSettlementIdentity>>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<ConsumedOutputVerification, ConsumedOutputVerificationStop> {
        let mut pending = Vec::new();
        let mut visited = OrdSet::new();
        charge_external(admission, roots.len())?;
        reserve_pending(&mut pending, roots.len(), admission)?;
        pending.extend(roots.map(|root| (root, true)));
        while let Some((evidence, direct)) = pending.pop() {
            admission
                .admit_visited_settlement(visited.len())
                .map_err(map_admission_stop)?;
            if visited.contains(evidence.identity) {
                continue;
            }
            visited.insert(Arc::clone(evidence.identity));
            if !at_observation::reads_at_or_after(selected, evidence.selected_native_root) {
                return Ok(ConsumedOutputVerification::ChangedUpstream);
            }
            let currentness = owner
                .consumed_output_currentness(selected, evidence.identity, admission)
                .map_err(map_admission_stop)?;
            let currentness = match currentness {
                ConsumedOutputCurrentness::Direct(currentness) => currentness,
                ConsumedOutputCurrentness::CanonicallyEqualClean(successor) => {
                    // This exact actor consequence proves the old consumed
                    // output equal to its clean certified successor. The old
                    // receipt and old source marks remain unchanged.
                    admission
                        .charge_external_work(1)
                        .map_err(map_admission_stop)?;
                    reserve_pending(marked, 2, admission)?;
                    marked.extend([Arc::clone(evidence.identity), successor]);
                    continue;
                }
                ConsumedOutputCurrentness::PendingEqualSuccessor => {
                    return Err(ConsumedOutputVerificationStop::PendingUpstream);
                }
                ConsumedOutputCurrentness::FullVerificationRequired => {
                    return Err(ConsumedOutputVerificationStop::Unavailable);
                }
            };
            match currentness {
                SourceSettlementCurrentness::Clean
                    if evidence.verification_requirement.is_none() =>
                {
                    reserve_pending(marked, 1, admission)?;
                    marked.push(Arc::clone(evidence.identity));
                }
                SourceSettlementCurrentness::PendingUpstream(_) => {
                    return Err(ConsumedOutputVerificationStop::PendingUpstream);
                }
                // As for a foreign selection above.
                SourceSettlementCurrentness::Foreign => {
                    return Err(ConsumedOutputVerificationStop::Unavailable);
                }
                SourceSettlementCurrentness::Dirty(_)
                    if evidence.verification_requirement.is_none() =>
                {
                    let reverified = owner.reverify_dirty(
                        runtime,
                        snapshot,
                        selected,
                        evidence.identity,
                        admission,
                    );
                    match reverified.map_err(map_verification_stop)? {
                        DirtyReverification::ChangedOrdinal(ordinal) => {
                            return Ok(if direct {
                                ConsumedOutputVerification::ChangedDirectFact(ordinal)
                            } else {
                                ConsumedOutputVerification::ChangedUpstream
                            });
                        }
                        DirtyReverification::Verified(token) => {
                            let cleared = owner.clear_verified_dirty(token, admission);
                            cleared.map_err(map_verification_stop)?;
                            reserve_pending(marked, 1, admission)?;
                            marked.push(Arc::clone(evidence.identity));
                        }
                        DirtyReverification::HistoricalCurrent
                        | DirtyReverification::AlreadyCurrent => {}
                    }
                }
                unmarked @ (SourceSettlementCurrentness::Clean
                | SourceSettlementCurrentness::Dirty(_)
                | SourceSettlementCurrentness::FullVerificationRequired(_)) => {
                    let changed =
                        Self::compare_own(evidence, direct, runtime, snapshot, admission)?;
                    if let Some(changed) = changed {
                        return Ok(changed);
                    }
                    if evidence.native_output_witness.is_some()
                        && matches!(
                            unmarked,
                            SourceSettlementCurrentness::FullVerificationRequired(_)
                        )
                    {
                        reserve_pending(verified, 1, admission)?;
                        verified.push((evidence, direct));
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
            }
        }
        Ok(ConsumedOutputVerification::Current)
    }
}
