use super::super::super::support::*;

pub(in crate::runtime::tests::stop_class) fn runtime_error_variant_key(
    error: &WorthQueryRuntimeError,
) -> &'static str {
    match error {
        WorthQueryRuntimeError::Advancement(_) => "advancement",
        WorthQueryRuntimeError::ForeignPhase(_) => "foreign_phase",
        WorthQueryRuntimeError::InstalledDomainAuthorityDenied(_) => {
            "installed_domain_authority_denied"
        }
        WorthQueryRuntimeError::MissingBackend => "missing_backend",
        WorthQueryRuntimeError::MissingRuntimeBridge => "missing_runtime_bridge",
        WorthQueryRuntimeError::MissingSchemaAdapter => "missing_schema_adapter",
        WorthQueryRuntimeError::MissingSnapshotIdentityAdapter => {
            "missing_snapshot_identity_adapter"
        }
        WorthQueryRuntimeError::MissingSourceAdapter => "missing_source_adapter",
        WorthQueryRuntimeError::MissingWriteAuthority => "missing_write_authority",
        WorthQueryRuntimeError::MissingSignalSink => "missing_signal_sink",
        WorthQueryRuntimeError::MissingSubscriptionActivation => "missing_subscription_activation",
        WorthQueryRuntimeError::MissingPreviewBasis => "missing_preview_basis",
        WorthQueryRuntimeError::MissingInspectorEvidence => "missing_inspector_evidence",
        WorthQueryRuntimeError::MissingIntentAuthority => "missing_intent_authority",
        WorthQueryRuntimeError::ExistingTruthAssertionDenied(_) => {
            "existing_truth_assertion_denied"
        }
        WorthQueryRuntimeError::ExistingTruthProbeDenied(_) => "existing_truth_probe_denied",
        WorthQueryRuntimeError::MutationBindingDenied(_) => "mutation_binding_denied",
        WorthQueryRuntimeError::MutationContinuityDenied(_) => "mutation_continuity_denied",
        WorthQueryRuntimeError::MutationContractDenied(_) => "mutation_contract_denied",
        WorthQueryRuntimeError::GraphCompositionDenied(_) => "graph_composition_denied",
        WorthQueryRuntimeError::MutationNamingDenied(_) => "mutation_naming_denied",
        WorthQueryRuntimeError::MutationTargetReferenceDenied(_) => {
            "mutation_target_reference_denied"
        }
        WorthQueryRuntimeError::ReadCompositionDenied(_) => "read_composition_denied",
        WorthQueryRuntimeError::Workspace(_) => "workspace",
        WorthQueryRuntimeError::Program(_) => "program",
        WorthQueryRuntimeError::UnknownProgram(_) => "unknown_program",
        WorthQueryRuntimeError::UnknownOperation { .. } => "unknown_operation",
        WorthQueryRuntimeError::MissingLiveView(_) => "missing_live_view",
        WorthQueryRuntimeError::MissingLiveSubscription(_) => "missing_live_subscription",
        WorthQueryRuntimeError::MissingDerivedView(_) => "missing_derived_view",
        WorthQueryRuntimeError::MissingEffect(_) => "missing_effect",
        WorthQueryRuntimeError::MissingPendingWriteIntent(_) => "missing_pending_write_intent",
        WorthQueryRuntimeError::RetainedRowDecode { .. } => "retained_row_decode",
        WorthQueryRuntimeError::ComputedDeclaration { .. } => "computed_declaration",
        WorthQueryRuntimeError::EffectDeclaration { .. } => "effect_declaration",
        WorthQueryRuntimeError::LiveSubscriptionInstallation { .. } => {
            "live_subscription_installation"
        }
        WorthQueryRuntimeError::UnsupportedAuthorityRequirement(_) => {
            "unsupported_authority_requirement"
        }
        WorthQueryRuntimeError::ExistingTruthAssertionRequiresAuthorityLane { .. } => {
            "existing_truth_assertion_requires_authority_lane"
        }
        WorthQueryRuntimeError::IntentCommitDenied { .. } => "intent_commit_denied",
        WorthQueryRuntimeError::IntentExecutionRoutingFailed { .. } => {
            "intent_execution_routing_failed"
        }
        WorthQueryRuntimeError::EffectPolicyDenied(_) => "effect_policy_denied",
        WorthQueryRuntimeError::PreviewPromotionStaleBasis(_) => "preview_promotion_stale_basis",
        WorthQueryRuntimeError::SharedReadStaleBasis { .. } => "shared_read_stale_basis",
        WorthQueryRuntimeError::JournalReplayDenied(_) => "journal_replay_denied",
        WorthQueryRuntimeError::PreviewPromotionAtomicBatchUnsupported(_) => {
            "preview_promotion_atomic_batch_unsupported"
        }
        WorthQueryRuntimeError::PreviewPromotionRebindingRequired(_) => {
            "preview_promotion_rebinding_required"
        }
        WorthQueryRuntimeError::PreviewPromotionWriteFailed { .. } => {
            "preview_promotion_write_failed"
        }
        WorthQueryRuntimeError::InvariantRegistration { .. } => "invariant_registration",
        WorthQueryRuntimeError::SessionLabelCollision { .. } => "session_label_collision",
        WorthQueryRuntimeError::PreviewOperationEffectDenied { .. } => {
            "preview_operation_effect_denied"
        }
        WorthQueryRuntimeError::UnsupportedFacadeFamily(_) => "unsupported_facade_family",
    }
}

