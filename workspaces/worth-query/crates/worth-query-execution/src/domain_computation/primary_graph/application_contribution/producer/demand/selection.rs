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
        let (candidates, lineage_work) = self
            .primary_provider
            .graph
            .output_lineage
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .retained_output_candidates(
                self.runtime.authority_identity().as_u64(),
                &self.installed_schema.binding_identity(),
                crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(source.source_root()),
                observation.lifecycle_incarnation(),
                observation.reference_generation().get(),
                &output_bindings,
                source.partition_identity(),
                maximum_work,
            )
            .map_err(|()| selection_budget_denial(Family::IDENTITY))?;
        let mut live_candidates = Vec::new();
        for (binding, correspondence, source_identity, source_facts) in candidates {
            let role = self
                .installed_producers
                .family_output_role::<Family>(binding)?;
            if let Some(entity) = correspondence.active_entity_for_role(role) {
                live_candidates.push((binding, entity, source_identity, source_facts));
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
                for (_, entity, identity, facts) in &live_candidates {
                    let live = view.entity_record_with_projection_scope(
                        *entity,
                        ProjectionAspectScope::empty(),
                        |record| Some(record.lifecycle()),
                    ) == Some(RecordLifecycleState::Live);
                    let mut facts_current = live;
                    if live && *identity == Some(source.idempotency_identity()) {
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
                    }
                    current.push((live, facts_current));
                }
                Ok::<_, WorthQueryOutputDemandDenial>(current)
            })?;
            let mut retained_output = false;
            for ((binding, _, identity, _), (live, facts_current)) in
                live_candidates.into_iter().zip(current)
            {
                if !live {
                    continue;
                }
                if identity == Some(source.idempotency_identity()) && facts_current {
                    return self.installed_producers.select_exact::<Family>(binding);
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
