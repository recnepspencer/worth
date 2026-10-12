use super::super::support::*;

pub(super) mod runtime_paths;
mod variant_keys;

use runtime_paths::{
    intent_commit_denied_error, intent_execution_routing_failed_error,
    preview_promotion_atomic_batch_unsupported_error, preview_promotion_rebinding_required_error,
    preview_promotion_stale_basis_error, preview_promotion_write_failed_error,
};
pub(super) use variant_keys::{runtime_error_variant_key, stop_class_variant_key};

pub(super) fn representative_runtime_stop_errors(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Vec<WorthQueryRuntimeError> {
    let binding = existing_binding();
    let status_touch = status_value_touch();
    let assertion_denial = WorthQueryExistingTruthAssertionDenial::new(
        &binding,
        WorthQueryExistingTruthAssertionDenialKind::MissingAssertedAspect,
        Some(status_touch.clone()),
        Some("\"open\"".to_string()),
        None,
        "missing asserted aspect",
    );
    let probe_denial = WorthQueryExistingTruthProbeDenial::new(
        &binding,
        WorthQueryExistingTruthProbeDenialKind::MissingProbedAspect,
        Some(status_touch),
        "missing probed aspect",
    );
    let binding_denial = WorthQueryExistingTruthBindingDenial::new(
        &binding,
        WorthQueryExistingTruthBindingDenialKind::CollectionMismatch,
        "wrong collection",
    );
    let continuity_intent = WorthQueryContinuityMutationIntent::rebind_existing_target(
        crate::runtime::WorthQueryMutationAuthorityIdentity::continuity_prior_authority(
            crate::runtime::WorthQueryContinuityPriorAuthorityLabel::new("authority:task-1")
                .expect("continuity prior authority label"),
        )
        .expect("continuity prior authority identity"),
        crate::runtime::WorthQueryMutationAuthorityIdentity::continuity_successor_authority(
            crate::runtime::WorthQueryContinuitySuccessorAuthorityLabel::new("authority:task-1")
                .expect("continuity successor authority label"),
        )
        .expect("continuity successor authority identity"),
    )
    .expect("continuity intent should build");
    let continuity_denial = WorthQueryContinuityMutationDenial::new(
        &continuity_intent,
        Some(&binding),
        WorthQueryContinuityMutationDenialKind::RequiresExistingTruthBinding,
        "continuity requires binding",
    );
    let graph_denial = WorthQueryGraphCompositionDenial::new(
        WorthQueryGraphCompositionDenialKind::DuplicateSymbolDeclaration,
        Some("task_symbol".to_string()),
        Some(
            crate::runtime::WorthQueryMutationTargetCollectionIdentity::new(
                "graph-composition-test",
                "Task",
            ),
        ),
        "duplicate symbol",
    );
    let naming_intent = WorthQueryNamingMutationIntent::attach_new_target(
        crate::runtime::WorthQueryMutationAuthorityIdentity::naming_attachment(
            crate::runtime::WorthQueryNamingAttachmentAuthorityLabel::new("attachment-1")
                .expect("naming attachment authority label"),
        )
        .expect("naming attachment identity"),
    );
    let naming_denial = WorthQueryNamingMutationDenial::new(
        &naming_intent,
        WorthQueryNamingMutationDenialKind::RequiresSameBatchTargetReference,
        "naming needs a same-batch target",
    );
    let target_reference =
        WorthQuerySymbolicTargetReference::new("task_symbol").expect("reference should build");
    let symbolic_denial = WorthQuerySymbolicTargetReferenceDenial::new(
        &target_reference,
        WorthQuerySymbolicTargetReferenceDenialKind::UnresolvedSameBatchTarget,
        "same-batch target unresolved",
    );
    let read_denial = WorthQueryReadDenial::new(
        WorthQueryReadDenialKind::ValidationDenied,
        "read validation failed",
    );
    let effect_policy_denial = WorthQueryEffectPolicy::DeriveOnly
        .admit(
            WorthQueryEffectAction::WriteIntent,
            WorthQueryAuthorityLane::AuthoritativeTruth,
        )
        .expect_err("derive-only write intent should deny");
    let support_denial = WorthQueryRuntimeSupportDenial::new(
        WorthQueryRuntimeFacadeFamily::Temporal,
        WorthQueryRuntimeFamilySupportStatus::Unsupported,
        Some(WorthQueryRuntimeFamilyTeachingPosture::SupportGateOnly),
        "temporal gate is closed",
    );
    let replay_denial = WorthQueryJournalReplayDenial::new(
        WorthQueryJournalReplayDenialKind::UnknownSegmentIdentity,
        "segment is not retained by replay registry",
    );
    let domain_handle_denial = crate::domain_installation::WorthQueryDomainHandleDenial::new(
        crate::domain_installation::WorthQueryDomainHandleDenialKind::DomainNotInstalled,
    );
    let mutation_contract_touch = test_aspect_touch("status.value");
    let mutation_contract_denial = WorthQueryMutationContractDenial::portable_export_denied(
        worth_foundational::facade::PortableAspectExportDenial::MissingContract(
            mutation_contract_touch.native_aspect_key().clone(),
        ),
    );

    vec![
        WorthQueryRuntimeError::InstalledDomainAuthorityDenied(domain_handle_denial),
        WorthQueryRuntimeError::MissingBackend,
        WorthQueryRuntimeError::MissingRuntimeBridge,
        WorthQueryRuntimeError::MissingSchemaAdapter,
        WorthQueryRuntimeError::MissingSnapshotIdentityAdapter,
        WorthQueryRuntimeError::MissingSourceAdapter,
        WorthQueryRuntimeError::MissingWriteAuthority,
        WorthQueryRuntimeError::MissingSignalSink,
        WorthQueryRuntimeError::MissingSubscriptionActivation,
        WorthQueryRuntimeError::MissingPreviewBasis,
        WorthQueryRuntimeError::MissingInspectorEvidence,
        WorthQueryRuntimeError::MissingIntentAuthority,
        WorthQueryRuntimeError::ExistingTruthAssertionDenied(assertion_denial),
        WorthQueryRuntimeError::ExistingTruthProbeDenied(probe_denial),
        WorthQueryRuntimeError::MutationBindingDenied(binding_denial),
        WorthQueryRuntimeError::MutationContinuityDenied(continuity_denial),
        WorthQueryRuntimeError::MutationContractDenied(mutation_contract_denial),
        WorthQueryRuntimeError::GraphCompositionDenied(graph_denial),
        WorthQueryRuntimeError::MutationNamingDenied(naming_denial),
        WorthQueryRuntimeError::MutationTargetReferenceDenied(symbolic_denial),
        WorthQueryRuntimeError::ReadCompositionDenied(read_denial),
        WorthQueryRuntimeError::Workspace(WorthQueryWorkspaceError::new("workspace failed")),
        WorthQueryRuntimeError::Program(WorthQueryProgramError::new("program failed")),
        WorthQueryRuntimeError::UnknownProgram("missing.program".to_string()),
        WorthQueryRuntimeError::UnknownOperation {
            program_id: "program.id".to_string(),
            operation_id: "operation.id".to_string(),
        },
        WorthQueryRuntimeError::MissingLiveView("view.live".to_string()),
        WorthQueryRuntimeError::MissingLiveSubscription("sub.live".to_string()),
        WorthQueryRuntimeError::MissingDerivedView("view.derived".to_string()),
        WorthQueryRuntimeError::SharedReadStaleBasis {
            snapshot_identity: crate::memory_workspace::admit_external_snapshot_label(
                "snapshot.stale",
            ),
        },
        WorthQueryRuntimeError::JournalReplayDenied(replay_denial),
        WorthQueryRuntimeError::MissingEffect("effect.name".to_string()),
        WorthQueryRuntimeError::MissingPendingWriteIntent("effect.name".to_string()),
        WorthQueryRuntimeError::RetainedRowDecode {
            view_name: "view.retained".to_string(),
            stage: "decode",
            message: "decode failed".to_string(),
        },
        WorthQueryRuntimeError::ComputedDeclaration {
            view_name: "view.computed".to_string(),
            stage: "declare",
            message: "computed failed".to_string(),
        },
        WorthQueryRuntimeError::EffectDeclaration {
            effect_name: "effect.declare".to_string(),
            stage: "declare",
            message: "effect failed".to_string(),
        },
        WorthQueryRuntimeError::LiveSubscriptionInstallation {
            view_name: "view.live".to_string(),
            stage: "install",
            message: "install failed".to_string(),
        },
        WorthQueryRuntimeError::UnsupportedAuthorityRequirement(
            WorthQueryAuthorityRequirement::Merge,
        ),
        WorthQueryRuntimeError::ExistingTruthAssertionRequiresAuthorityLane {
            required_lane: WorthQueryAuthorityLane::AuthoritativeTruth,
        },
        intent_commit_denied_error(request_execution),
        intent_execution_routing_failed_error(request_execution),
        WorthQueryRuntimeError::EffectPolicyDenied(effect_policy_denial),
        preview_promotion_stale_basis_error(request_execution),
        preview_promotion_atomic_batch_unsupported_error(request_execution),
        preview_promotion_rebinding_required_error(request_execution),
        preview_promotion_write_failed_error(request_execution),
        WorthQueryRuntimeError::InvariantRegistration {
            stage: "registration",
            message: "registration failed".to_string(),
        },
        WorthQueryRuntimeError::SessionLabelCollision {
            authority_lane: WorthQueryAuthorityLane::PreviewTruth,
            label: test_session_label("stop-class-collision"),
        },
        WorthQueryRuntimeError::PreviewOperationEffectDenied {
            label: test_session_label("preview-label"),
            stage: "effect-admission",
            message: "preview declaration denied".to_string(),
        },
        WorthQueryRuntimeError::UnsupportedFacadeFamily(support_denial),
    ]
}

pub(super) fn representative_runtime_generated_stop_errors(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> Vec<WorthQueryRuntimeError> {
    vec![
        intent_commit_denied_error(request_execution),
        intent_execution_routing_failed_error(request_execution),
        preview_promotion_stale_basis_error(request_execution),
        preview_promotion_atomic_batch_unsupported_error(request_execution),
        preview_promotion_rebinding_required_error(request_execution),
        preview_promotion_write_failed_error(request_execution),
    ]
}

pub(super) fn existing_binding() -> WorthQueryExistingTruthTargetBinding {
    WorthQueryExistingTruthTargetBinding::direct_entity(
        crate::runtime::WorthQueryMutationAuthorityIdentity::existing_truth_binding_authority(
            crate::runtime::WorthQueryExistingTruthBindingAuthorityLabel::new("authority:task-1")
                .expect("existing-truth authority label"),
        )
        .expect("existing-truth authority identity"),
        test_entity_identity("Task:1"),
    )
    .expect("binding should build")
    .in_target_collection("Task")
    .expect("collection should build")
}

pub(super) fn status_value_touch() -> WorthQueryAspectTouch {
    test_aspect_touch("status.value")
}
