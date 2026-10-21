//! Sole producer source/lifecycle selection on the original native borrow.
use super::*;

impl<Schema: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(super) fn select_output_producer_with_remaining_core<Family>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: &WorthQueryObservedSource<
            <<Family as WorthQueryProducerOutputFamily<Schema>>::Source as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::Query,
        >,
        profile_kind: &'static str,

        retained_program_basis: Option<
            &crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation,
        >,
        selected_override: Option<&crate::domain_computation::primary_graph::product_operation::SharedSelectedProductOperation<'_, Schema>>,
        native: &mut crate::domain_computation::primary_graph::application_query::ApplicationQueryNativeAccess<'_>,
    ) -> Result<SelectedWithEntry<'_, Schema>, WorthQueryOutputDemandDenial>
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
        if let Some(retained_basis) = retained_program_basis {
            if let Some(shared) = selected_override {
                if !selected_basis::historical_basis_matches(self, retained_basis, shared)? {
                    return Err(WorthQueryOutputDemandDenial::new(
                        WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                        Family::IDENTITY,
                    ));
                }
            } else {
                let retained = self
                    .select_application_read_observation_with_request(execution, retained_basis)
                    .map_err(|cause| match cause {
                        cause
                        @ crate::basis::WorthQueryProductBranchAdmissionDenial::Stopped(_) => {
                            WorthQueryOutputDemandDenial::product_selection(cause, Family::IDENTITY)
                        }
                        crate::basis::WorthQueryProductBranchAdmissionDenial::Refused(_) => {
                            WorthQueryOutputDemandDenial::new(
                                WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                                Family::IDENTITY,
                            )
                        }
                    })?;
                if crate::basis::WorthQueryProductBranchReadIdentity::from_observation(
                    retained.product().observation(),
                ) != *observation
                {
                    return Err(WorthQueryOutputDemandDenial::new(
                        WorthQueryOutputDemandDenialKind::ForeignSource,
                        Family::IDENTITY,
                    ));
                }
            }
        }
        if let Some(shared) = selected_override {
            selected_basis::require_fresh_source(self, observation, Family::IDENTITY, shared)?;
        }
        let output_bindings = self.installed_producers.family_output_bindings::<Family>();
        let scope = crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(source.source_root());
        let mut lineage = self
            .primary_provider
            .graph
            .output_lineage
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        self.record_recovered_output_candidates::<Family>(
            source,
            observation,
            scope,
            &output_bindings,
            &mut lineage,
        );
        let candidates = lineage
            .retained_output_candidates_metered(
                self.runtime.authority_identity().as_u64(),
                &self.installed_schema.binding_identity(),
                scope,
                observation.lifecycle_incarnation(),
                observation.reference_generation().get(),
                &output_bindings,
                source.partition_identity(),
                execution,
            )
            .map_err(|stop| selected_basis::selection_lookup_denial(stop, Family::IDENTITY))?;
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
            // Debit basis admission and lifecycle visits before entering that
            // phase. The caller's remainder survives all subsequent failures.

            let newly_selected = if selected_override.is_none() {
                Some(
                    self.on_branch(observation.product_branch())
                        .select_with_request(execution)
                        .map_err(|denial| {
                            WorthQueryOutputDemandDenial::product_selection(
                                denial,
                                "output lifecycle basis could not be selected",
                            )
                        })?,
                )
            } else {
                None
            };
            let selected = selected_override.map_or_else(
                || {
                    newly_selected
                        .as_ref()
                        .expect("ordinary selection owns a basis")
                },
                |shared| shared.selected(),
            );
            let source_basis_is_admitted = if selected_override.is_some() {
                true // The exact selected owner check precedes candidate lookup.
            } else {
                source_basis_is_admitted(
                    observation,
                    selected.product().observation(),
                    retained_program_basis.is_some(),
                )
            };
            if !source_basis_is_admitted {
                return Err(WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::Superseded,
                    Family::IDENTITY,
                )
                .with_recovery_posture(WorthQueryOutputDemandRecoveryPosture::Retryable));
            }
            let graph = self.runtime.primary_graph().ok_or_else(|| {
                WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::ForeignSource,
                    Family::IDENTITY,
                )
            })?;
            let current = native.with_runtime_mut(graph, execution,
                || WorthQueryOutputDemandDenial::new(WorthQueryOutputDemandDenialKind::ForeignSource, Family::IDENTITY), |runtime| {
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
                for (candidate, entity) in &live_candidates {
                    let live=retained_liveness::read(execution,&view,*entity)?;
                    let identity_current = candidate.source_identity.is_some_and(|identity| {
                        match identity {
                            RecordedSourceIdentity::Runtime(runtime) => runtime == source.idempotency_identity(),
                            RecordedSourceIdentity::Checkpoint(checkpoint) => checkpoint == source.checkpoint_identity(),
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
                    let owner = &self.primary_provider.graph.source_owner.invalidation_owner;
                    let proven = live
                        && match candidate.source_identity {
                            Some(RecordedSourceIdentity::Checkpoint(_)) => true,
                            _ => {
                                witness.is_some()
                                    && retained_output_settlement_is_verified(runtime, selected.application_basis().snapshot_handle(), candidate.verification_requirement, &candidate.settlement_identity, owner)?
                            }
                        };
                    // Existing live output determines Preserve even when its producing
                    // build cannot reuse. Identity never substitutes for native liveness.
                    let (_, installed) = self.installed_producers.select_exact::<Family>(candidate.binding)?;
                    let implementation_current = candidate.producing_implementation.is_some_and(|original| {
                        original == worth_relational::facade::mvcc::RequiredOutputProducingImplementation::new(installed.edition.build_identity(), installed.edition.digest())
                    });
                    let facts_current = implementation_current && proven
                        && matches!(
                            compare_retained_output_dependencies(execution,
                                runtime,
                                selected.application_basis().snapshot_handle(),
                                identity_current,
                                candidate.observed_source_facts.as_deref(),
                            )?,
                            OutputDependencySelection::Reuse
                        )
                        && matches!(
                            compare_retained_output_witness(
                                runtime,
                                selected.application_basis().snapshot_handle(),
                                witness,
                            )?,
                            OutputDependencySelection::Reuse
                        );
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
                    let (mut selected, entry) = self
                        .installed_producers
                        .select_exact::<Family>(candidate.binding)?;
                    // A runtime record reaches the input cutoff and needs its
                    // declared decision proof. A current checkpoint record is
                    // readmitted through its complete recovered facts instead.
                    let recovered = matches!(candidate.source_identity, Some(
                        crate::domain_computation::primary_graph::output_lineage::RecordedSourceIdentity::Checkpoint(_)
                    ));
                    if recovered
                        || entry.declaration.input_reuse.is_some_and(|contract| {
                            contract.determinism()
                                == worth_foundational::facade::DeterminismContract::CanonicalBitwise
                        })
                    {
                        selected.retained_idempotency_key =
                            Some(candidate.idempotency_key_identity);
                        selected.retained_output_binding = Some(candidate.binding);
                        return Ok((selected, entry));
                    }
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
