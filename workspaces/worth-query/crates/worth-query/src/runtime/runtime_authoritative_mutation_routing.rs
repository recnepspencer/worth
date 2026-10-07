use super::*;
use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::memory_workspace::WorthQueryEntityIdentity;

pub(super) struct WorthQueryAuthoritativeMutationRoutingInput {
    pub(super) receipt: WorthQueryMutationReceipt,
    pub(super) declaration: WorthQueryAuthoritativeMutationDeclaration,
    pub(super) target_evidence: WorthQueryAuthoritativeMutationTargetEvidence,
    pub(super) execution_evidence: WorthQueryAuthoritativeMutationExecutionEvidence,
}

pub(super) struct WorthQueryAuthoritativeMutationDeclaration {
    pub(super) mutation_family: WorthQueryMutationFamily,
    pub(super) declared_collection_identity: Option<WorthQueryMutationTargetCollectionIdentity>,
    pub(super) declared_entity_identity: Option<WorthQueryEntityIdentity>,
    pub(super) declared_aspect_operations: Vec<crate::runtime::WorthQueryAspectMutationOperation>,
    pub(super) declared_aspect_value_digest: Option<WorthQueryEvidenceIdentity>,
    pub(super) mutation_metadata: WorthQueryMutationMetadata,
}

pub(super) struct WorthQueryAuthoritativeMutationTargetEvidence {
    pub(super) existing_truth_binding: Option<WorthQueryExistingTruthTargetBinding>,
    pub(super) existing_truth_assertion: Option<WorthQueryVerifiedExistingTruthAssertion>,
    pub(super) symbolic_target_reference: Option<WorthQuerySymbolicTargetReference>,
    pub(super) symbolic_aspect_resolution_evidence: Vec<WorthQuerySymbolicAspectResolutionEvidence>,
    pub(super) naming_intent: Option<WorthQueryNamingMutationIntent>,
    pub(super) continuity_intent: Option<WorthQueryContinuityMutationIntent>,
}

pub(super) struct WorthQueryAuthoritativeMutationExecutionEvidence {
    pub(super) decision_trace_envelope: Option<WorthQueryIntentDecisionTraceEnvelope>,
    pub(super) execution_provenance: Option<WorthQueryIntentExecutionProvenance>,
}

pub(super) struct WorthQueryPreparedAuthoritativeMutationRouting {
    declaration: WorthQueryAuthoritativeMutationDeclaration,
    target_evidence: WorthQueryAuthoritativeMutationTargetEvidence,
}

impl WorthQueryPreparedAuthoritativeMutationRouting {
    pub(super) fn from_direct_command(
        command: &WorthQueryWriteCommand,
        existing_truth_assertion: Option<WorthQueryVerifiedExistingTruthAssertion>,
    ) -> Self {
        Self {
            declaration: WorthQueryAuthoritativeMutationDeclaration {
                mutation_family: command.mutation_family(),
                declared_collection_identity: command.declared_collection_identity(),
                declared_entity_identity: command.declared_entity_identity(),
                declared_aspect_operations: command.declared_aspect_operations(),
                declared_aspect_value_digest: command_declared_aspect_value_identity(command),
                mutation_metadata: command.mutation_metadata(),
            },
            target_evidence: WorthQueryAuthoritativeMutationTargetEvidence {
                existing_truth_binding: command.existing_truth_binding().cloned(),
                existing_truth_assertion,
                symbolic_target_reference: None,
                symbolic_aspect_resolution_evidence: Vec::new(),
                naming_intent: command.naming_intent().cloned(),
                continuity_intent: command.continuity_intent().cloned(),
            },
        }
    }

    pub(super) fn mutation_family(&self) -> WorthQueryMutationFamily {
        self.declaration.mutation_family
    }

    pub(super) fn declared_aspect_value_digest(&self) -> Option<&WorthQueryEvidenceIdentity> {
        self.declaration.declared_aspect_value_digest.as_ref()
    }

    pub(super) fn existing_truth_binding(&self) -> Option<&WorthQueryExistingTruthTargetBinding> {
        self.target_evidence.existing_truth_binding.as_ref()
    }

    pub(super) fn naming_intent(&self) -> Option<&WorthQueryNamingMutationIntent> {
        self.target_evidence.naming_intent.as_ref()
    }

    pub(super) fn continuity_intent(&self) -> Option<&WorthQueryContinuityMutationIntent> {
        self.target_evidence.continuity_intent.as_ref()
    }

    pub(super) fn complete(
        self,
        receipt: WorthQueryMutationReceipt,
        execution_evidence: WorthQueryAuthoritativeMutationExecutionEvidence,
    ) -> WorthQueryAuthoritativeMutationRoutingInput {
        WorthQueryAuthoritativeMutationRoutingInput {
            receipt,
            declaration: self.declaration,
            target_evidence: self.target_evidence,
            execution_evidence,
        }
    }
}

impl WorthQueryAuthoritativeMutationRoutingInput {
    pub(super) fn from_intent_execution(
        receipt: WorthQueryMutationReceipt,
        summary: (
            WorthQueryMutationFamily,
            Option<WorthQueryMutationTargetCollectionIdentity>,
            Option<WorthQueryEntityIdentity>,
        ),
        decision_trace_envelope: WorthQueryIntentDecisionTraceEnvelope,
        execution_provenance: WorthQueryIntentExecutionProvenance,
    ) -> Self {
        Self {
            receipt,
            declaration: WorthQueryAuthoritativeMutationDeclaration {
                mutation_family: summary.0,
                declared_collection_identity: summary.1,
                declared_entity_identity: summary.2,
                declared_aspect_operations: Vec::new(),
                declared_aspect_value_digest: None,
                mutation_metadata: WorthQueryMutationMetadata::default(),
            },
            target_evidence: WorthQueryAuthoritativeMutationTargetEvidence {
                existing_truth_binding: None,
                existing_truth_assertion: None,
                symbolic_target_reference: None,
                symbolic_aspect_resolution_evidence: Vec::new(),
                naming_intent: None,
                continuity_intent: None,
            },
            execution_evidence: WorthQueryAuthoritativeMutationExecutionEvidence {
                decision_trace_envelope: Some(decision_trace_envelope),
                execution_provenance: Some(execution_provenance),
            },
        }
    }
}

