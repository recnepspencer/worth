//! The inverse of a stable record's fact composition.

use std::{mem::size_of, sync::Arc};

use super::stable_record::charge_initialized_clone;
use crate::domain_computation::primary_graph::{
    output_lineage::{
        invalidation::{arc_slice_bytes, retained_fact_payload_bytes, InvalidationEditAdmission},
        retained_capacity::{LineageRetentionLedger, RetainedLineageCapacity},
        SealedNativeOutputWitness,
    },
    WorthQueryApplicationObservedFact as Fact,
};

/// A stable alias holds its origin's handler prefix, then that origin's
/// performed output projected as facts, then the alias's own source suffix.
/// A row that seals a witness of its own holds the performed sequence: the
/// prefix and the source suffix. Every allocation and the retained custody
/// are admitted before a fact is copied; `None` means the alias does not hold
/// that composition or the copy was not admitted.
pub(in crate::domain_computation::primary_graph::output_lineage) fn performed_fact_sequence(
    alias_facts: &[Fact],
    prefix_count: usize,
    origin_witness: &SealedNativeOutputWitness,
    retention: &LineageRetentionLedger,
    admission: &mut InvalidationEditAdmission,
) -> Option<(Arc<[Fact]>, RetainedLineageCapacity)> {
    let projected = origin_witness
        .prepare_fact_projection(admission)
        .ok()??
        .count();
    let prefix = alias_facts.get(..prefix_count)?;
    let suffix = alias_facts.get(prefix_count.checked_add(projected)?..)?;
    let total = prefix.len().checked_add(suffix.len())?;
    let mut payload = 0u64;
    for fact in prefix.iter().chain(suffix) {
        admission.charge_external_work(1).ok()?;
        payload = payload.checked_add(retained_fact_payload_bytes(fact, admission).ok()?)?;
        charge_initialized_clone(fact, admission).ok()?;
    }
    // Building the flat Vec initializes every row; Arc conversion copies
    // every initialized row again.
    let count = u64::try_from(total).ok()?;
    admission
        .charge_external_work(count.checked_mul(2)?.checked_add(5)?)
        .ok()?;
    let vec_bytes = u64::try_from(total.checked_mul(size_of::<Fact>())?).ok()?;
    let arc_bytes = arc_slice_bytes::<Fact>(total)?;
    admission
        .admit_read_scratch(vec_bytes.checked_add(arc_bytes)?.checked_add(payload)?)
        .ok()?;
    let capacity = retention.reserve(arc_bytes.checked_add(payload)?).ok()?;
    let mut facts = Vec::with_capacity(total);
    facts.extend(prefix.iter().cloned());
    facts.extend(suffix.iter().cloned());
    Some((Arc::from(facts.into_boxed_slice()), capacity))
}
