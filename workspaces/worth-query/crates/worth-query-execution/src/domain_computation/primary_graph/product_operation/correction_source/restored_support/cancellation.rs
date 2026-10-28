//! Exact pairs from already selected canonical occurrences and performed evidence.
use super::super::{
    WorthQueryCorrectionDerivationDenial as Denial, WorthQuerySelectedProductOperation,
};
use super::WorthQueryCorrectionRestoredSupportDenial as Stop;
use crate::domain_computation::application_aftermath::WorthQueryAftermathCausalRole as Role;
use crate::domain_computation::primary_graph::provider::{
    CorrectionPerformedWitness, CorrectionWitnessDenial,
};
use std::sync::Arc;
use worth_execution::{ExecutionOrderedMap, ExecutionRequest};
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::history::{
    CommitId, RelationalAuthoredWriteOccurrence, RelationalCommitReceipt,
};

pub(super) type Occurrences = ExecutionOrderedMap<CommitId, RelationalAuthoredWriteOccurrence>;
type Witnesses = ExecutionOrderedMap<CommitId, Arc<CorrectionPerformedWitness>>;

/// Every selected occurrence is covered once by its actual performed pair.
pub(super) fn require<Schema: ApplicationSchema>(
    target: &WorthQuerySelectedProductOperation<'_, Schema>,
    execution: ExecutionRequest<'_, '_>,
    occurrences: &Occurrences,
) -> Result<(), Denial> {
    let provider = &target.application().primary_provider;
    let mut witnesses = Witnesses::default();
    let mut covered = ExecutionOrderedMap::<CommitId, ()>::default();
    super::admit_work(execution, occurrences.len() as u64).map_err(Denial::RestoredSupport)?;
    for (commit, occurrence) in occurrences {
        let witness = provider
            .capture_correction_witness(execution, occurrence.receipt())
            .map_err(witness_denial)?
            .ok_or(Denial::RestoredSupport(Stop::CanonicalEvidenceUnavailable))?;
        super::ordered_visit(execution, witnesses.len()).map_err(Denial::RestoredSupport)?;
        witnesses
            .insert_cloned_native_for_request(commit, &witness, execution)
            .map_err(|cause| Denial::RestoredSupport(Stop::Storage(cause)))?;
    }
    super::admit_work(execution, occurrences.len() as u64).map_err(Denial::RestoredSupport)?;
    for (commit, reverse) in occurrences {
        execution
            .consult()
            .map_err(|cause| Denial::RestoredSupport(Stop::Request(cause)))?;
        super::ordered_visit(execution, witnesses.len()).map_err(Denial::RestoredSupport)?;
        let after = witnesses
            .get(commit)
            .expect("captured each selected occurrence");
        let Some(sealed) = after
            .aftermath
            .as_ref()
            .filter(|sealed| sealed.role() == Role::RecordedReverse)
        else {
            continue;
        };
        let source_id = sealed.recorded_source().relational().commit_id;
        super::ordered_visit(execution, occurrences.len()).map_err(Denial::RestoredSupport)?;
        let source = occurrences.get(&source_id).ok_or(Denial::SourceDiverged)?;
        super::ordered_visit(execution, witnesses.len()).map_err(Denial::RestoredSupport)?;
        let before = witnesses.get(&source_id).ok_or(Denial::SourceDiverged)?;
        require_pair(target, execution, source, reverse, before, after)?;
        for id in [source_id, *commit] {
            super::ordered_visit(execution, covered.len()).map_err(Denial::RestoredSupport)?;
            if covered.contains_key(&id) {
                return Err(Denial::SourceDiverged);
            }
            super::ordered_visit(execution, covered.len()).map_err(Denial::RestoredSupport)?;
            covered
                .insert_cloned_native_for_request(&id, &(), execution)
                .map_err(|cause| Denial::RestoredSupport(Stop::Storage(cause)))?;
        }
    }
    if covered.len() != occurrences.len() {
        return Err(Denial::SourceDiverged);
    }
    Ok(())
}

