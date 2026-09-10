use super::bridge::certification_bridge_from_source;
use crate::declarative_live::{DeclarativeLiveQueryRequest, DeclarativeLiveViewShape};
use crate::facade::foundation::{
    DeclarativeProjectionField, WorthQueryLiveViewHandle, WorthQueryMutationDelta,
    WorthQueryMutationKind, WorthQueryMutationReceipt, WorthQueryWorkspaceError,
};
use crate::facade::runtime::{
    LiveViewDeclarationAdmissionBoundaryReceipt, QuerySchemaView, ScalarAspectType,
    SchemaFieldView, WorthQueryAuthoredAspectMutation, WorthQueryAuthorityLane,
    WorthQueryBackendAdmissibleMutation, WorthQueryExistingTruthAssertionDenial,
    WorthQueryExistingTruthProbeDenial, WorthQueryExistingTruthProbeDenialKind,
    WorthQueryIntentAuthorityAdapter, WorthQueryIntentDeclaration, WorthQueryIntentExecution,
    WorthQueryLiveArtifactTarget, WorthQueryRuntime,
    WorthQueryRuntimeExistingTruthVerificationAdapter, WorthQueryRuntimeFacadeFamily,
    WorthQueryRuntimeFamilySupport, WorthQueryRuntimeSchemaAdapter,
    WorthQueryRuntimeSnapshotIdentityAdapter, WorthQueryRuntimeSourceAdapter,
    WorthQueryRuntimeSupportProfile, WorthQueryWriteCommand,
};
use crate::identity::hash_parts;
use crate::memory_workspace::WorthQuerySnapshotIdentity;
use crate::memory_workspace::{WorthQueryEntity, WorthQueryLivePatch};
use crate::runtime::WorthQueryMutationTargetCollectionIdentity;
use std::collections::BTreeMap;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_runtime_bridge::facade::{RelationalBridgeSnapshotIdentityParts, RuntimeBridge};

use super::write_authority::CertificationWriteAuthority;
use super::{
    certification_entity_identity, certification_snapshot_identity, identity_id_touch,
    title_value_touch,
};

mod aspect_contracts;
mod invariant_violation_authority;
mod runtime_adapters;
use aspect_contracts::certification_aspect_contracts;
use invariant_violation_authority::InvariantViolationCertificationIntentAuthority;
use runtime_adapters::{
    CertificationInspectorEvidence, CertificationPreviewBasis, CertificationSignalSink,
    CertificationSubscriptionActivation,
};

pub(crate) fn certification_runtime() -> WorthQueryRuntime {
    let (source, bridge) = certification_product_root();
    WorthQueryRuntime::builder(
        crate::consumer_kit::test_backend::in_memory_test_product_world_resources(),
    )
    .aspect_contracts(certification_aspect_contracts())
    .expect("certification aspect contracts should install")
    .runtime_bridge(bridge)
    .relational_source_owner(source)
    .conditional_execution_resources(
        crate::runtime::WorthQueryConditionalExecutionResources::development(),
    )
    .schema_adapter(CertificationSchemaAdapter)
    .source_adapter(CertificationSourceAdapter::default())
    .snapshot_identity(CertificationSnapshotIdentity)
    .existing_truth_verification(CertificationExistingTruthVerification)
    .write_authority(CertificationWriteAuthority)
    .signal_sink(CertificationSignalSink)
    .subscription_activation(CertificationSubscriptionActivation)
    .preview_basis(CertificationPreviewBasis)
    .inspector_evidence(CertificationInspectorEvidence)
    .intent_authority(CertificationIntentAuthority)
    .support_profile(certification_support_profile())
    .build_backend_from_parts()
    .build()
    .expect("certification runtime backend parts should build")
}

