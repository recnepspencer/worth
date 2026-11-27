//! Metered capture and verification of complete whole-result invalidation
//! footprints (combining result-set negative/related-record evidence with
//! returned-row positive projection evidence).
use std::cmp::Ordering;
use std::sync::Arc;

use worth_execution::ExecutionRequest;
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryRequestInterruption, WorthQueryRequestScope,
};
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::snapshots::SnapshotHandle;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

use crate::domain_computation::primary_graph::application_query::observed_source::{
    SourceFactExpansionControl, WorthQueryObservedResultSet, WorthQueryObservedSource,
    WorthQuerySourceExpectationDenial, WorthQuerySourceExpectationDenialKind,
};
use crate::domain_computation::primary_graph::output_reuse::{
    first_moved_fact_in, RetainedFactStop, RetainedFactVerification,
};
use crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationObservedFact as Fact, WorthQueryPrimaryGraphLayout,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WholeResultFootprintStatus {
    Current,
    Invalidated,
}

pub enum FootprintStop {
    Cancelled,
    DeadlineExceeded,
    Execution(Cause),
    Unavailable,
}

/// Complete deduplicated physical facts observed by one whole-result query
/// execution, carrying its own admitted memory reservation.
///
/// Complexity:
///   Capture: O(F log F) over F total result-set + row facts.
///   Verify:  O(F) bounded native currentness probes, short-circuiting on the
///            first moved fact.
pub struct WholeResultInvalidationFootprint {
    facts: Vec<Fact>,
    _allocation: Arc<SourceFactsAllocation>,
}

impl WholeResultInvalidationFootprint {
    pub fn capture<Query>(
        execution: &ExecutionRequest<'_, '_>,
        layout: &WorthQueryPrimaryGraphLayout,
        subject: &str,
        request: &WorthQueryRequestScope,
        result_set: &WorthQueryObservedResultSet<Query>,
        row_sources: &[WorthQueryObservedSource<Query>],
    ) -> Result<Self, FootprintStop> {
        check_interruption(request)?;
        let mut total_hint = 0_usize;
        let mut total_retained_payload_bytes = 0_u64;
        let mut max_peak_scratch_bytes = 0_u64;
        let mut upfront_work = 1_u64;
        for source in std::iter::once(result_set.source()).chain(row_sources.iter()) {
            let quote = source
                .fact_conversion_quote()
                .ok_or(FootprintStop::Execution(Cause::WorkCounterOverflow))?;
            total_hint = total_hint
                .checked_add(quote.fact_count)
                .ok_or(FootprintStop::Execution(Cause::WorkCounterOverflow))?;
            total_retained_payload_bytes = total_retained_payload_bytes
                .checked_add(quote.retained_payload_bytes)
                .ok_or(FootprintStop::Execution(Cause::ChargedBytesOverflow))?;
            max_peak_scratch_bytes = max_peak_scratch_bytes.max(quote.peak_scratch_bytes);
            upfront_work = upfront_work
                .checked_add(quote.conversion_work)
                .ok_or(FootprintStop::Execution(Cause::WorkCounterOverflow))?;
        }
        SourceFactsAllocation::work(execution, upfront_work).map_err(map_cause)?;

        let allocation = SourceFactsAllocation::open(execution).map_err(map_cause)?;
        // Permanently charge both retained Fact slots and all surviving field
        // path vectors / aspect & field strings on `allocation` so they outlive
        // `scratch` across the full lifetime of this footprint.
        allocation
            .claim_slice::<Fact>(execution, total_hint)
            .map_err(map_cause)?;
        if total_retained_payload_bytes > 0 {
            allocation
                .claim(execution, total_retained_payload_bytes)
                .map_err(map_cause)?;
        }
        let scratch = SourceFactsAllocation::open(execution).map_err(map_cause)?;
        if max_peak_scratch_bytes > 0 {
            scratch
                .claim(execution, max_peak_scratch_bytes)
                .map_err(map_cause)?;
        }
        let control = SourceFactExpansionControl::in_request(*execution, Some(request));

        let mut combined = Vec::new();
        combined
            .try_reserve_exact(total_hint)
            .map_err(|_| FootprintStop::Execution(Cause::AllocationFailed))?;
        let mut claimed_slots = total_hint;
        if combined.capacity() > claimed_slots {
            allocation
                .claim_slice::<Fact>(execution, combined.capacity() - claimed_slots)
                .map_err(map_cause)?;
            claimed_slots = combined.capacity();
        }
        let mut append = |source: &WorthQueryObservedSource<Query>| -> Result<(), FootprintStop> {
            check_interruption(request)?;
            let expanded = source
                .validated_facts(layout, subject, control)
                .map_err(map_expectation_denial)?;
            let needed = combined
                .len()
                .checked_add(expanded.len())
                .ok_or(FootprintStop::Execution(Cause::ChargedBytesOverflow))?;
            if needed > claimed_slots {
                allocation
                    .claim_slice::<Fact>(execution, needed - claimed_slots)
                    .map_err(map_cause)?;
                claimed_slots = needed;
            }
            if needed > combined.capacity() {
                combined
                    .try_reserve_exact(needed - combined.len())
                    .map_err(|_| FootprintStop::Execution(Cause::AllocationFailed))?;
                if combined.capacity() > claimed_slots {
                    allocation
                        .claim_slice::<Fact>(execution, combined.capacity() - claimed_slots)
                        .map_err(map_cause)?;
                    claimed_slots = combined.capacity();
                }
            }
            combined.extend(expanded);
            Ok(())
        };
        append(result_set.source())?;
        for row in row_sources {
            append(row)?;
        }
        drop(scratch);

        let facts = deduplicate_facts(execution, request, combined)?;
        Ok(Self {
            facts,
            _allocation: allocation,
        })
    }

