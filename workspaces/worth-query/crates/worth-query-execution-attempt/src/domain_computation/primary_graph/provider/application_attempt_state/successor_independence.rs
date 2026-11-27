//! A successor round re-derives from its chain's own-output view; any other
//! read of a key an earlier round wrote is that round's effect, not C (§1.5).
use super::*;
use crate::domain_computation::primary_graph::application_attempt::fact::touch_key;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact as ObservedFact;
use crate::domain_computation::primary_graph::invariant_projection::read_basis::OwnOutputOverlay;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationDecisionFact as Fact;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenial as Denial, WorthQueryApplicationAttemptDenialKind as Kind,
};
use worth_query_execution_runtime::facade::ports::ProductBranchLeaseInternalPort as _;
use worth_relational::facade::mvcc::ValidatedMutationTouch as Touch;

impl super::super::WorthQueryPrimaryGraphProvider {
    /// The local create coordinate of `session`'s original idempotency record.
    pub fn coupled_writer_original_key(
        &self,
        session: WorthQueryProviderSessionView<'_>,
    ) -> String {
        let attempts = self
            .attempts
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let attempt = attempts
            .attempt_state(session)
            .expect("actual coupled writer")
            .attempt();
        super::super::idempotency::original_client_key(attempt.idempotency())
    }

    /// The first read of `session`'s writer, outside `own`, that a touch its
    /// chain's earlier rounds certified could change and that does not read
    /// the same at the chain head's pre-pin as at `current`, with that read's
    /// record when it has one. `None` when the round is independent.
    pub fn coupled_writer_prior_effect(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        session: WorthQueryProviderSessionView<'_>,
        before: &crate::basis::WorthQueryProductBranchLease,
        current: &worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
        own: &OwnOutputOverlay,
        prior: &[Touch],
    ) -> Result<Option<Option<worth_relational::facade::identity::EntityId>>, Denial> {
        self.graph.with_runtime(execution, |runtime| {
            let snapshot =
                crate::domain_computation::primary_graph::exact_basis_access::open_exact_basis_snapshot(
                    runtime,
                    before.relational_basis(),
                )
                .map_err(|_| Denial::new(Kind::StorageUnavailable, "successor independence C basis"))?;
            let answer = (|| {
                let attempts = self
                    .attempts
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                let attempt = attempts
                    .attempt_state(session)
                    .expect("actual successor writer before Native append")
                    .attempt();
                for fact in attempt.facts().values() {
                    let fact = match fact {
                        Fact::Application { fact, .. }
                        | Fact::ObservedSource { fact }
                        | Fact::ApplicationObservedSource { fact, .. } => fact,
                        Fact::Principal(_) | Fact::Authorization { .. } => continue,
                    };
                    // Each read scans every touch the chain's earlier rounds made.
                    charge(execution, prior.len())?;
                    let observed = std::ops::Deref::deref(&fact);
                    let record = touch_key::fact_record(observed);
                    let covered = record.is_some_and(|record| own.covers(record));
                    // The own-output view served this read at the head pre-pin.
                    if covered && touch_key::overlay_served(observed) {
                        continue;
                    }
                    // Native's exclusive set holds every entity the chain wrote
                    // (`IntervenedOutput` otherwise), so an entity-local touch
                    // only ever changed a covered record (a delete's cascade
                    // writes relation records only): a read of another
                    // single record, or a selection the overlay merged, cannot
                    // see it. A read of a covered record the view did not serve
                    // is judged by the touch and the head/current comparison.
                    let overlaid = (record.is_some() && !covered)
                        || matches!(observed, ObservedFact::IndexedEntitySelection { .. });
                    // A served relation list (#277) already excludes the
                    // chain's own links and unlinks, and the relations a
                    // chain delete retires are recorded too. A delete is
                    // judged on what Native did not record (below). Every
                    // other touch still gates by the raw comparison
                    // (relation field initialisation and unrepresentable
                    // mutation stay conservative).
                    let relation_served = touch_key::relation_served(observed);
                    let touched = prior.iter().any(|touch| {
                        !(overlaid && entity_local(touch))
                            && !(relation_served && (relation_link(touch) || deletes(touch)))
                            && touch_key::touch_changes(touch, observed)
                    });
                    // A read that is the same before the chain as now cannot
                    // have observed the chain's effect: the whole read when a
                    // touch could change it, or the uncovered part of a served
                    // relation list when the chain only deleted.
                    let compared = if touched {
                        fact.support_equal_after_own_effect_at(
                            execution, runtime, crate::domain_computation::primary_graph::output_lineage::CertificationBasis::of(
                                before.lifecycle_incarnation(),
                                before.reference_generation(),
                                &snapshot,
                            ), own.head(), current,
                        )
                    } else if relation_served && prior.iter().any(deletes) {
                        own.uncovered_residue_equal(runtime, execution, &snapshot, current, observed)
                    } else {
                        continue;
                    };
                    let equal = match compared {
                        Ok(equal) => equal,
                        // Unavailable at this image: independence is not
                        // proved, so the round stops typed (`PriorEffectRead`).
                        Err(cause) if !crate::domain_computation::primary_graph::provider::application_attempt_state::candidate_read_support::is_hard_comparison_stop(&cause) => false,
                        Err(cause) => return Err(cause),
                    };
                    if !equal {
                        return Ok(Some(record));
                    }
                }
                Ok(None)
            })();
            crate::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
            answer
        })
    }
}

/// Charge one read's scan of `touches` earlier-round touches.
fn charge(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    touches: usize,
) -> Result<(), Denial> {
    let subject = "successor independence";
    let work = u64::try_from(touches)
        .ok()
        .and_then(|work| work.checked_add(1))
        .ok_or_else(|| Denial::new(Kind::CandidateCapacityExceeded, subject))?;
    execution
        .run_native_with_checkpoint(|context| {
            crate::domain_computation::primary_graph::admitted_capture::checkpoint(context, work)
        })
        .map_err(|cause| Denial::relational_read_stopped(cause.into(), subject))?
        .0
        .map_err(|cause| {
            Denial::relational_read_stopped(
                worth_execution::WorkCeilingDenial::Stopped(cause).into(),
                subject,
            )
        })
}

/// A touch that links or unlinks a relation: the served relation list
/// already excludes the chain's own.
fn relation_link(touch: &Touch) -> bool {
    matches!(
        touch,
        Touch::LinkRelation { .. } | Touch::UnlinkRelation { .. }
    )
}

/// A touch that deletes an entity, and with it that entity's relations.
fn deletes(touch: &Touch) -> bool {
    matches!(touch, Touch::DeleteEntity { .. })
}

/// A touch that writes only the entity it creates, changes or deletes.
fn entity_local(touch: &Touch) -> bool {
    matches!(
        touch,
        Touch::CreateEntity { .. } | Touch::WriteEntityField { .. } | Touch::DeleteEntity { .. }
    )
}
