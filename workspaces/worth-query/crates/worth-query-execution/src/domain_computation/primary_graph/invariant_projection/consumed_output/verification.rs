use std::sync::{Arc, OnceLock};

use im::OrdSet;
use worth_relational::facade::mvcc::{CompanionCellEditStop, CompanionPreflightStop};
use worth_relational::facade::runtime::{PositionedRelationalSnapshot, RelationalRuntime};
use worth_relational::facade::snapshots::SnapshotHandle;

use super::ConsumedOutputEvidence;

mod work_budget;
use crate::domain_computation::primary_graph::{
    application_attempt::{WorthQueryApplicationObservedFact, WorthQuerySourceCurrentnessFailure},
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
use work_budget::{charge_external, debit_wrapper_work, map_verification_stop, reserve_pending};

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
    /// Every actor lookup and fact comparison uses the same remaining allowance.
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
        remaining_work: &mut usize,
    ) -> Result<ConsumedOutputVerification, ConsumedOutputVerificationStop> {
        let mut admission = owner.read_admission(*remaining_work);
        let result = Self::verify_views(
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
            &mut admission,
        );
        debit_wrapper_work(&admission, remaining_work)?;
        result
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

    pub(in crate::domain_computation::primary_graph) fn verify_many_at(
        roots: &[Self],
        owner: &SourceInvalidationOwner,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        selected: &PositionedRelationalSnapshot,
        remaining_work: &mut usize,
    ) -> Result<ConsumedOutputVerification, ConsumedOutputVerificationStop> {
        let mut admission = owner.read_admission(*remaining_work);
        let result = Self::verify_many_with_admission(
            roots,
            owner,
            runtime,
            snapshot,
            selected,
            &mut admission,
        );
        debit_wrapper_work(&admission, remaining_work)?;
        result
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
        let result =
            Self::verify_marked_views(roots.clone(), owner, runtime, snapshot, selected, admission);
        #[cfg(feature = "certification-invalidation-equivalence")]
        {
            super::equivalence::check(roots, image, result, admission)
        }
        #[cfg(not(feature = "certification-invalidation-equivalence"))]
        {
            result
        }
    }

    fn verify_marked_views<'a>(
        roots: impl ExactSizeIterator<Item = EvidenceView<'a>>,
        owner: &SourceInvalidationOwner,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        selected: &PositionedRelationalSnapshot,
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
            let observed = &evidence.selected_native_root;
            if selected.runtime_instance_id() != observed.runtime_instance_id()
                || selected.branch_id() != observed.branch_id()
                || selected.version_id() < observed.version_id()
                || selected.position() < observed.position()
            {
                return Ok(ConsumedOutputVerification::ChangedUpstream);
            }
            if evidence.verification_requirement == Some(FullVerificationReason::CheckpointRestore)
            {
                return Err(ConsumedOutputVerificationStop::Unavailable);
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
                    drop(successor);
                    continue;
                }
                ConsumedOutputCurrentness::PendingEqualSuccessor => {
                    return Err(ConsumedOutputVerificationStop::PendingUpstream);
                }
                ConsumedOutputCurrentness::FullVerificationRequired(_) => {
                    return Err(ConsumedOutputVerificationStop::Unavailable);
                }
            };
            match currentness {
                SourceSettlementCurrentness::Clean
                    if evidence.verification_requirement.is_none() => {}
                SourceSettlementCurrentness::PendingUpstream(_) => {
                    return Err(ConsumedOutputVerificationStop::PendingUpstream);
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
                        }
                        DirtyReverification::HistoricalCurrent
                        | DirtyReverification::AlreadyCurrent => {}
                    }
                }
                SourceSettlementCurrentness::FullVerificationRequired(
                    FullVerificationReason::CheckpointRestore,
                ) => return Err(ConsumedOutputVerificationStop::Unavailable),
                SourceSettlementCurrentness::Clean
                | SourceSettlementCurrentness::Dirty(_)
                | SourceSettlementCurrentness::FullVerificationRequired(_) => {
                    for (ordinal, fact) in evidence.source_facts.iter().enumerate() {
                        if !fact_is_current(fact, runtime, snapshot, admission)? {
                            return Ok(if direct {
                                ConsumedOutputVerification::ChangedDirectFact(ordinal)
                            } else {
                                ConsumedOutputVerification::ChangedUpstream
                            });
                        }
                    }
                    if let Some(witness) = evidence.native_output_witness {
                        let witness = witness
                            .get()
                            .ok_or(ConsumedOutputVerificationStop::Unavailable)?;
                        if !witness
                            .unchanged_in(runtime, snapshot, admission)
                            .map_err(map_admission_stop)?
                        {
                            return Ok(ConsumedOutputVerification::ChangedUpstream);
                        }
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

fn fact_is_current(
    fact: &WorthQueryApplicationObservedFact,
    runtime: &RelationalRuntime,
    snapshot: &SnapshotHandle,
    admission: &mut InvalidationEditAdmission,
) -> Result<bool, ConsumedOutputVerificationStop> {
    let available = admission.remaining_work();
    if available == 0 {
        return Err(ConsumedOutputVerificationStop::WorkExhausted);
    }
    let prepaid = fact
        .exact_probe_work()
        .map_err(|_| ConsumedOutputVerificationStop::WorkExhausted)?
        .unwrap_or(0);
    charge_external(admission, prepaid)?;
    let (current, work) = fact
        .source_currentness_in(runtime, snapshot, available)
        .map_err(|failure| match failure {
            WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded => {
                ConsumedOutputVerificationStop::WorkExhausted
            }
            WorthQuerySourceCurrentnessFailure::Unavailable => {
                ConsumedOutputVerificationStop::Unavailable
            }
        })?;
    charge_external(admission, work.saturating_sub(prepaid))?;
    Ok(current)
}
