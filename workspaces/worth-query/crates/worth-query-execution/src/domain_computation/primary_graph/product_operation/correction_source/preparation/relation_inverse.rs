//! Exact relation inversion compares the performed actor's sealed material with Native's draft.
use super::*;
use crate::domain_computation::application_aftermath::{
    WorthQueryRelationPreImageDenial as Kind, WorthQueryRelationPreImageFailure as Failure,
};
use crate::domain_computation::primary_graph::admitted_capture::checkpoint;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout;
use crate::domain_computation::{
    WorthQueryProviderSessionDenialKind, WorthQueryProviderSessionFailure as Denial,
    WorthQueryProviderSessionProtocolStage,
};
use worth_relational::facade::{mvcc::ValidatedRelationalProposal, transactions::RecordRef};

impl WorthQueryPreparedCorrectionSource {
    pub(in crate::domain_computation::primary_graph) fn validate_relation_inverse(
        &self,
        candidate: &ValidatedRelationalProposal,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        layout: &WorthQueryPrimaryGraphLayout,
    ) -> Result<(), Denial> {
        if self.lineage.role() != WorthQueryAftermathCausalRole::RecordedReverse {
            return Ok(());
        }
        let evidence = self.material.commit_evidence();
        let demanded = evidence
            .original_authority()
            .installed_aftermath()
            .and_then(|contract| contract.mechanism())
            .and_then(|mechanism| match mechanism {
                worth_query_installation::facade::InstalledCorrectionMechanism::RecordedInverse(
                    inverse,
                ) => Some(inverse.preimage_demand()),
                _ => None,
            });
        let Some(demand) = demanded.filter(|demand| !demand.relations().is_empty()) else {
            return Ok(());
        };
        let missing = || {
            fail(
                Kind::MissingObservation,
                evidence
                    .output_correspondence()
                    .native_witness_roles()
                    .map(|(_, _, _, id)| RecordRef::Entity(id))
                    .collect(),
            )
        };
        let retained = evidence.retained_preimage().ok_or_else(missing)?;
        if retained.relation_loci() != demand.relations() {
            return Err(missing());
        }
        let kinds: Vec<_> = demand
            .relations()
            .iter()
            .map(|locus| {
                layout
                    .relation(locus.relation())
                    .map(|relation| relation.kind)
                    .ok_or_else(missing)
            })
            .collect::<Result<_, _>>()?;
        let inverse = candidate
            .relation_mutation_footprint(execution, &kinds)
            .map_err(|cause| {
                base("inverse relation mutation projection refused")
                    .with_native_preparation_error(cause)
            })?;
        let original = retained.relations();
        let actual = inverse.relations();
        let memory = execution
            .allocation_policy()
            .reserve_memory(actual.len() as u64)
            .map_err(|cause| Denial::allocation_denied(cause, "relation inverse preparation"))?;
        let mut used = Vec::new();
        used.try_reserve_exact(actual.len())
            .map_err(|_| base("inverse relation comparison allocation failed"))?;
        used.resize(actual.len(), false);
        execution
            .run_native_with_checkpoint(|context| {
                let remap = |id,
                             context: &mut worth_execution::MapKernelContext<'_, '_>|
                 -> Result<EntityId, Denial> {
                    checkpoint(context, inverse.restored_entities().len() as u64).map_err(
                        |cause| request_stop(worth_execution::WorkCeilingDenial::Stopped(cause)),
                    )?;
                    Ok(inverse
                        .restored_entities()
                        .iter()
                        .find_map(|(old, fresh)| (*old == id).then_some(*fresh))
                        .unwrap_or(id))
                };
                // Every performed relation still present must be removed by its own identity.
                for row in original {
                    let mutation = row.mutation();
                    let Some(ends) = mutation.after() else {
                        continue;
                    };
                    let mut matched = false;
                    for transition in actual {
                        checkpoint(context, 1).map_err(|cause| {
                            request_stop(worth_execution::WorkCeilingDenial::Stopped(cause))
                        })?;
                        if transition.relation() == mutation.relation()
                            && transition.kind() == mutation.kind()
                            && transition.before() == Some(ends)
                        {
                            matched = true;
                            break;
                        }
                    }
                    if !matched {
                        return Err(fail(
                            Kind::IncompleteInverse,
                            vec![RecordRef::Relation(mutation.relation())],
                        ));
                    }
                }
                for transition in actual {
                    if transition.before().is_none() {
                        continue;
                    }
                    let mut matched = false;
                    for row in original {
                        checkpoint(context, 1).map_err(|cause| {
                            request_stop(worth_execution::WorkCeilingDenial::Stopped(cause))
                        })?;
                        let mutation = row.mutation();
                        if mutation.relation() == transition.relation()
                            && mutation.kind() == transition.kind()
                            && mutation.after() == transition.before()
                        {
                            matched = true;
                            break;
                        }
                    }
                    if !matched {
                        return Err(fail(
                            Kind::ForeignEndpoint,
                            vec![RecordRef::Relation(transition.relation())],
                        ));
                    }
                }
                // Fresh relation IDs are allowed; their kind and remapped endpoint multiset is exact.
                for row in original {
                    let mutation = row.mutation();
                    let Some((from, to)) = mutation.before() else {
                        continue;
                    };
                    let expected = (remap(from, context)?, remap(to, context)?);
                    let mut matched = None;
                    for (index, transition) in actual.iter().enumerate() {
                        checkpoint(context, 1).map_err(|cause| {
                            request_stop(worth_execution::WorkCeilingDenial::Stopped(cause))
                        })?;
                        if !used[index]
                            && transition.kind() == mutation.kind()
                            && transition.after() == Some(expected)
                        {
                            matched = Some(index);
                            break;
                        }
                    }
                    match matched {
                        Some(index) => used[index] = true,
                        None => {
                            return Err(fail(
                                Kind::IncompleteInverse,
                                vec![RecordRef::Relation(mutation.relation())],
                            ))
                        }
                    }
                }
                let supplied = actual
                    .iter()
                    .filter(|transition| transition.after().is_some())
                    .count();
                let expected = original
                    .iter()
                    .filter(|row| row.mutation().before().is_some())
                    .count();
                checkpoint(context, (actual.len() + original.len()) as u64).map_err(|cause| {
                    request_stop(worth_execution::WorkCeilingDenial::Stopped(cause))
                })?;
                if supplied > expected {
                    let subject = actual
                        .iter()
                        .enumerate()
                        .find(|(index, transition)| transition.after().is_some() && !used[*index])
                        .expect("extra supplied endpoint pair")
                        .1
                        .relation();
                    return Err(fail(Kind::ExtraInverse, vec![RecordRef::Relation(subject)]));
                }
                Ok::<(), Denial>(())
            })
            .map_err(request_stop)?
            .0?;
        drop(memory);
        Ok(())
    }
}
fn base(detail: &str) -> Denial {
    Denial::new(
        WorthQueryProviderSessionDenialKind::ProviderRejected,
        WorthQueryProviderSessionProtocolStage::Commit,
        detail,
        Default::default(),
    )
}
fn fail(kind: Kind, records: Vec<RecordRef>) -> Denial {
    base("recorded inverse relation material does not match its source")
        .with_relation_preimage_failure(Failure::new(kind, records))
}
fn request_stop(cause: worth_execution::WorkCeilingDenial) -> Denial {
    Denial::checked_read_stopped(crate::domain_computation::primary_graph::application_contribution::publication_execution_kind(cause), WorthQueryProviderSessionProtocolStage::Commit, "relation inverse request stopped")
}
