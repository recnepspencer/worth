use super::*;
use crate::domain_computation::primary_graph::{
    application_attempt::WorthQuerySourceCurrentnessFailure,
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
        let WorthQueryApplicationBasisSelectionIdentity::Product(observation) = &source.selection
        else {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                Family::IDENTITY,
            ));
        };
        let output_bindings = self.installed_producers.family_output_bindings::<Family>();
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
                    let mut facts_current = live && identity_current;
                    if facts_current {
                        if let Some(facts) = &candidate.observed_source_facts {
                        for fact in facts.iter() {
                            let (equal, work) = fact
                                .source_currentness_in(
                                    runtime,
                                    selected.application_basis().snapshot_handle(),
                                    remaining_work,
                                )
                                .map_err(|failure| match failure {
                                    WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded => {
                                        selection_budget_denial(Family::IDENTITY)
                                    }
                                    WorthQuerySourceCurrentnessFailure::Unavailable => {
                                        WorthQueryOutputDemandDenial::new(
                                            WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                                            Family::IDENTITY,
                                        )
                                    }
                                })?;
                            remaining_work -= work;
                            if !equal {
                                facts_current = false;
                                break;
                            }
                        }
                        } else if !matches!(candidate.source_identity, Some(crate::domain_computation::primary_graph::output_lineage::RecordedSourceIdentity::Checkpoint(_))) {
                            facts_current = false;
                        }
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
                    return self
                        .installed_producers
                        .select_exact::<Family>(candidate.binding);
                }
                retained_output = true;
            }
            if retained_output {
                return self.installed_producers.select::<Family>(
                    WorthQueryProducerApplicability::new(
                        profile_kind,
                        WorthQueryProducerLifecyclePosture::Preserve,
                    ),
                );
            }
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