    pub fn verify_at_snapshot(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        request: &WorthQueryRequestScope,
    ) -> Result<WholeResultFootprintStatus, FootprintStop> {
        check_interruption(request)?;
        if !self.facts.is_empty() {
            let scan_work = u64::try_from(self.facts.len())
                .map_err(|_| FootprintStop::Execution(Cause::WorkCounterOverflow))?;
            SourceFactsAllocation::work(execution, scan_work).map_err(map_cause)?;
        }
        let query_control = || match request.interruption() {
            None => Ok(()),
            Some(WorthQueryRequestInterruption::Cancelled) => Err(Cause::Cancelled),
            Some(WorthQueryRequestInterruption::DeadlineExceeded) => Err(Cause::DeadlineElapsed),
        };
        let mut verification =
            RetainedFactVerification::new(execution, runtime, snapshot, Some(&query_control));
        match first_moved_fact_in(&mut verification, self.facts.iter()) {
            Ok(None) => Ok(WholeResultFootprintStatus::Current),
            Ok(Some(_)) => Ok(WholeResultFootprintStatus::Invalidated),
            Err(stop) => Err(map_fact_stop(stop)),
        }
    }
}

fn fact_tag(fact: &Fact) -> u8 {
    use worth_relational::facade::runtime::RelationalAdjacencyDirection as Dir;
    match fact {
        Fact::SourceReadAuthorization { .. } => 0,
        Fact::ManagedIndexSource { .. } => 1,
        Fact::ComponentSelection { .. } => 2,
        Fact::Aggregate { .. } => 3,
        Fact::DerivedSource { .. } => 4,
        Fact::StructuralWalk { .. } => 5,
        Fact::SourceEntity { .. } => 6,
        Fact::SourceAspectRevision { .. } => 7,
        Fact::SourceFieldRevision { .. } => 8,
        Fact::SourceAdjacencyRevision {
            direction: Dir::Outgoing,
            ..
        } => 9,
        Fact::SourceAdjacencyRevision {
            direction: Dir::Incoming,
            ..
        } => 10,
        Fact::PersistentNameHistory { .. } => 11,
        Fact::PersistentGroup { .. } => 12,
        _ => 13,
    }
}

fn cmp_source_fact(left: &Fact, right: &Fact) -> Ordering {
    let tag_ord = fact_tag(left).cmp(&fact_tag(right));
    if tag_ord != Ordering::Equal {
        return tag_ord;
    }
    match (left, right) {
        (
            Fact::SourceReadAuthorization { observation: a },
            Fact::SourceReadAuthorization { observation: b },
        ) => a.key().cmp(b.key()),
        (
            Fact::ManagedIndexSource { observation: a },
            Fact::ManagedIndexSource { observation: b },
        ) => a.key().cmp(b.key()),
        (
            Fact::ComponentSelection { observation: a },
            Fact::ComponentSelection { observation: b },
        ) => a.key().cmp(b.key()),
        (Fact::Aggregate { observation: a }, Fact::Aggregate { observation: b }) => {
            a.key().cmp(b.key())
        }
        (Fact::DerivedSource { observation: a }, Fact::DerivedSource { observation: b }) => {
            a.key().cmp(b.key())
        }
        (Fact::StructuralWalk { observation: a }, Fact::StructuralWalk { observation: b }) => {
            a.key().cmp(b.key())
        }
        (Fact::SourceEntity { entity_id: a }, Fact::SourceEntity { entity_id: b }) => a.cmp(b),
        (
            Fact::SourceAspectRevision {
                entity_id: ae,
                aspect: aa,
                ..
            },
            Fact::SourceAspectRevision {
                entity_id: be,
                aspect: ba,
                ..
            },
        ) => ae.cmp(be).then_with(|| aa.cmp(ba)),
        (
            Fact::SourceFieldRevision {
                entity_id: ae,
                locator: al,
                ..
            },
            Fact::SourceFieldRevision {
                entity_id: be,
                locator: bl,
                ..
            },
        ) => ae.cmp(be).then_with(|| al.cmp(bl)),
        (
            Fact::SourceAdjacencyRevision {
                anchor: aa,
                relation_kind: ar,
                ..
            },
            Fact::SourceAdjacencyRevision {
                anchor: ba,
                relation_kind: br,
                ..
            },
        ) => aa.cmp(ba).then_with(|| ar.cmp(br)),
        (
            Fact::PersistentNameHistory { read: a, .. },
            Fact::PersistentNameHistory { read: b, .. },
        ) => a.cmp(b),
        (Fact::PersistentGroup { read: a, .. }, Fact::PersistentGroup { read: b, .. }) => a.cmp(b),
        _ => Ordering::Equal,
    }
}

