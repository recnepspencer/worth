use std::collections::BTreeMap;

use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::facade::foundation::{
    DeclarativeLiveQueryRequest, WorthQueryLiveViewHandle, WorthQueryMutationDelta,
    WorthQueryMutationKind, WorthQueryMutationReceipt, WorthQueryWorkspaceError,
};
use crate::facade::runtime::{
    LiveViewDeclarationAdmissionBoundaryReceipt, QuerySchemaView,
    SignalInvalidationBoundaryReceipt, SubscriptionActivationBoundaryReceipt,
    SubscriptionActivationInput, WorthQueryAuthoredAspectMutation, WorthQueryAuthorityLane,
    WorthQueryBackendAdmissibleMutation, WorthQueryBasisAdmissionEvidenceRow,
    WorthQueryEffectPolicy, WorthQueryIntentAuthorityAdapter, WorthQueryIntentDeclaration,
    WorthQueryIntentExecution, WorthQueryLiveArtifactTarget, WorthQueryPreviewBasisAdmission,
    WorthQueryRuntime, WorthQueryRuntimeEvidenceAuthority, WorthQueryRuntimeFacadeFamily,
    WorthQueryRuntimeFamilySupport, WorthQueryRuntimeInspectionEvidence,
    WorthQueryRuntimeInspectorEvidenceAdapter, WorthQueryRuntimePreviewBasisAdapter,
    WorthQueryRuntimeSchemaAdapter, WorthQueryRuntimeSignalSinkAdapter,
    WorthQueryRuntimeSnapshotIdentityAdapter, WorthQueryRuntimeSourceAdapter,
    WorthQueryRuntimeSubscriptionActivationAdapter, WorthQueryRuntimeSupportProfile,
    WorthQueryWriteCommand, WorthQueryWriteReceipt,
};
use crate::identity::hash_parts;
use crate::memory_workspace::{WorthQueryCommitIdentity, WorthQuerySnapshotIdentity};
use crate::memory_workspace::{WorthQueryEntity, WorthQueryLivePatch};
use crate::runtime::{
    runtime_subscription_support_evidence_identity, WorthQueryMutationTargetCollectionIdentity,
};
use crate::WorthQuerySessionLabel;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_runtime_bridge::facade::{RelationalBridgeSnapshotIdentityParts, RuntimeBridge};

mod transcript_aspect_contracts;
mod transcript_authority;
mod transcript_bridge;

use transcript_aspect_contracts::transcript_aspect_contracts;
use transcript_authority::TranscriptWriteAuthority;
use transcript_bridge::transcript_bridge;

pub(super) fn transcript_runtime(produced_aspects: &[&str]) -> WorthQueryRuntime {
    let runtime = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
        .runtime_name("worth-query-transcript-product")
        .build();
    let source = worth_query_execution::facade::integration::WorthQueryRelationalSourceOwner::new(
        runtime,
        "transcript-product",
    )
    .expect("transcript Product source should admit");
    let bridge = transcript_bridge(source.bridge_source());
    WorthQueryRuntime::builder(
        crate::consumer_kit::test_backend::in_memory_test_product_world_resources(),
    )
    .aspect_contracts(transcript_aspect_contracts(produced_aspects))
    .expect("transcript aspect contracts should install")
    .runtime_bridge(bridge)
    .relational_source_owner(source)
    .conditional_execution_resources(
        crate::runtime::WorthQueryConditionalExecutionResources::development(),
    )
    .schema_adapter(TranscriptSchemaAdapter)
    .source_adapter(TranscriptSourceAdapter::default())
    .snapshot_identity(TranscriptSnapshotIdentity)
    .write_authority(TranscriptWriteAuthority)
    .signal_sink(TranscriptSignalSink)
    .subscription_activation(TranscriptSubscriptionActivation)
    .preview_basis(TranscriptPreviewBasis)
    .inspector_evidence(TranscriptInspectorEvidence)
    .intent_authority(TranscriptIntentAuthority)
    .support_profile(intent_support_profile())
    .build_backend_from_parts()
    .build()
    .expect("transcript runtime backend parts should build")
}

fn intent_support_profile() -> WorthQueryRuntimeSupportProfile {
    WorthQueryRuntimeSupportProfile::bridge_backed(
        "transcript-subscription-activation",
        "transcript-preview-basis",
        "transcript-inspector-evidence",
    )
    .with_family_support(WorthQueryRuntimeFamilySupport::supported(
        WorthQueryRuntimeFacadeFamily::Intent,
        [
            WorthQueryAuthorityLane::AuthoritativeTruth,
            WorthQueryAuthorityLane::BranchLocalTruth,
            WorthQueryAuthorityLane::PreviewTruth,
        ],
        [],
        ["transcript-intent-authority"],
    ))
}

struct TranscriptSchemaAdapter;

