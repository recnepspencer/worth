//! The completed Source's ordered row and empty-result dependency owners.
use super::{source_completion::SourceDenial, WorthQueryManagedIndexStop as Stop};
use crate::domain_computation::primary_graph::{
    application_query::{
        observed_source::{
            ManagedSourceDependencies, WorthQueryObservedResultSet, WorthQueryObservedSource,
            WorthQuerySourceExpectationDenial as Denial,
            WorthQuerySourceExpectationDenialKind as Kind,
        },
        one_shot::outcome::material::MaterialCompletion,
    },
    source_facts::SourceFactsAllocation,
};
use sha2::{Digest, Sha256};
use std::sync::Arc;

pub(super) fn capture<Q>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    rows: &[WorthQueryObservedSource<Q>],
    result_set: &WorthQueryObservedResultSet<Q>,
    allocation: &Arc<SourceFactsAllocation>,
) -> Result<Arc<ManagedSourceDependencies>, SourceDenial> {
    let storage = MaterialCompletion::new(execution, allocation);
    if !result_set.source().has_complete_output_dependencies() {
        return Err(Denial::new(Kind::IncompleteFootprint, "managed source set").into());
    }
    let owners = super::invocation::controlled(execution, |control| {
        let count = rows
            .len()
            .checked_add(1)
            .ok_or_else(|| control.refuse(Stop::CapacityOverflow))?;
        // Including the set owner in the bound prevents count overflow at the
        // later borrowed traversal; only row slots belong in this exact array.
        control.checkpoint(
            u64::try_from(count).map_err(|_| control.refuse(Stop::CapacityOverflow))?,
        )?;
        let mut owners =
            worth_execution::ExecutionRequestArrayBuilder::allocate(*execution, rows.len())
                .map_err(|cause| control.refuse(Stop::Storage(cause)))?;
        for row in rows {
            control.checkpoint(1)?;
            owners
                .push(Arc::clone(&row.source_meaning))
                .map_err(|cause| control.refuse(Stop::Storage(cause)))?;
        }
        owners
            .seal()
            .map_err(|cause| control.refuse(Stop::Storage(cause)).into())
    })?;
    let mut fact_count = 0usize;
    let mut field_count = 0usize;
    for source in rows.iter().chain(std::iter::once(result_set.source())) {
        storage.work(1)?;
        source.validate_completeness("managed source")?;
        if !ManagedSourceDependencies::accepts(&source.source_meaning) {
            return Err(Denial::new(Kind::SourceContractMismatch, "managed source").into());
        }
        // Derive counts during the admitted completed-footprint visit; a
        // later count does not traverse the Source roster or spend its grant.
        let footprint = source.source_meaning.footprint();
        fact_count = footprint
            .fact_count()
            .and_then(|count| fact_count.checked_add(count))
            .ok_or_else(|| Denial::new(Kind::PreparationMemoryExceeded, "managed source count"))?;
        field_count = footprint
            .field_count()
            .and_then(|count| field_count.checked_add(count))
            .ok_or_else(|| {
                Denial::new(
                    Kind::PreparationMemoryExceeded,
                    "managed source field count",
                )
            })?;
    }
    let mut digest = Sha256::new();
    let domain = b"worth-query:managed-source-dependencies:v1";
    storage.work(domain.len() as u64 + 8)?;
    digest.update(domain);
    digest.update(u64::try_from(rows.len()).map_err(|_| worth_relational::facade::transactions::RelationalExecutionDenialCause::WorkCounterOverflow)?.to_be_bytes());
    for source in rows.iter().chain(std::iter::once(result_set.source())) {
        storage.work(32)?;
        digest.update(source.checkpoint_observation_identity());
    }
    storage.arc::<ManagedSourceDependencies>()?;
    Ok(Arc::new(ManagedSourceDependencies {
        rows: owners,
        result_set: Arc::clone(&result_set.source().source_meaning),
        identity: digest.finalize().into(),
        fact_count,
        field_count,
        _allocation: Arc::clone(allocation),
    }))
}