impl WorthQueryRuntime {
    pub(super) fn route_authoritative_mutation_receipt(
        &mut self,
        input: WorthQueryAuthoritativeMutationRoutingInput,
    ) -> Result<WorthQueryWriteReceipt, WorthQueryRuntimeError> {
        let WorthQueryAuthoritativeMutationRoutingInput {
            receipt,
            declaration,
            target_evidence,
            execution_evidence,
        } = input;
        let WorthQueryAuthoritativeMutationDeclaration {
            mutation_family,
            declared_collection_identity,
            declared_entity_identity,
            declared_aspect_operations,
            declared_aspect_value_digest,
            mutation_metadata,
        } = declaration;
        let WorthQueryAuthoritativeMutationTargetEvidence {
            existing_truth_binding,
            existing_truth_assertion,
            symbolic_target_reference,
            symbolic_aspect_resolution_evidence,
            naming_intent,
            continuity_intent,
        } = target_evidence;
        let WorthQueryAuthoritativeMutationExecutionEvidence {
            decision_trace_envelope,
            execution_provenance,
        } = execution_evidence;
        let (_, target_collection, mut target_entity_identity) =
            classify_receipt_mutation_summary(&receipt);
        let mut target_collection_identity = target_collection;
        if let Some(binding) = existing_truth_binding.as_ref() {
            target_collection_identity = binding.target_collection_identity().cloned();
            target_entity_identity = Some(binding.resolved_entity_artifact_identity());
        }
        let summary = self.route_authoritative_mutation_summary(&receipt, &mutation_metadata)?;
        self.capture_shared_read_generation(receipt.snapshot_identity.clone());
        Ok(WorthQueryWriteReceipt::from_mutation_receipt(
            receipt,
            mutation_family,
            declared_collection_identity,
            declared_entity_identity,
            existing_truth_binding,
            existing_truth_assertion,
            symbolic_target_reference,
            symbolic_aspect_resolution_evidence,
            naming_intent,
            continuity_intent,
            target_collection_identity,
            target_entity_identity,
            declared_aspect_operations,
            declared_aspect_value_digest,
            mutation_metadata,
            summary.affected_live_view_targets,
            summary.affected_derived_view_targets,
            summary.considered_computed_view_count,
            summary.considered_effect_count,
            summary.delivered_effect_count,
            summary.pending_write_intent_count,
            summary.suppressed_effect_count,
            summary.meaningful_effect_suppression_count,
            summary.effect_expression_failure_count,
            summary.refresh_fallback,
            decision_trace_envelope,
            execution_provenance,
        ))
    }

    pub(super) fn route_authoritative_mutation_summary(
        &mut self,
        receipt: &WorthQueryMutationReceipt,
        mutation_metadata: &WorthQueryMutationMetadata,
    ) -> Result<WorthQueryRoutedMutationSummary, WorthQueryRuntimeError> {
        let affected_live_view_targets = route_live_subscription_delivery(
            &mut self.active_subscriptions,
            &mut self.live_subscriptions,
            &self.live_subscription_index,
            &self.installed_live_routes,
            receipt,
        )?;
        let computed_candidate_live_views = self.computed_candidate_live_views(receipt);
        let retained_live_view_targets = retained_live_view_names_for_candidates(
            &self.derived_views,
            &self.derived_dependency_index,
            computed_candidate_live_views.iter().cloned(),
        );
        let retained_live_rows = retained_live_view_targets
            .into_iter()
            .map(|target| {
                let rows = self.backend.live_entities_for_target(&target)?;
                Ok((target, rows))
            })
            .collect::<Result<BTreeMap<_, _>, crate::memory_workspace::WorthQueryWorkspaceError>>(
            )?;
        let computed_result = route_derived_view_patches(
            &mut self.derived_views,
            &self.derived_dependency_index,
            computed_candidate_live_views,
            &retained_live_rows,
            receipt,
            mutation_metadata,
        );
        let refresh_fallback = computed_result.refresh_fallback();
        let considered_computed_view_count = computed_result.considered_view_count();
        let affected_derived_view_targets = computed_result.affected_view_targets();
        let live_artifact_target_collections = self.live_artifact_target_collections();
        let effect_result = route_effect_deliveries(
            &mut self.effects,
            &self.effect_index,
            &self.derived_views,
            &live_artifact_target_collections,
            receipt,
            &affected_live_view_targets,
            &affected_derived_view_targets,
        );
        Ok(WorthQueryRoutedMutationSummary {
            affected_live_view_targets,
            affected_derived_view_targets,
            considered_computed_view_count,
            considered_effect_count: effect_result.considered_effect_count(),
            delivered_effect_count: effect_result.delivered_effect_count(),
            pending_write_intent_count: effect_result.pending_write_intent_count(),
            suppressed_effect_count: effect_result.suppressed_effect_count(),
            meaningful_effect_suppression_count: effect_result.meaningful_suppression_count(),
            effect_expression_failure_count: effect_result.expression_failure_count(),
            refresh_fallback,
        })
    }
}