struct TranscriptSnapshotIdentity;

impl WorthQueryRuntimeSnapshotIdentityAdapter for TranscriptSnapshotIdentity {
    fn current_snapshot_identity(&self) -> WorthQuerySnapshotIdentity {
        WorthQuerySnapshotIdentity::preview(
            WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::RuntimeStateSnapshot)
                .field_shape(
                    WorthQueryEvidenceTag::new("transcript_snapshot_authority"),
                    "runtime-api-stabilization",
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("transcript_snapshot_sequence"),
                    1,
                )
                .seal(),
        )
    }
}

impl WorthQueryRuntimeSchemaAdapter for TranscriptSchemaAdapter {
    fn admit_live_view(
        &self,
        name: &str,
        request: &DeclarativeLiveQueryRequest,
        _schema_view: &QuerySchemaView,
    ) -> Result<LiveViewDeclarationAdmissionBoundaryReceipt, WorthQueryWorkspaceError> {
        let receipt = self.build_live_view_declaration_admission_receipt(name, request);
        Ok(self.build_live_view_declaration_boundary_receipt(name, request, receipt))
    }
}

#[derive(Default)]
struct TranscriptSourceAdapter {
    live_views: BTreeMap<WorthQueryLiveArtifactTarget, WorthQueryMutationTargetCollectionIdentity>,
}

impl WorthQueryRuntimeSourceAdapter for TranscriptSourceAdapter {
    fn declare_live_view(
        &mut self,
        name: String,
        request: DeclarativeLiveQueryRequest,
        _schema_view: QuerySchemaView,
    ) -> Result<WorthQueryLiveViewHandle, WorthQueryWorkspaceError> {
        let live_target = WorthQueryLiveArtifactTarget::from_view_name(name.clone());
        self.live_views
            .insert(live_target, request.target_collection_identity());
        Ok(WorthQueryLiveViewHandle::new(name))
    }

    fn close_live_view(&mut self, name: &str) -> Result<(), WorthQueryWorkspaceError> {
        self.live_views
            .remove(&WorthQueryLiveArtifactTarget::from_view_name(name));
        Ok(())
    }

    fn live_entities_for_target(
        &self,
        _target: &WorthQueryLiveArtifactTarget,
    ) -> Vec<WorthQueryEntity> {
        Vec::new()
    }

    fn drain_live_patches_for_target(
        &mut self,
        _target: &WorthQueryLiveArtifactTarget,
    ) -> Vec<WorthQueryLivePatch> {
        Vec::new()
    }

    fn affected_live_view_targets(
        &self,
        receipt: &WorthQueryMutationReceipt,
    ) -> Vec<WorthQueryLiveArtifactTarget> {
        let mut affected = receipt
            .deltas
            .iter()
            .flat_map(|delta| {
                self.live_views
                    .iter()
                    .filter(move |(_, collection)| {
                        delta
                            .target_collection_identity()
                            .same_target_collection_as(collection)
                    })
                    .map(|(target, _)| target.clone())
            })
            .collect::<Vec<_>>();
        affected.sort();
        affected.dedup();
        affected
    }
}

struct TranscriptIntentAuthority;

impl WorthQueryIntentAuthorityAdapter for TranscriptIntentAuthority {
    fn execute_intent(
        &mut self,
        bridge: &RuntimeBridge,
        _relational_runtime: Option<&mut RelationalRuntime>,
        declaration: &WorthQueryIntentDeclaration,
    ) -> Result<WorthQueryIntentExecution, WorthQueryWorkspaceError> {
        let collection = declaration
            .input_string_field("collection")
            .unwrap_or("TranscriptEntity")
            .to_string();
        let mutation_receipt = transcript_intent_mutation_receipt(bridge, &collection)?;
        Ok(WorthQueryIntentExecution::admitted(
            declaration.strategy_name(),
            declaration.strategy_version(),
            "transcript-strategy-descriptor-digest",
            declaration.input_digest(),
            hash_parts(&[
                "transcript-intent-produced-mutation".to_string(),
                mutation_receipt
                    .commit_identity
                    .evidence_identity()
                    .as_str()
                    .to_string(),
                mutation_receipt
                    .snapshot_identity
                    .evidence_identity()
                    .as_str()
                    .to_string(),
            ]),
            [
                "transcript-relational-invariant:acyclic",
                "transcript-relational-invariant:authority-lane",
            ],
            mutation_receipt,
        ))
    }
}

