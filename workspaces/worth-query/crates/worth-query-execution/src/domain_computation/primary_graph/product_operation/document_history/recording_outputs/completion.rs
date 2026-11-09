//! The canonical collection owner seals one endpoint; the journal adopts it atomically.
use super::*;
impl<S: ApplicationSchema>
    crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<S>
{
    pub fn qualify_recording_output_forest(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        request: &WorthQueryRequestScope,
        origin: &Arc<WorthQueryHistoryRecordingOutputOrigin>,
        trees: &[Arc<WorthQueryRetainedRecordingOutputTree>],
    ) -> Result<WorthQueryCompletedRecordingOutputs, WorthQueryRecordingOutputQualificationDenial>
    {
        let execution = phase
            .checked_execution_request_for(&self.runtime.product_runtime)
            .map_err(stopped)?;
        if trees.len() != origin.profile.tree_profiles().len() {
            return Err(foreign().into());
        }
        for (index, tree) in trees.iter().enumerate() {
            checkpoint(&execution)?;
            if tree.index != index || !Arc::ptr_eq(&tree.origin, origin) {
                return Err(foreign().into());
            }
        }
        let settlements = trees.iter().flat_map(|tree| tree.settlements());
        let endpoint=crate::domain_computation::primary_graph::application_output_demand::original_endpoint::derive(
            &execution,&origin.observation,&mut settlements.clone(),&mut settlements.clone()).map_err(WorthQueryOriginalReadOutputDenial::Output)?;
        let selected = self
            .select_application_read_observation(phase, &endpoint)
            .map_err(WorthQueryOriginalReadOutputDenial::Observation)?;
        for tree in trees {
            checkpoint(&execution)?;
            match &tree.completion {
                RecordingTreeCompletion::Original(original) => {
                    original.require_current_at(
                        self.select_application_read_observation(phase, &endpoint)
                            .map_err(WorthQueryOriginalReadOutputDenial::Observation)?,
                        phase,
                        access,
                        request,
                    )?;
                }
                RecordingTreeCompletion::VerifiedParent(verified) => {
                    verified
                        .require_current_at(
                            self.select_application_read_observation(phase, &endpoint)
                                .map_err(WorthQueryOriginalReadOutputDenial::Observation)?,
                            phase,
                            access,
                            request,
                        )
                        .map_err(WorthQueryRecordingOutputQualificationDenial::Verified)?;
                    let profile = &origin.profile.tree_profiles()[tree.index];
                    let current = graph_current_parent(self, &selected, &execution, profile)?;
                    if !current {
                        return Err(WorthQueryRecordingOutputQualificationDenial::TreeChanged {
                            source: profile.source_intent().source_subject(),
                            family: profile.recording_family(),
                        });
                    }
                }
                RecordingTreeCompletion::VerifiedLeaf => {
                    if origin.verify_tree_at(self, phase, tree.index, &endpoint)?
                        != verification::TreeVerification::CurrentLeaf
                    {
                        return Err(WorthQueryRecordingOutputQualificationDenial::TreeChanged {
                            source: origin.profile.tree_profiles()[tree.index]
                                .source_intent()
                                .source_subject(),
                            family: origin.profile.tree_profiles()[tree.index].recording_family(),
                        });
                    }
                }
                RecordingTreeCompletion::SourceOutsideDomain => {
                    if origin.verify_tree_at(self, phase, tree.index, &endpoint)?
                        != verification::TreeVerification::OutsideDomain
                    {
                        return Err(WorthQueryRecordingOutputQualificationDenial::TreeChanged {
                            source: origin.profile.tree_profiles()[tree.index]
                                .source_intent()
                                .source_subject(),
                            family: origin.profile.tree_profiles()[tree.index].recording_family(),
                        });
                    }
                }
                RecordingTreeCompletion::SourceNotLive => {
                    if origin.verify_tree_at(self, phase, tree.index, &endpoint)?
                        != verification::TreeVerification::SourceNotLive
                    {
                        return Err(WorthQueryRecordingOutputQualificationDenial::TreeChanged {
                            source: origin.profile.tree_profiles()[tree.index]
                                .source_intent()
                                .source_subject(),
                            family: origin.profile.tree_profiles()[tree.index].recording_family(),
                        });
                    }
                }
            }
        }
        let graph = &self.runtime.primary_provider.graph;
        if let Some(publisher) = graph
            .application_publication
            .as_ref()
            .filter(|publisher| publisher.native.owns_source_scopes())
        {
            graph.with_runtime(&execution, |native| {
                let observed = native
                    .observe_owned_model_publications(
                        &execution,
                        &publisher.native,
                        selected.product().relational_basis(),
                        graph.layout.installed_history_ownership(),
                        origin.profile.recording_model().entity(),
                    )
                    .map_err(|cause| {
                        WorthQueryRecordingOutputQualificationDenial::Ownership(Box::new(cause))
                    })?;
                for route in observed.routes() {
                    checkpoint(&execution)?;
                    use worth_relational::facade::branch::RelationalOwnedModelOutputRoute as Route;
                    let current = match route {
                        Route::Published(route) => selected
                            .current_route_is_current(&execution, native, &route)
                            .map_err(WorthQueryOriginalReadOutputDenial::Output)?,
                        Route::CodecRetired { .. } => false,
                    };
                    if !current {
                        // No source READ was admitted for an omitted family.
                        // Report model completeness without disclosing its source.
                        return Err(WorthQueryRecordingOutputQualificationDenial::Incomplete {
                            model: origin.profile.recording_model().entity(),
                        });
                    }
                }
                Ok(())
            })?;
        }
        let bytes = trees
            .len()
            .checked_mul(std::mem::size_of::<
                Arc<WorthQueryRetainedRecordingOutputTree>,
            >())
            .and_then(|bytes| {
                bytes.checked_add(std::mem::size_of::<WorthQueryCompletedRecordingOutputs>())
            })
            .ok_or_else(|| resource(Cause::ChargedBytesOverflow))?;
        let memory = memory::reserve(&execution, execution.has_accounting(), bytes as u64)
            .map_err(resource)?;
        let mut retained = Vec::with_capacity(trees.len());
        for tree in trees {
            checkpoint(&execution)?;
            retained.push(Arc::clone(tree));
        }
        Ok(WorthQueryCompletedRecordingOutputs {
            discovery_recipes: origin.recipes.seal(&execution).map_err(resource)?,
            origin: Arc::clone(origin),
            endpoint,
            _trees: retained,
            _memory: memory,
        })
    }
}
impl WorthQueryCompletedRecordingOutputs {
    pub fn observation(&self) -> &Arc<WorthQueryApplicationReadObservation> {
        &self.endpoint
    }
    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn require_source(
        &self,
        reserved: &WorthQueryDocumentHistoryReservation,
        candidate: &WorthQueryExpectedProductBranch<'_, impl ApplicationSchema>,
        scope: &super::super::HistoryOperationScope,
        source: super::super::source_evidence::WorthQueryHistorySourceEvidence<'_>,
        required: Option<&WorthQueryCompletedRequiredOutputs>,
    ) -> Result<(), WorthQueryDocumentHistoryDenial> {
        let origin = &self.origin;
        if origin.node != reserved.record.node
            || origin.expected != reserved.expected
            || origin.candidate != candidate.branch()
            || !source.matches(scope, origin.intent)
            || !match source {
                super::super::source_evidence::WorthQueryHistorySourceEvidence::Live(receipt) => {
                    origin.intent == receipt.idempotency_binding()
                        && &origin.source
                            == receipt.committed_product_publication().composite_commit()
                }
                super::super::source_evidence::WorthQueryHistorySourceEvidence::Recorded(_) => {
                    source.is_selected_at(origin.observation.product_observation())
                }
            }
            || self.endpoint.branch_incarnation() != candidate.branch().occurrence()
        {
            return Err(WorthQueryDocumentHistoryDenial::ReceiptMismatch);
        }
        match (origin.root_posture, required) {
            (None, None) => {}
            (Some(posture), Some(required))
                if posture == required.root_posture() && match source {
                    super::super::source_evidence::WorthQueryHistorySourceEvidence::Live(
                        receipt,
                    ) => required.matches_source(receipt),
                    super::super::source_evidence::WorthQueryHistorySourceEvidence::Recorded(_) => {
                        false
                    }
                }
                    && required.retained_output().product_observation()
                        == self.endpoint.product_observation() => {}
            _ => return Err(WorthQueryDocumentHistoryDenial::ReceiptMismatch),
        }
        Ok(())
    }
    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn endpoint(
        &self,
    ) -> &Arc<WorthQueryApplicationReadObservation> {
        &self.endpoint
    }
    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn root_posture(
        &self,
    ) -> Option<WorthQueryOutputSettlementPosture> {
        self.origin.root_posture
    }
}

fn graph_current_parent<S: ApplicationSchema>(
    runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<S>,
    selected: &crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<'_, S>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    profile: &WorthQueryHistoryOutputProfile,
) -> Result<bool, WorthQueryRecordingOutputQualificationDenial> {
    runtime.runtime.primary_provider.graph.with_runtime(execution,|native|
        selected.current_owned_route_is_current(execution,native,profile.recording_model().entity(),profile.source_intent().source_subject(),profile.recording_family(),profile.source_intent().source_partition())
            .map_err(|cause|match cause {
                crate::domain_computation::primary_graph::application_output_demand::CurrentOwnedRouteDenial::Ownership(cause)=>WorthQueryRecordingOutputQualificationDenial::Ownership(Box::new(cause)),
                crate::domain_computation::primary_graph::application_output_demand::CurrentOwnedRouteDenial::Output(cause)=>WorthQueryOriginalReadOutputDenial::Output(cause).into(),
            }))
}