fn require_pair<Schema: ApplicationSchema>(
    target: &WorthQuerySelectedProductOperation<'_, Schema>,
    execution: ExecutionRequest<'_, '_>,
    source: &RelationalAuthoredWriteOccurrence,
    reverse: &RelationalAuthoredWriteOccurrence,
    before: &CorrectionPerformedWitness,
    after: &CorrectionPerformedWitness,
) -> Result<(), Denial> {
    let application = target.application();
    let sealed = after.aftermath.as_ref().ok_or(Denial::SourceDiverged)?;
    let mut work = 0u64;
    for receipt in [
        source.receipt(),
        reverse.receipt(),
        sealed.recorded_source().relational(),
        sealed.child(),
    ] {
        work = work
            .checked_add(receipt_work(receipt)?)
            .ok_or(Denial::RestoredSupport(Stop::Interrupted(
                worth_execution::MapKernelStop::WorkCounterOverflow,
            )))?;
    }
    super::admit_work(execution, work).map_err(Denial::RestoredSupport)?;
    if sealed.role() != Role::RecordedReverse
        || sealed.recorded_source().world() != &before.world
        || sealed.recorded_source().relational() != source.receipt()
        || sealed.child() != reverse.receipt()
        || reverse.receipt().parents.as_slice() != [sealed.parent().commit_id]
        || sealed.correction_lineage().selected_alternative().is_some()
        || sealed.correction_lineage().active_reversal().is_some()
    {
        return Err(Denial::RestoredSupport(Stop::CanonicalAssociationMismatch));
    }
    let allocations = reverse.allocated_records();
    super::admit_work(execution, allocations.len() as u64).map_err(Denial::RestoredSupport)?;
    if !allocations
        .into_iter()
        .any(|record| record == sealed.fact_record())
    {
        return Err(Denial::RestoredSupport(Stop::CanonicalAssociationMismatch));
    }
    if !application
        .primary_provider
        .validate_correction_witness(execution, target.product().relational_basis(), sealed)
        .map_err(witness_denial)?
    {
        return Err(Denial::RestoredSupport(Stop::CanonicalAssociationMismatch));
    }
    // World independently resolves both actual performed Product associations;
    // its existing ancestry work remains separate from selected posting visits.
    for (witness, occurrence) in [(before, source), (after, reverse)] {
        let ancestor = application
            .product_runtime
            .owner
            .observation_port()
            .select_retained_product_branch_ancestor(
                &execution,
                target.product().observation(),
                &witness.world,
                None,
            )
            .map_err(Denial::Ancestry)?;
        if ancestor
            .source()
            .basis()
            .relational_basis()
            .observation()
            .commit_id()
            != Some(occurrence.commit_id())
        {
            return Err(Denial::RestoredSupport(Stop::CanonicalAssociationMismatch));
        }
    }
    Ok(())
}

fn witness_denial(cause: CorrectionWitnessDenial) -> Denial {
    match cause {
        CorrectionWitnessDenial::Request(cause) | CorrectionWitnessDenial::CopyAdmission(cause) => {
            Denial::RestoredSupport(Stop::Request(cause))
        }
        CorrectionWitnessDenial::Interrupted(cause) => {
            Denial::RestoredSupport(Stop::Interrupted(cause))
        }
        CorrectionWitnessDenial::SizeOverflow => Denial::RestoredSupport(Stop::SizeOverflow),
        CorrectionWitnessDenial::Unavailable => {
            Denial::RestoredSupport(Stop::CanonicalEvidenceUnavailable)
        }
        CorrectionWitnessDenial::Basis(cause) => Denial::Basis(cause),
        CorrectionWitnessDenial::NativeRead(cause) => {
            Denial::RestoredSupport(Stop::NativeRead(cause))
        }
    }
}

fn receipt_work(receipt: &RelationalCommitReceipt) -> Result<u64, Denial> {
    (receipt.parents.len() as u64)
        .checked_mul(std::mem::size_of::<CommitId>() as u64)
        .and_then(|work| work.checked_add(receipt.branch_id.0.len() as u64))
        .and_then(|work| work.checked_add(std::mem::size_of_val(receipt) as u64))
        .ok_or(Denial::RestoredSupport(Stop::Interrupted(
            worth_execution::MapKernelStop::WorkCounterOverflow,
        )))
}