pub(crate) fn certification_runtime_with_invariant_violation_authority() -> WorthQueryRuntime {
    let (source, bridge) = certification_product_root();
    WorthQueryRuntime::builder(
        crate::consumer_kit::test_backend::in_memory_test_product_world_resources(),
    )
    .aspect_contracts(certification_aspect_contracts())
    .expect("certification aspect contracts should install")
    .runtime_bridge(bridge)
    .relational_source_owner(source)
    .conditional_execution_resources(
        crate::runtime::WorthQueryConditionalExecutionResources::development(),
    )
    .schema_adapter(CertificationSchemaAdapter)
    .source_adapter(CertificationSourceAdapter::default())
    .snapshot_identity(CertificationSnapshotIdentity)
    .existing_truth_verification(CertificationExistingTruthVerification)
    .write_authority(CertificationWriteAuthority)
    .signal_sink(CertificationSignalSink)
    .subscription_activation(CertificationSubscriptionActivation)
    .preview_basis(CertificationPreviewBasis)
    .inspector_evidence(CertificationInspectorEvidence)
    .intent_authority(InvariantViolationCertificationIntentAuthority)
    .support_profile(certification_support_profile())
    .build_backend_from_parts()
    .build()
    .expect("certification runtime backend parts should build")
}

fn certification_product_root() -> (
    worth_query_execution::facade::integration::WorthQueryRelationalSourceOwner,
    RuntimeBridge,
) {
    let runtime = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
        .runtime_name("worth-query-intent-certification-product")
        .build();
    let source = worth_query_execution::facade::integration::WorthQueryRelationalSourceOwner::new(
        runtime,
        "intent-certification-product",
    )
    .expect("certification Product source should admit");
    let bridge = certification_bridge_from_source(source.bridge_source());
    (source, bridge)
}

struct CertificationSnapshotIdentity;

impl WorthQueryRuntimeSnapshotIdentityAdapter for CertificationSnapshotIdentity {
    fn current_snapshot_identity(&self) -> WorthQuerySnapshotIdentity {
        certification_snapshot_identity("certification-runtime-current-snapshot")
    }
}

pub(super) fn certification_support_profile() -> WorthQueryRuntimeSupportProfile {
    WorthQueryRuntimeSupportProfile::bridge_backed(
        "certification-subscription-activation",
        "certification-preview-basis",
        "certification-inspector-evidence",
    )
    .with_family_support(WorthQueryRuntimeFamilySupport::supported(
        WorthQueryRuntimeFacadeFamily::Intent,
        [
            WorthQueryAuthorityLane::AuthoritativeTruth,
            WorthQueryAuthorityLane::BranchLocalTruth,
            WorthQueryAuthorityLane::PreviewTruth,
        ],
        [],
        ["certification-intent-authority"],
    ))
    .with_bridge_backed_verification_support(
        "probe_existing",
        "direct_entity_identity",
        true,
        true,
        None,
    )
}

pub(crate) fn certification_task_live_request() -> DeclarativeLiveQueryRequest {
    DeclarativeLiveQueryRequest::new("Task", DeclarativeLiveViewShape::table())
        .project(
            DeclarativeProjectionField::from_authoring_parts("identity", "id")
                .delivered_as("identity.id"),
        )
        .project(
            DeclarativeProjectionField::from_authoring_parts("title", "value")
                .delivered_as("title"),
        )
        .order_by(DeclarativeProjectionField::from_authoring_parts(
            "title", "value",
        ))
}

pub(crate) fn certification_task_schema() -> QuerySchemaView {
    QuerySchemaView::new(
        "certification-task",
        [
            SchemaFieldView::new(
                crate::authoring::AspectName::new("identity")
                    .expect("schema aspect literal must be valid"),
                crate::authoring::FieldName::new("id").expect("schema field literal must be valid"),
                ScalarAspectType::String,
            ),
            SchemaFieldView::new(
                crate::authoring::AspectName::new("title")
                    .expect("schema aspect literal must be valid"),
                crate::authoring::FieldName::new("value")
                    .expect("schema field literal must be valid"),
                ScalarAspectType::String,
            ),
        ],
        [],
    )
}

struct CertificationSchemaAdapter;

