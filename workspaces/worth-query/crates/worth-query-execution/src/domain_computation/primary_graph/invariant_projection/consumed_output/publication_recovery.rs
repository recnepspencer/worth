//! A freshly performed consumer restores missing derived upstream rows.
use crate::domain_computation::primary_graph::invariant_projection::{
    ConsumedOutputEvidence, ConsumedOutputVerification,
};
use crate::domain_computation::primary_graph::output_lineage::invalidation::{
    FullVerificationReason, InvalidationEditAdmission, SourceInvalidationOwner,
    SourceSettlementCurrentness,
};
use std::sync::Arc;
use worth_relational::facade::{
    runtime::{PositionedRelationalSnapshot, RelationalRuntime},
    snapshots::SnapshotHandle,
};

enum Visit<'a> {
    Enter(&'a ConsumedOutputEvidence),
    Established(&'a ConsumedOutputEvidence),
}

impl ConsumedOutputEvidence {
    pub(in crate::domain_computation::primary_graph) fn establish_missing(
        owner: &SourceInvalidationOwner,
        roots: &[ConsumedOutputEvidence],
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        selected: &PositionedRelationalSnapshot,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), FullVerificationReason> {
        let incomplete = FullVerificationReason::RegistrationIncomplete;
        let mut pending = Vec::new();
        reserve(&mut pending, roots.len(), admission)?;
        pending.extend(roots.iter().map(Visit::Enter));
        let mut checked = im::OrdSet::new();
        while let Some(visit) = pending.pop() {
            let evidence = match visit {
                Visit::Enter(evidence) => {
                    admission
                        .admit_visited_settlement(checked.len())
                        .map_err(|_| incomplete)?;
                    if checked.contains(evidence.identity()) {
                        continue;
                    }
                    checked.insert(Arc::clone(evidence.identity()));
                    let state = owner
                        .currentness(selected, evidence.identity(), admission)
                        .map_err(|_| incomplete)?;
                    if !state.no_row_answers() {
                        continue;
                    }
                    if matches!(state, SourceSettlementCurrentness::Foreign) {
                        return Err(incomplete);
                    }
                    // Postconditions cannot prove the original computation. This
                    // compares its original source facts and sealed output here.
                    if ConsumedOutputEvidence::verify_many_with_admission(
                        std::slice::from_ref(evidence),
                        owner,
                        runtime,
                        snapshot,
                        selected,
                        admission,
                    )
                    .map_err(|_| incomplete)?
                        != ConsumedOutputVerification::Current
                    {
                        return Err(FullVerificationReason::MissingSettlement);
                    }
                    reserve(
                        &mut pending,
                        evidence.upstream.len().saturating_add(1),
                        admission,
                    )?;
                    pending.push(Visit::Established(evidence));
                    pending.extend(evidence.upstream.iter().map(Visit::Enter));
                    continue;
                }
                Visit::Established(evidence) => evidence,
            };
            let witness = evidence
                .native_output_witness
                .as_ref()
                .and_then(|w| w.get())
                .ok_or(FullVerificationReason::NativeRevisionUnavailable)?;
            let upstream = crate::domain_computation::primary_graph::output_lineage::invalidation::collect_consumed_output_upstream(&*evidence.upstream, admission)
            .map_err(|_| incomplete)?;
            owner
                .establish_verified_consumed(
                    selected,
                    evidence.identity(),
                    &evidence.source_facts,
                    witness,
                    upstream,
                    admission,
                )
                .map_err(|_| incomplete)?;
        }
        Ok(())
    }
}

fn reserve<'a>(
    pending: &mut Vec<Visit<'a>>,
    additional: usize,
    admission: &mut InvalidationEditAdmission,
) -> Result<(), FullVerificationReason> {
    let incomplete = FullVerificationReason::RegistrationIncomplete;
    let bytes = pending
        .len()
        .checked_add(additional)
        .and_then(|count| count.checked_mul(std::mem::size_of::<Visit<'a>>()))
        .and_then(|n| u64::try_from(n).ok())
        .ok_or(incomplete)?;
    admission
        .admit_read_scratch(bytes)
        .map_err(|_| incomplete)?;
    admission
        .charge_external_work(pending.len() as u64)
        .map_err(|_| incomplete)?;
    pending
        .try_reserve_exact(additional)
        .map_err(|_| incomplete)
}
