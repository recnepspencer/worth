//! Query backend for presentation-owned async custody only.

use worth_query::facade::{foundation, runtime};
use worth_runtime_bridge::facade::{RuntimeBridge, RuntimeBridgeBuilder};

pub(super) fn configure(
    builder: runtime::WorthQueryRuntimeBuilder,
) -> runtime::WorthQueryRuntimeBuilder {
    presentation_consumer_support(builder)
        .relational_product_bridge("worth-ui-presentation-async", presentation_bridge)
        .schema_adapter(PresentationSchema)
        .source_adapter(PresentationSource)
        .snapshot_identity(PresentationSnapshot)
        .write_authority(PresentationWriteDenial)
        .signal_sink(PresentationSignalSink)
        .subscription_activation(PresentationSubscription)
        .preview_basis(PresentationPreviewDenial)
        .inspector_evidence(PresentationInspectionDenial)
        .support_profile(presentation_support_profile())
        .build_backend_from_parts()
}

fn presentation_consumer_support(
    builder: runtime::WorthQueryRuntimeBuilder,
) -> runtime::WorthQueryRuntimeBuilder {
    use worth_query::facade::domain::{
        WorthQueryConsumerSupportDimension as Dimension,
        WorthQueryConsumerSupportPosture as Posture,
    };
    [
        Dimension::Live,
        Dimension::Invalidation,
        Dimension::AsyncResultState,
        Dimension::Recovery,
        Dimension::DependencyImpact,
        Dimension::ConditionalEvaluation,
        Dimension::ConditionalComparator,
        Dimension::ConditionalTrigger,
    ]
    .into_iter()
    .fold(builder, |builder, dimension| {
        builder.consumer_support_posture(dimension, Posture::Supported)
    })
}

fn presentation_bridge(
    source: worth_relational::facade::bridge::RuntimeBridgeRelationalSource,
) -> Result<RuntimeBridge, worth_runtime_bridge::facade::BridgeBuildError> {
    let builder = RuntimeBridgeBuilder::new()
        .with_relational_source(source)
        .with_signal_sink(crate::query_bridge_support::UiBridgeSignalSink)
        .with_writeback_authority(crate::query_bridge_support::UiBridgeWritebackAuthority);
    let mut registrations = super::presentation_bridge_registrations().into_iter();
    let (first_mapping, first_aspect) = registrations
        .next()
        .expect("the presentation async domain declares at least one bridge mapping");
    let mut builder = builder
        .register_mapping(first_mapping)
        .register_aspect_mapping(first_aspect);
    for (mapping, aspect_mapping) in registrations {
        builder = builder
            .register_mapping(mapping)
            .register_aspect_mapping(aspect_mapping);
    }
    builder.build()
}

fn presentation_support_profile() -> runtime::WorthQueryRuntimeSupportProfile {
    use runtime::{
        WorthQueryAuthorityLane as Lane, WorthQueryRuntimeFacadeFamily as Family,
        WorthQueryRuntimeFamilySupport as Support,
    };
    runtime::WorthQueryRuntimeSupportProfile::new([
        Support::supported(
            Family::Read,
            [Lane::AuthoritativeTruth],
            [],
            ["presentation-async-read"],
        ),
        Support::supported(
            Family::Live,
            [Lane::AuthoritativeTruth],
            [],
            ["presentation-async-live"],
        ),
        Support::supported(
            Family::AsyncResource,
            [Lane::AsyncResourceState],
            [],
            ["presentation-async-owned-source"],
        ),
        Support::supported(
            Family::MixedCauseDelivery,
            [Lane::BridgeExternalState],
            [],
            ["presentation-async-revalidation"],
        ),
    ])
    .with_unsupported_batch_authority()
}

struct PresentationSchema;
impl runtime::WorthQueryRuntimeSchemaAdapter for PresentationSchema {
    fn admit_live_view(
        &self,
        name: &str,
        request: &foundation::DeclarativeLiveQueryRequest,
        _schema_view: &runtime::QuerySchemaView,
    ) -> Result<
        runtime::LiveViewDeclarationAdmissionBoundaryReceipt,
        foundation::WorthQueryWorkspaceError,
    > {
        let admitted = self.build_live_view_declaration_admission_receipt(name, request);
        Ok(self.build_live_view_declaration_boundary_receipt(name, request, admitted))
    }
}