pub(in crate::runtime::tests::stop_class) fn stop_class_variant_key(
    stop_class: WorthQueryStopClass<'_>,
) -> &'static str {
    match stop_class {
        WorthQueryStopClass::Advancement { .. } => "advancement",
        WorthQueryStopClass::ForeignPhase { .. } => "foreign_phase",
        WorthQueryStopClass::InstalledDomainAuthorityDenied { .. } => {
            "installed_domain_authority_denied"
        }
        WorthQueryStopClass::MissingRuntimeComponent { .. } => "missing_runtime_component",
        WorthQueryStopClass::ExistingTruthAssertionDenied { .. } => {
            "existing_truth_assertion_denied"
        }
        WorthQueryStopClass::ExistingTruthProbeDenied { .. } => "existing_truth_probe_denied",
        WorthQueryStopClass::MutationBindingDenied { .. } => "mutation_binding_denied",
        WorthQueryStopClass::MutationContinuityDenied { .. } => "mutation_continuity_denied",
        WorthQueryStopClass::MutationContractDenied { .. } => "mutation_contract_denied",
        WorthQueryStopClass::GraphCompositionDenied { .. } => "graph_composition_denied",
        WorthQueryStopClass::MutationNamingDenied { .. } => "mutation_naming_denied",
        WorthQueryStopClass::MutationTargetReferenceDenied { .. } => {
            "mutation_target_reference_denied"
        }
        WorthQueryStopClass::ReadCompositionDenied { .. } => "read_composition_denied",
        WorthQueryStopClass::Workspace { .. } => "workspace",
        WorthQueryStopClass::Program { .. } => "program",
        WorthQueryStopClass::RuntimeLookupFailed { .. } => "runtime_lookup_failed",
        WorthQueryStopClass::MissingRuntimeArtifact { .. } => "missing_runtime_artifact",
        WorthQueryStopClass::SharedReadStaleBasis { .. } => "shared_read_stale_basis",
        WorthQueryStopClass::JournalReplayDenied { .. } => "journal_replay_denied",
        WorthQueryStopClass::RuntimeDeclarationFailed { .. } => "runtime_declaration_failed",
        WorthQueryStopClass::PreviewOperationEffectDenied { .. } => {
            "preview_operation_effect_denied"
        }
        WorthQueryStopClass::SessionLabelCollision { .. } => "session_label_collision",
        WorthQueryStopClass::UnsupportedAuthorityRequirement { .. } => {
            "unsupported_authority_requirement"
        }
        WorthQueryStopClass::ExistingTruthAssertionRequiresAuthorityLane { .. } => {
            "existing_truth_assertion_requires_authority_lane"
        }
        WorthQueryStopClass::IntentCommitDenied { .. } => "intent_commit_denied",
        WorthQueryStopClass::IntentExecutionRoutingFailed { .. } => {
            "intent_execution_routing_failed"
        }
        WorthQueryStopClass::EffectPolicyDenied { .. } => "effect_policy_denied",
        WorthQueryStopClass::PreviewPromotionDenied { .. } => "preview_promotion_denied",
        WorthQueryStopClass::FamilyAdmissionDenied { .. } => "family_admission_denied",
    }
}