fn transcript_intent_mutation_receipt(
    bridge: &RuntimeBridge,
    collection: &str,
) -> Result<WorthQueryMutationReceipt, WorthQueryWorkspaceError> {
    let entity_identity = crate::memory_workspace::WorthQueryEntityIdentity::from_relational_record(
        worth_runtime_bridge::facade::RelationalBridgeRecordIdentityParts::entity(1, 1, 0),
    );
    let snapshot_identity = WorthQuerySnapshotIdentity::from_relational_snapshot(
        RelationalBridgeSnapshotIdentityParts::new(1, 1),
    );
    let touch = crate::runtime::WorthQueryAspectTouch::aspect_field_path(
        worth_foundational::facade::AspectKey::new("identity")
            .expect("transcript identity aspect should admit"),
        worth_foundational::facade::CanonicalFieldPath::single(
            worth_foundational::facade::FieldKey::new("id")
                .expect("transcript identity field should admit"),
        ),
    );
    let command = WorthQueryWriteCommand::UpdateAspects {
        entity_identity: entity_identity.clone(),
        aspects: vec![WorthQueryAuthoredAspectMutation::new_set(
            touch.clone(),
            WorthQueryAuthoredAspectMutation::native_string_value("transcript-intent-entity-1"),
        )
        .map_err(|error| WorthQueryWorkspaceError::new(format!("{error:?}")))?],
        metadata: Default::default(),
        naming_intent: None,
        continuity_intent: None,
    };
    let contracts = crate::runtime::native_aspect_contracts::WorthQueryNativeAspectContractRegistry::from_contracts(
        transcript_aspect_contracts(&[]),
    )
    .map_err(|error| WorthQueryWorkspaceError::new(format!("{error:?}")))?;
    let mutation = WorthQueryBackendAdmissibleMutation::from_authored_command(command, &contracts)
        .map_err(|error| WorthQueryWorkspaceError::new(format!("{error:?}")))?;
    let bridge_authority = crate::runtime::build_bridge_authority_bundle(
        bridge,
        &snapshot_identity,
        &mutation,
        crate::runtime::WorthQueryBridgeMutationTarget::new(
            collection,
            &entity_identity,
            WorthQueryMutationKind::Updated,
        ),
    )?;
    Ok(WorthQueryMutationReceipt::from_bridge_authoritative_parts(
        WorthQueryCommitIdentity::from_relational_commit_id(1),
        snapshot_identity,
        vec![WorthQueryMutationDelta::from_touched_aspects(
            collection,
            entity_identity,
            WorthQueryMutationKind::Updated,
            vec![touch],
        )],
        bridge_authority,
    ))
}

struct TranscriptSignalSink;

impl WorthQueryRuntimeSignalSinkAdapter for TranscriptSignalSink {
    fn route_write_receipt(
        &mut self,
        receipt: &WorthQueryMutationReceipt,
    ) -> Result<SignalInvalidationBoundaryReceipt, WorthQueryWorkspaceError> {
        let routed = self.build_signal_invalidation_routing_receipt(receipt)?;
        self.build_signal_invalidation_boundary_receipt(receipt, routed)
    }
}

struct TranscriptSubscriptionActivation;

impl WorthQueryRuntimeSubscriptionActivationAdapter for TranscriptSubscriptionActivation {
    fn support_evidence_identity(&self) -> WorthQueryEvidenceIdentity {
        runtime_subscription_support_evidence_identity("transcript-subscription-activation")
    }

    fn admit_activation(
        &mut self,
        view_name: &str,
        activation: &SubscriptionActivationInput,
    ) -> Result<SubscriptionActivationBoundaryReceipt, WorthQueryWorkspaceError> {
        let receipt = self.build_subscription_activation_receipt(view_name, activation);
        Ok(self.build_subscription_activation_boundary_receipt(view_name, activation, receipt))
    }
}

struct TranscriptPreviewBasis;

impl WorthQueryRuntimePreviewBasisAdapter for TranscriptPreviewBasis {
    fn admit_preview_basis(
        &self,
        label: &WorthQuerySessionLabel,
        effect_policy: WorthQueryEffectPolicy,
        authority: &WorthQueryRuntimeEvidenceAuthority,
    ) -> Result<WorthQueryPreviewBasisAdmission, WorthQueryWorkspaceError> {
        Ok(WorthQueryPreviewBasisAdmission::new(
            authority,
            label.clone(),
            effect_policy,
            WorthQueryBasisAdmissionEvidenceRow::rows_from_values(["transcript-preview-basis"]),
        ))
    }
}

struct TranscriptInspectorEvidence;

impl WorthQueryRuntimeInspectorEvidenceAdapter for TranscriptInspectorEvidence {
    fn inspect_write_receipt(
        &self,
        receipt: &WorthQueryWriteReceipt,
        authority: &WorthQueryRuntimeEvidenceAuthority,
    ) -> Result<WorthQueryRuntimeInspectionEvidence, WorthQueryWorkspaceError> {
        Ok(WorthQueryRuntimeInspectionEvidence::new(
            authority,
            "transcript-write-receipt",
            receipt.authority_lane(),
            ["transcript-inspector-evidence"],
        ))
    }
}