struct PresentationSource;
impl runtime::WorthQueryRuntimeSourceAdapter for PresentationSource {
    fn declare_live_view(
        &mut self,
        name: String,
        _request: foundation::DeclarativeLiveQueryRequest,
        _schema_view: runtime::QuerySchemaView,
    ) -> Result<foundation::WorthQueryLiveViewHandle, foundation::WorthQueryWorkspaceError> {
        Ok(foundation::WorthQueryLiveViewHandle::new(name))
    }
    fn close_live_view(&mut self, _name: &str) -> Result<(), foundation::WorthQueryWorkspaceError> {
        Ok(())
    }
    fn live_entities_for_target(
        &self,
        _target: &runtime::WorthQueryLiveArtifactTarget,
    ) -> Vec<foundation::WorthQueryEntity> {
        Vec::new()
    }
    fn drain_live_patches_for_target(
        &mut self,
        _target: &runtime::WorthQueryLiveArtifactTarget,
    ) -> Vec<foundation::WorthQueryLivePatch> {
        Vec::new()
    }
    fn affected_live_view_targets(
        &self,
        _receipt: &foundation::WorthQueryMutationReceipt,
    ) -> Vec<runtime::WorthQueryLiveArtifactTarget> {
        Vec::new()
    }
}

struct PresentationWriteDenial;

struct PresentationSnapshot;
impl runtime::WorthQueryRuntimeSnapshotIdentityAdapter for PresentationSnapshot {
    fn current_snapshot_identity(&self) -> foundation::WorthQuerySnapshotIdentity {
        foundation::WorthQuerySnapshotIdentity::empty_relational_state()
    }
}

impl runtime::WorthQueryRuntimeWriteAuthorityAdapter for PresentationWriteDenial {
    fn write(
        &mut self,
        _bridge: &RuntimeBridge,
        _relational_runtime: Option<&mut worth_relational::facade::runtime::RelationalRuntime>,
        _mutation: runtime::WorthQueryBackendAdmissibleMutation,
    ) -> Result<runtime::WriteAuthorityExecutionReceipt, foundation::WorthQueryWorkspaceError> {
        Err(foundation::WorthQueryWorkspaceError::new(
            "presentation-async owns no product write authority",
        ))
    }
}

struct PresentationSignalSink;
impl runtime::WorthQueryRuntimeSignalSinkAdapter for PresentationSignalSink {
    fn route_write_receipt(
        &mut self,
        _receipt: &foundation::WorthQueryMutationReceipt,
    ) -> Result<runtime::SignalInvalidationBoundaryReceipt, foundation::WorthQueryWorkspaceError>
    {
        Err(foundation::WorthQueryWorkspaceError::new(
            "presentation-async has no product writes to route",
        ))
    }
}

struct PresentationSubscription;
impl runtime::WorthQueryRuntimeSubscriptionActivationAdapter for PresentationSubscription {
    fn support_evidence_identity(&self) -> runtime::WorthQueryEvidenceIdentity {
        runtime::runtime_subscription_support_evidence_identity("presentation-async-live")
    }
    fn admit_activation(
        &mut self,
        view_name: &str,
        activation: &runtime::SubscriptionActivationInput,
    ) -> Result<runtime::SubscriptionActivationBoundaryReceipt, foundation::WorthQueryWorkspaceError>
    {
        let admitted = self.build_subscription_activation_receipt(view_name, activation);
        Ok(self.build_subscription_activation_boundary_receipt(view_name, activation, admitted))
    }
}

struct PresentationPreviewDenial;
impl runtime::WorthQueryRuntimePreviewBasisAdapter for PresentationPreviewDenial {
    fn admit_preview_basis(
        &self,
        _label: &runtime::WorthQuerySessionLabel,
        _effect_policy: runtime::WorthQueryEffectPolicy,
        _authority: &runtime::WorthQueryRuntimeEvidenceAuthority,
    ) -> Result<runtime::WorthQueryPreviewBasisAdmission, foundation::WorthQueryWorkspaceError>
    {
        Err(foundation::WorthQueryWorkspaceError::new(
            "presentation-async has no preview authority",
        ))
    }
}

struct PresentationInspectionDenial;
impl runtime::WorthQueryRuntimeInspectorEvidenceAdapter for PresentationInspectionDenial {
    fn inspect_write_receipt(
        &self,
        _receipt: &runtime::WorthQueryWriteReceipt,
        _authority: &runtime::WorthQueryRuntimeEvidenceAuthority,
    ) -> Result<runtime::WorthQueryRuntimeInspectionEvidence, foundation::WorthQueryWorkspaceError>
    {
        Err(foundation::WorthQueryWorkspaceError::new(
            "presentation-async has no write receipt to inspect",
        ))
    }
}
