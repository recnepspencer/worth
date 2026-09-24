use super::*;
use crate::domain_computation::primary_graph::{
    output_reuse::{
        compare_retained_output_dependencies, require_installed_output_dependencies,
        OutputDependencySelection,
    },
    WorthQueryApplicationBasisSelectionIdentity, WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_relational::facade::{runtime::ProjectionAspectScope, storage::RecordLifecycleState};

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    pub fn select_output_producer<Family>(
        &self,
        source: &WorthQueryObservedSource<
            <<Family as WorthQueryProducerOutputFamily<Schema>>::Source as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::Query,
        >,
        profile_kind: &'static str,
        maximum_work: usize,
    ) -> Result<WorthQuerySelectedApplicationProducer, WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        if source.runtime_authority != self.runtime.authority_identity().as_u64()
            || source.schema_binding != self.installed_schema.binding_identity()
        {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                Family::IDENTITY,
            ));
        }
        let installed = self
            .installed_schema
            .installed_query_binding::<Family::Source>()
            .map_err(|_| {
                WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::ForeignSource,
                    Family::IDENTITY,
                )
            })?;
        require_installed_output_dependencies(installed.query().output_dependencies(), source)?;
        let WorthQueryApplicationBasisSelectionIdentity::Product(observation) = &source.selection
        else {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                Family::IDENTITY,
            ));
        };
        let output_bindings = self.installed_producers.family_output_bindings::<Family>();
        // WORTH-UI-TEMPORARY-INSTRUMENTATION
        let trace_geometry = std::env::var_os("WORTH_REOPEN_TRACE").is_some()
            && Family::IDENTITY.to_ascii_lowercase().contains("geometry");
        let scope = crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(source.source_root());
        let mut lineage = self
            .primary_provider
            .graph
            .output_lineage
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for recovered in &self.recovered_outputs {
            let checkpoint = &recovered.checkpoint;
            let Some(binding) = recovered.correspondence.binding_type() else {
                continue;
            };
            if checkpoint.scope != scope
                || checkpoint.source_partition != source.partition_identity()
                || !output_bindings.contains(&binding)
            {
                continue;
            }
            lineage.record_recovered_prior_output(
                binding,
                self.runtime.authority_identity().as_u64(),
                self.installed_schema.binding_identity(),
                scope,
                observation.lifecycle_incarnation(),
                observation.reference_generation().get(),
                std::sync::Arc::clone(&recovered.correspondence),
                crate::domain_computation::primary_graph::output_lineage::RecordedSourceIdentity::Checkpoint(
                    crate::domain_computation::primary_graph::application_query::WorthQueryCheckpointSourceIdentity::new(checkpoint.source),
                ),
                checkpoint.source_partition,
                checkpoint.producer_dependency,
                checkpoint.idempotency_key,
                checkpoint.resources,
            );
        }
        let (candidates, lineage_work) = lineage
            .retained_output_candidates(
                self.runtime.authority_identity().as_u64(),
                &self.installed_schema.binding_identity(),
                scope,
                observation.lifecycle_incarnation(),
                observation.reference_generation().get(),
                &output_bindings,
                source.partition_identity(),
                maximum_work,
            )
            .map_err(|()| selection_budget_denial(Family::IDENTITY))?;
        if trace_geometry {
            eprintln!(
                "[WORTH_REOPEN_TRACE] geometry selection {}: bindings={}, candidates={}, lineage_work={}",
                Family::IDENTITY,
                output_bindings.len(),
                candidates.len(),
                lineage_work
            );
        }
        drop(lineage);
        let mut live_candidates = Vec::new();
        for candidate in candidates {
            let role = self
                .installed_producers
                .family_output_role::<Family>(candidate.binding)?;
            if let Some(entity) = candidate.correspondence.active_entity_for_role(role) {
                live_candidates.push((candidate, entity));
            }
        }
        if trace_geometry {
            eprintln!(
                "[WORTH_REOPEN_TRACE] geometry selection {}: active_candidates={}",
                Family::IDENTITY,
                live_candidates.len()
            );
        }
        if !live_candidates.is_empty() {
            // One admission plus one lifecycle lookup per candidate must fit before
            // retaining a basis or reading its relational snapshot.
            let required_work = lineage_work
                .checked_add(1)
                .and_then(|work| work.checked_add(live_candidates.len()))
                .ok_or_else(|| selection_budget_denial(Family::IDENTITY))?;
            if required_work > maximum_work {
                return Err(selection_budget_denial(Family::IDENTITY));
            }
            let selected = self
                .on_branch(observation.product_branch())
                .select()
                .map_err(|denial| {
                    WorthQueryOutputDemandDenial::product_selection(
                        denial,
                        "output lifecycle basis could not be selected",
                    )
                })?;
            if crate::basis::WorthQueryProductBranchReadIdentity::from_observation(
                selected.product().observation(),
            ) != *observation
            {
                return Err(WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::Superseded,
                    Family::IDENTITY,
                )
                .with_recovery_posture(WorthQueryOutputDemandRecoveryPosture::Retryable));
            }
            let current = self.primary_provider.graph.with_runtime(|runtime| {
                let truth = runtime.read_truth();
                let view = truth
                    .project_snapshot(selected.application_basis().snapshot_handle())
                    .ok_or_else(|| {
                        WorthQueryOutputDemandDenial::new(
                            WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                            Family::IDENTITY,
                        )
                    })?;
                let mut remaining_work = maximum_work - required_work;
                let mut current = Vec::with_capacity(live_candidates.len());
                for (candidate, entity) in &live_candidates {
                    let live = view.entity_record_with_projection_scope(
                        *entity,
                        ProjectionAspectScope::empty(),
                        |record| Some(record.lifecycle()),
                    ) == Some(RecordLifecycleState::Live);
                    let identity_current = candidate.source_identity.is_some_and(|identity| {
                        match identity {
                            crate::domain_computation::primary_graph::output_lineage::RecordedSourceIdentity::Runtime(runtime) => runtime == source.idempotency_identity(),
                            crate::domain_computation::primary_graph::output_lineage::RecordedSourceIdentity::Checkpoint(checkpoint) => checkpoint == source.checkpoint_identity(),
                        }
                    });
                    let facts_current = live
                        && matches!(
                            compare_retained_output_dependencies(
                                runtime,
                                selected.application_basis().snapshot_handle(),
                                identity_current,
                                candidate.observed_source_facts.as_deref(),
                                &mut remaining_work,
                            )?,
                            OutputDependencySelection::Reuse
                        );
                    if trace_geometry {
                        eprintln!(
                            "[WORTH_REOPEN_TRACE] geometry selection {} candidate: live={}, identity_current={}, facts_present={}, facts_current={}, resources_present={}",
                            Family::IDENTITY,
                            live,
                            identity_current,
                            candidate.observed_source_facts.as_ref().is_some_and(|facts| !facts.is_empty()),
                            facts_current,
                            candidate.resources.is_some()
                        );
                    }
                    current.push((live, facts_current));
                }
                Ok::<_, WorthQueryOutputDemandDenial>(current)
            })?;
            let mut retained_output = false;
            for ((candidate, _), (live, facts_current)) in live_candidates.into_iter().zip(current)
            {
                if !live {
                    continue;
                }
                if facts_current {
                    let resources = candidate.resources.ok_or_else(|| {
                        WorthQueryOutputDemandDenial::new(
                            WorthQueryOutputDemandDenialKind::IncompleteDependencyCoverage,
                            "retained output lacks its producer resource profile",
                        )
                    })?;
                    let mut selected = self
                        .installed_producers
                        .select_exact::<Family>(candidate.binding)?;
                    selected.retained_resources = Some(resources);
                    if trace_geometry {
                        eprintln!(
                            "[WORTH_REOPEN_TRACE] geometry selection {}: Reuse",
                            Family::IDENTITY
                        );
                    }
                    return Ok(selected);
                }
                retained_output = true;
            }
            if retained_output {
                if trace_geometry {
                    eprintln!(
                        "[WORTH_REOPEN_TRACE] geometry selection {}: Preserve",
                        Family::IDENTITY
                    );
                }
                return self.installed_producers.select::<Family>(
                    WorthQueryProducerApplicability::new(
                        profile_kind,
                        WorthQueryProducerLifecyclePosture::Preserve,
                    ),
                );
            }
        }
        if trace_geometry {
            eprintln!(
                "[WORTH_REOPEN_TRACE] geometry selection {}: Initial",
                Family::IDENTITY
            );
        }
        self.installed_producers
            .select::<Family>(WorthQueryProducerApplicability::new(
                profile_kind,
                WorthQueryProducerLifecyclePosture::Initial,
            ))
    }
}

fn selection_budget_denial(family: &str) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded, family)
        .with_recovery_posture(WorthQueryOutputDemandRecoveryPosture::Retryable)
}