fn n_log2_n(count: usize) -> Option<u64> {
    if count <= 1 {
        return u64::try_from(count).ok();
    }
    let log2 = u64::from(usize::BITS - count.leading_zeros()).max(1);
    u64::try_from(count).ok()?.checked_mul(log2)
}

fn deduplicate_facts(
    execution: &ExecutionRequest<'_, '_>,
    request: &WorthQueryRequestScope,
    mut facts: Vec<Fact>,
) -> Result<Vec<Fact>, FootprintStop> {
    let count = facts.len();
    if count <= 1 {
        return Ok(facts);
    }
    check_interruption(request)?;
    let dedup_work = n_log2_n(count).ok_or(FootprintStop::Execution(Cause::WorkCounterOverflow))?;
    SourceFactsAllocation::work(execution, dedup_work).map_err(map_cause)?;

    sort_and_deduplicate(&mut facts);
    check_interruption(request)?;
    Ok(facts)
}

fn check_interruption(request: &WorthQueryRequestScope) -> Result<(), FootprintStop> {
    match request.interruption() {
        None => Ok(()),
        Some(WorthQueryRequestInterruption::Cancelled) => Err(FootprintStop::Cancelled),
        Some(WorthQueryRequestInterruption::DeadlineExceeded) => {
            Err(FootprintStop::DeadlineExceeded)
        }
    }
}

fn map_cause(cause: Cause) -> FootprintStop {
    match cause {
        Cause::Cancelled => FootprintStop::Cancelled,
        Cause::DeadlineElapsed => FootprintStop::DeadlineExceeded,
        other => FootprintStop::Execution(other),
    }
}

fn map_expectation_denial(denial: WorthQuerySourceExpectationDenial) -> FootprintStop {
    if let Some(interruption) = denial.retention_interruption() {
        return match interruption {
            WorthQueryRequestInterruption::Cancelled => FootprintStop::Cancelled,
            WorthQueryRequestInterruption::DeadlineExceeded => FootprintStop::DeadlineExceeded,
        };
    }
    match denial.kind() {
        WorthQuerySourceExpectationDenialKind::WorkBudgetExceeded => {
            FootprintStop::Execution(Cause::WorkExhausted)
        }
        WorthQuerySourceExpectationDenialKind::PreparationMemoryExceeded => {
            FootprintStop::Execution(Cause::AllocationFailed)
        }
        WorthQuerySourceExpectationDenialKind::RetainedSourceStorageDenied => {
            if denial.execution_denial().is_some() {
                FootprintStop::Execution(Cause::WorkExhausted)
            } else if denial.allocation_denial().is_some() {
                FootprintStop::Execution(Cause::AllocationFailed)
            } else {
                FootprintStop::Unavailable
            }
        }
        _ => FootprintStop::Unavailable,
    }
}

fn map_fact_stop(stop: RetainedFactStop<'_>) -> FootprintStop {
    match stop {
        RetainedFactStop::Execution(cause) => map_cause(cause),
        RetainedFactStop::Interrupted(event) => match event.interruption() {
            worth_relational::facade::mvcc::RelationalOperationInterruption::Cancelled => {
                FootprintStop::Cancelled
            }
            worth_relational::facade::mvcc::RelationalOperationInterruption::TimedOut => {
                FootprintStop::DeadlineExceeded
            }
        },
        RetainedFactStop::Work => FootprintStop::Execution(Cause::WorkExhausted),
        RetainedFactStop::ManagedIndex(_)
        | RetainedFactStop::OutputObservation(_)
        | RetainedFactStop::Unavailable(_)
        | RetainedFactStop::Refused(_) => FootprintStop::Unavailable,
    }
}

/// Sorts `facts` canonically and drops exact duplicates only: two facts the
/// order cannot tell apart are both kept unless they are equal, so no fact
/// ever stands in for another.
fn sort_and_deduplicate(facts: &mut Vec<Fact>) {
    facts.sort_unstable_by(cmp_source_fact);
    facts.dedup_by(|a, b| cmp_source_fact(a, b) == Ordering::Equal && a == b);
}

#[cfg(test)]
#[path = "footprint/tests.rs"]
mod tests;
