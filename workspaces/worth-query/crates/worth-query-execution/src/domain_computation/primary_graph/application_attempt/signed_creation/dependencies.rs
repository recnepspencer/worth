//! Exact native revisions distinguish an untouched subject from an ABA edit.
use super::super::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationEffectProgram,
    WorthQueryApplicationObservedFact as Fact,
};
use super::sealing::WorthQuerySignedCreationPreparationDenial as Denial;
use crate::domain_computation::primary_graph::application_contribution::{
    QueryMemoryReservation, QueryRequestExecution,
};

pub(super) fn capture<Schema, Operation, Input, Scope>(
    execution: &QueryRequestExecution<'_>,
    mut candidate: WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
) -> Result<
    (
        WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>,
        QueryMemoryReservation,
    ),
    Denial,
> {
    let (count, bytes) = candidate
        .read_set
        .facts
        .iter()
        .filter_map(source)
        .try_fold((0usize, 0usize), |(count, bytes), (_, locator)| {
            Some((
                count.checked_add(1)?,
                bytes
                    .checked_add(size_of::<Fact>())?
                    .checked_add(locator.owned_allocation_capacity_bytes())?,
            ))
        })
        .ok_or(Denial::RepresentationCapacity)?;
    let memory = execution
        .reserve(u64::try_from(bytes).map_err(|_| Denial::RepresentationCapacity)?)
        .map_err(resource)?;
    let request = execution.execution_request().map_err(resource)?;
    let revisions = execution
        .decision_stage(|context| {
            let mut revisions = Vec::with_capacity(count);
            candidate
                .read_set
                .lease
                .handle()
                .with_runtime(&request, |runtime| {
                    let view = runtime
                        .read_truth()
                        .project_snapshot(candidate.read_set.lease.snapshot())
                        .ok_or(Denial::CandidateMismatch)?;
                    for (entity_id, locator) in candidate.read_set.facts.iter().filter_map(source) {
                        context.checkpoint(1).map_err(stopped)?;
                        // Body decision locators describe planned reads. Native
                        // MVCC revisions require the authoritative locator for
                        // that same installed field, never a value-only fallback.
                        let locator = worth_foundational::facade::AspectFieldLocator::new(
                            worth_foundational::facade::LocatorAuthority::Authoritative,
                            locator.aspect().aspect_key().clone(),
                            locator.field_path().clone(),
                        );
                        let native_revision = view
                            .entity_field_revision(&request, entity_id, &locator)
                            .map_err(|cause| {
                                Denial::Attempt(WorthQueryApplicationAttemptDenial::from(cause))
                            })?;
                        if native_revision.is_none() {
                            return Err(Denial::CandidateMismatch);
                        }
                        revisions.push(Fact::SourceFieldRevision {
                            entity_id,
                            locator,
                            native_revision,
                        });
                    }
                    Ok(revisions)
                })
        })
        .map_err(Denial::Execution)??;
    let facts = candidate.read_set.facts;
    let total = facts
        .len()
        .checked_add(revisions.len())
        .ok_or(Denial::RepresentationCapacity)?;
    candidate.read_set.facts = super::super::read_set::admit_array(
        total,
        facts.into_iter().chain(revisions),
        request.allocation_policy(),
        candidate.read_set.admission.operation(),
        || {
            candidate
                .read_set
                .admission
                .validate_current_authority()
                .map_err(WorthQueryApplicationAttemptDenial::request_authority_lost)
        },
    )
    .map_err(Denial::Attempt)?;
    Ok((candidate, memory))
}

fn source(
    fact: &Fact,
) -> Option<(
    worth_relational::facade::identity::EntityId,
    &worth_foundational::facade::AspectFieldLocator,
)> {
    match fact {
        Fact::Field {
            entity_id, locator, ..
        }
        | Fact::AbsentField {
            entity_id, locator, ..
        } => Some((*entity_id, locator)),
        _ => None,
    }
}
fn resource(
    cause: crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial,
) -> Denial {
    Denial::Execution(
        crate::domain_computation::primary_graph::WorthQueryAdvancementDenial::Resource(cause),
    )
}
fn stopped(cause: worth_execution::MapKernelStop) -> Denial {
    Denial::Execution(
        crate::domain_computation::primary_graph::application_contribution::advancement_denial(
            worth_execution::WorkCeilingDenial::Stopped(cause),
        ),
    )
}