impl WorthQueryRuntimeSchemaAdapter for CertificationSchemaAdapter {
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
struct CertificationSourceAdapter {
    live_views: BTreeMap<WorthQueryLiveArtifactTarget, WorthQueryMutationTargetCollectionIdentity>,
}

impl WorthQueryRuntimeSourceAdapter for CertificationSourceAdapter {
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

struct CertificationExistingTruthVerification;

impl WorthQueryRuntimeExistingTruthVerificationAdapter for CertificationExistingTruthVerification {
    fn verify_existing_truth_assertion(
        &self,
        _binding: &crate::runtime::WorthQueryExistingTruthTargetBinding,
        _aspects: &[crate::runtime::WorthQueryAuthoredAspectMutation],
    ) -> Result<(), WorthQueryExistingTruthAssertionDenial> {
        Ok(())
    }

    fn probe_existing_truth(
        &self,
        request: &crate::runtime::WorthQueryExistingTruthProbeRequest,
    ) -> Result<
        Vec<crate::runtime::WorthQueryExistingTruthProbeField>,
        WorthQueryExistingTruthProbeDenial,
    > {
        let mut values = Vec::with_capacity(request.aspect_touches().len());
        for aspect_touch in request.aspect_touches() {
            let value = if aspect_touch == &identity_id_touch() {
                crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(
                    "task-1".to_string(),
                )
            } else if aspect_touch == &title_value_touch() {
                crate::runtime::WorthQueryAuthoredAspectMutation::native_string_value(
                    "Seed title".to_string(),
                )
            } else {
                return Err(WorthQueryExistingTruthProbeDenial::new(
                    request.binding(),
                    WorthQueryExistingTruthProbeDenialKind::MissingProbedAspect,
                    Some(aspect_touch.clone()),
                    "certification verification adapter does not expose that aspect",
                ));
            };
            values.push(
                crate::runtime::WorthQueryExistingTruthProbeField::from_admitted_aspect_touch(
                    aspect_touch.clone(),
                    value,
                ),
            );
        }
        Ok(values)
    }
}

struct CertificationIntentAuthority;

impl WorthQueryIntentAuthorityAdapter for CertificationIntentAuthority {
    fn execute_intent(
        &mut self,
        bridge: &RuntimeBridge,
        _relational_runtime: Option<&mut RelationalRuntime>,
        declaration: &WorthQueryIntentDeclaration,
    ) -> Result<WorthQueryIntentExecution, WorthQueryWorkspaceError> {
        let collection = declaration
            .input_string_field("collection")
            .unwrap_or("Task")
            .to_string();
        let mutation_receipt = certification_intent_mutation_receipt(bridge, &collection)?;
        Ok(WorthQueryIntentExecution::admitted(
            declaration.strategy_name(),
            declaration.strategy_version(),
            "certification-strategy-descriptor-digest",
            declaration.input_digest(),
            hash_parts(&[
                "certification-intent-produced-mutation".to_string(),
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
                "certification-relational-invariant:acyclic",
                "certification-relational-invariant:authority-lane",
            ],
            mutation_receipt,
        ))
    }
}

fn certification_intent_mutation_receipt(
    bridge: &RuntimeBridge,
    collection: &str,
) -> Result<WorthQueryMutationReceipt, WorthQueryWorkspaceError> {
    let entity_identity = certification_entity_identity("certification-intent-entity-1");
    let snapshot_identity = WorthQuerySnapshotIdentity::from_relational_snapshot(
        RelationalBridgeSnapshotIdentityParts::new(1, 1),
    );
    let touch = title_value_touch();
    let command = WorthQueryWriteCommand::UpdateAspects {
        entity_identity: entity_identity.clone(),
        aspects: vec![WorthQueryAuthoredAspectMutation::new_set(
            touch.clone(),
            WorthQueryAuthoredAspectMutation::native_string_value("Certification title"),
        )
        .map_err(|error| WorthQueryWorkspaceError::new(format!("{error:?}")))?],
        metadata: Default::default(),
        naming_intent: None,
        continuity_intent: None,
    };
    let contracts = crate::runtime::native_aspect_contracts::WorthQueryNativeAspectContractRegistry::from_contracts(
        certification_aspect_contracts(),
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
        crate::memory_workspace::WorthQueryCommitIdentity::from_relational_commit_id(1),
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
