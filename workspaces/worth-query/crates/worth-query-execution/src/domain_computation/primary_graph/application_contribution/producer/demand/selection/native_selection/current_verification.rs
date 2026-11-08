//! Native liveness, producing build, source facts and witness form one verdict.
use super::*;
use crate::domain_computation::primary_graph::output_lineage::current_output::retained_candidates::WorthQueryRetainedOutputCandidate;

pub(super) fn verify<Schema, Family>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &mut worth_relational::facade::runtime::RelationalRuntime,
    selected: &crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<
        '_,
        Schema,
    >,
    source: &WorthQueryObservedSource<FamilySourceQuery<Schema, Family>>,
    live_candidates: &[(
        WorthQueryRetainedOutputCandidate,
        worth_relational::facade::identity::EntityId,
    )],
) -> Result<Vec<(bool, bool)>, WorthQueryOutputDemandDenial>
where
    Schema: ApplicationSchema + 'static,
    Family: WorthQueryProducerOutputFamily<Schema>,
{
    let truth = runtime.read_truth();
    let view = truth
        .project_snapshot(selected.application_basis().snapshot_handle())
        .ok_or_else(|| {
            WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                Family::IDENTITY,
            )
        })?;
    let mut current = Vec::with_capacity(live_candidates.len());
    use crate::domain_computation::primary_graph::output_lineage::RecordedSourceIdentity;
    for (candidate, entity) in live_candidates {
        let live = retained_liveness::read(execution, &view, *entity)?;
        if candidate.historical_selection.is_some() {
            let exact = live
                && application.restored_output_is_current::<Family>(
                    execution, runtime, selected, candidate,
                )?;
            current.push((live, exact));
            continue;
        }
        let identity_current = candidate
            .source_identity
            .is_some_and(|identity| match identity {
                RecordedSourceIdentity::Runtime(runtime) => {
                    runtime == source.idempotency_identity()
                }
                RecordedSourceIdentity::Checkpoint(checkpoint) => {
                    checkpoint == source.checkpoint_identity()
                }
            });
        // The source facts and the performed output witness are
        // one fact set: either half moving ends exact reuse, and a
        // live output selects Preserve. Checkpoints state the output half among their
        // facts. A row of this runtime proves it by a sealed
        // witness and a settlement the cutoff verifies: one the
        // cutoff declines is not exact. A settlement it verifies
        // in full is compared below like any other.
        let witness = candidate
            .native_output_witness
            .as_ref()
            .and_then(|witness| witness.get());
        let owner = &application
            .runtime
            .primary_provider
            .graph
            .source_owner
            .invalidation_owner;
        let proven = live
            && match candidate.source_identity {
                Some(RecordedSourceIdentity::Checkpoint(_)) => true,
                _ => {
                    witness.is_some()
                        && retained_output_settlement_is_verified(
                            runtime,
                            selected.application_basis().snapshot_handle(),
                            candidate.verification_requirement,
                            &candidate.settlement_identity,
                            owner,
                        )?
                }
            };
        // Existing live output determines Preserve even when its producing
        // build cannot reuse. Identity never substitutes for native liveness.
        let (_, installed) = application
            .installed_producers
            .select_exact::<Family>(candidate.binding)?;
        let implementation_current = candidate.producing_implementation.is_some_and(|original| {
            original
                == worth_relational::facade::mvcc::RequiredOutputProducingImplementation::new(
                    installed.edition.build_identity(),
                    installed.edition.digest(),
                )
        });
        let facts_current = implementation_current
            && proven
            && matches!(
                compare_retained_output_dependencies(
                    execution,
                    runtime,
                    selected.application_basis().snapshot_handle(),
                    identity_current,
                    candidate.observed_source_facts.as_deref(),
                )?,
                OutputDependencySelection::Reuse
            )
            && matches!(
                compare_retained_output_witness(
                    execution,
                    runtime,
                    selected.application_basis().snapshot_handle(),
                    witness,
                )?,
                OutputDependencySelection::Reuse
            );
        current.push((live, facts_current));
    }
    Ok::<_, WorthQueryOutputDemandDenial>(current)
}
