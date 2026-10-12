//! Pays and resolves the post-effect source-fact walk, preserving its first request stop.
use super::*;
pub(super) fn rebase(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    prepared: PreparedSourceFactRebase,
    retired: &BTreeSet<EntityId>,
    producer_output: bool,
    maximum_indexed_rebase_work: usize,
    admission: &mut crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission,
) -> RebasedSourceFacts {
    let PreparedSourceFactRebase {
        facts,
        moved_by_own_effect,
        endpoints,
        mut actions,
        mut rebased,
        mut superseded,
    } = prepared;
    let mut endpoints = endpoints.into_iter().peekable();
    // The width the decision declared for its indexed selections, metered
    // across the walk.
    let mut indexed_width = crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission::new(
        worth_relational::facade::mvcc::CompanionPreflightBudget {
            maximum_work_visits: u64::try_from(maximum_indexed_rebase_work).unwrap_or(u64::MAX),
            maximum_preparation_bytes: 0,
        },
    );
    let mut failure = None;
    let mut own_effect = OwnEffectOnReads::NONE_ASKED;
    if !moved_by_own_effect.is_empty() {
        own_effect = own_effect.join(OwnEffect::Moved);
    }
    for (ordinal, fact) in facts.iter().enumerate() {
        // Past a failure the walk decides one thing: what the effect did to
        // each fact a source query read.
        if failure.is_some() && !resolution::reads_source(fact) {
            continue;
        }
        let unasked = &facts[ordinal..];
        let prepared = match resolution::PreparedFactRebase::prepare(
            runtime,
            execution,
            snapshot,
            fact,
            retired,
            producer_output,
            admission,
            &mut indexed_width,
            endpoints
                .next_if(|(prepared_ordinal, _)| *prepared_ordinal == ordinal)
                .map(|(_, payload)| payload),
        ) {
            Ok(prepared) => prepared,
            Err(stop) => return stopped(failure, stop, own_effect, unasked, producer_output),
        };
        match prepared {
            Ok(action) => {
                if matches!(action, resolution::PreparedFactRebase::KeepSuperseded) {
                    own_effect = own_effect.join(OwnEffect::Moved);
                }
                if failure.is_none() {
                    actions.push(action);
                }
            }
            Err(RebaseVerificationReason::Execution(cause)) => {
                return RebasedSourceFacts::VerificationRequired {
                    reason: RebaseVerificationReason::Execution(cause),
                    own_effect: own_effect.with_unanswered(
                        unasked,
                        producer_output,
                        OwnEffect::Undecidable,
                    ),
                }
            }
            Err(reason) => {
                let reason = match reason {
                    RebaseVerificationReason::NativeRevisionUnavailable => {
                        RebaseVerificationReason::NativeFactRevisionUnavailable(ordinal)
                    }
                    RebaseVerificationReason::IndexedSelectionDenied(denial) => {
                        RebaseVerificationReason::IndexedSelectionFactDenied(ordinal, denial)
                    }
                    reason => reason,
                };
                own_effect = own_effect.with_unanswered(
                    std::slice::from_ref(fact),
                    producer_output,
                    OwnEffect::Undecidable,
                );
                failure.get_or_insert(reason);
            }
        }
    }
    if let Some(reason) = failure {
        return RebasedSourceFacts::VerificationRequired { reason, own_effect };
    }
    for (ordinal, (fact, action)) in facts.into_iter().zip(actions).enumerate() {
        if matches!(action, resolution::PreparedFactRebase::KeepSuperseded)
            || moved_by_own_effect.binary_search(&ordinal).is_ok()
        {
            superseded.push(ordinal);
        }
        rebased.push(action.apply(fact));
    }
    if superseded.is_empty() {
        RebasedSourceFacts::Exact(rebased.into())
    } else {
        RebasedSourceFacts::SupersededByOwnEffect {
            facts: rebased.into(),
            ordinals: superseded.into(),
        }
    }
}

/// A meter stop ends the walk and answers nothing about the reads it leaves,
/// so each one the walk would have asked about counts as moved. The commit is
/// superseded and refreshes, and its recompute pays for its own rebase. Only
/// a comparison that cannot answer leaves the effect undecidable.
fn stopped(
    failure: Option<RebaseVerificationReason>,
    stop: CompanionPreflightStop,
    own_effect: OwnEffectOnReads,
    unasked: &[WorthQueryApplicationObservedFact],
    producer_output: bool,
) -> RebasedSourceFacts {
    RebasedSourceFacts::VerificationRequired {
        reason: failure.unwrap_or(RebaseVerificationReason::AdmissionDenied(stop)),
        own_effect: own_effect.with_unanswered(unasked, producer_output, OwnEffect::Moved),
    }
}
