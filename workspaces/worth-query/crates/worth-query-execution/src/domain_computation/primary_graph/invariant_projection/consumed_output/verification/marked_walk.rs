//! Marked currentness with full comparison when its retained basis expires.

use super::*;

impl ConsumedOutputEvidence {
    pub(super) fn verify_marked_views<'a>(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        roots: impl ExactSizeIterator<Item = EvidenceView<'a>>,
        owner: &SourceInvalidationOwner,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        selected: &PositionedRelationalSnapshot,
        verified: &mut Vec<(EvidenceView<'a>, bool)>,
        marked: &mut Vec<Arc<RecordedSettlementIdentity>>,
        recovered: &mut Vec<crate::domain_computation::primary_graph::output_lineage::invalidation::EqualityRecoveryRow>,
    ) -> Result<ConsumedOutputVerification, ConsumedOutputVerificationStop> {
        let mut pending = Vec::new();
        let mut visited = OrdSet::new();

        reserve_pending(&mut pending, roots.len())?;
        pending.extend(roots.map(|root| (root, true)));
        while let Some((evidence, direct)) = pending.pop() {
            if visited.contains(evidence.identity) {
                continue;
            }
            visited.insert(Arc::clone(evidence.identity));
            if !at_observation::reads_at_or_after(selected, evidence.selected_native_root) {
                return Ok(ConsumedOutputVerification::Unaligned);
            }
            let currentness = owner
                .consumed_output_currentness(selected, evidence.identity)
                .map_err(map_admission_stop)?;
            let currentness = match currentness {
                ConsumedOutputCurrentness::Direct(currentness) => currentness,
                ConsumedOutputCurrentness::CanonicallyEqualClean(successor) => {
                    // This exact actor consequence proves the old consumed
                    // output equal to its clean certified successor. The old
                    // receipt and old source marks remain unchanged.

                    reserve_pending(marked, 2)?;
                    marked.extend([Arc::clone(evidence.identity), successor]);
                    continue;
                }
                ConsumedOutputCurrentness::PendingEqualSuccessor => {
                    if let Some(changed) =
                        Self::compare_own(execution, evidence, direct, runtime, snapshot)?
                    {
                        return Ok(changed);
                    }
                    return Ok(ConsumedOutputVerification::PendingUpstream);
                }
                ConsumedOutputCurrentness::FullVerificationRequired => {
                    let recovered = Self::compare_equal_chain(
                        execution, evidence, owner, runtime, snapshot, selected, recovered,
                    )?;
                    if recovered != ConsumedOutputVerification::Current {
                        return Ok(recovered);
                    }
                    continue;
                }
            };
            match currentness {
                SourceSettlementCurrentness::Clean
                    if evidence.verification_requirement.is_none() =>
                {
                    reserve_pending(marked, 1)?;
                    marked.push(Arc::clone(evidence.identity));
                }
                SourceSettlementCurrentness::PendingUpstream(_) => {
                    if let Some(changed) =
                        Self::compare_own(execution, evidence, direct, runtime, snapshot)?
                    {
                        return Ok(changed);
                    }
                    return Ok(ConsumedOutputVerification::PendingUpstream);
                }
                // As for a foreign selection above.
                SourceSettlementCurrentness::Foreign => {
                    return Err(ConsumedOutputVerificationStop::Unavailable);
                }
                SourceSettlementCurrentness::Dirty(_)
                    if evidence.verification_requirement.is_none() =>
                {
                    let reverified = owner.reverify_dirty(
                        execution,
                        runtime,
                        snapshot,
                        selected,
                        evidence.identity,
                    );
                    match reverified.map_err(map_verification_stop)? {
                        DirtyReverification::ChangedComputation => {
                            return Ok(ConsumedOutputVerification::ChangedUpstream);
                        }
                        DirtyReverification::ChangedOrdinal(ordinal) => {
                            return Ok(if direct {
                                ConsumedOutputVerification::ChangedDirectFact(ordinal)
                            } else {
                                ConsumedOutputVerification::ChangedUpstream
                            });
                        }
                        DirtyReverification::Verified(token) => {
                            let cleared = owner.clear_verified_dirty(token);
                            cleared.map_err(map_verification_stop)?;
                            reserve_pending(marked, 1)?;
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
                        Self::compare_own(execution, evidence, direct, runtime, snapshot)?;
                    if let Some(changed) = changed {
                        return Ok(changed);
                    }
                    if evidence.native_output_witness.is_some()
                        && matches!(
                            unmarked,
                            SourceSettlementCurrentness::FullVerificationRequired(_)
                        )
                    {
                        reserve_pending(verified, 1)?;
                        verified.push((evidence, direct));
                    }

                    reserve_pending(&mut pending, evidence.upstream.len())?;
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
