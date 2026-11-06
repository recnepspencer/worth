mod budget;
use budget::*;

use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use std::num::NonZeroUsize;

use super::super::{RepresentativeArtifacts, WorthQueryLowerRuntimeRepresentativeEvidenceSource};
use crate::declarative_live::{
    declare_runtime_live_query_session_with_grouped_baseline, DeclarativeLiveQueryRequest,
    DeclarativeLiveViewShape, DeclarativeProjectionField,
};
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::lower_runtime_routing::{
    worth_query_lower_runtime_retained_evidence_identity,
    LiveViewDeclarationAdmissionBoundaryReceipt, SubscriptionActivationBoundaryReceipt,
    WorthQueryLowerRuntimeAuthorityOwner, WorthQueryLowerRuntimeBoundaryEnvelope,
    WorthQueryLowerRuntimeBoundaryExecutionReceipt, WorthQueryLowerRuntimeCapabilityEligibility,
    WorthQueryLowerRuntimeCapabilityRequest, WorthQueryLowerRuntimeRouteKind,
    WorthQueryLowerRuntimeRoutePlan, WorthQueryLowerRuntimeRouteSubjectIdentity,
    WorthQueryLowerRuntimeSeamKey, WorthQueryLowerRuntimeSubjectIdentity,
};
use crate::memory_workspace::WorthQuerySnapshotIdentity;
use crate::runtime::{
    LiveViewDeclarationAdmissionReceipt, WorthQueryRuntimeLiveSubscriptionInstallation,
};
use crate::schema_view::{QuerySchemaView, ScalarAspectType, SchemaFieldView};
use crate::subscription::{
    admit_active_subscription_lane, admit_query_subscription, attach_subscription_consumer,
    declare_query_subscription, lower_query_subscription_to_bridge, open_active_subscription_lane,
    prepare_subscription_activation, select_query_subscription_family, ActiveAllocationScopeWidth,
    ActiveFanoutWidth, ActiveRegistryLookupWidth, ActiveSubscriptionAllocationPosture,
    ActiveSubscriptionRuntime, ActiveSubscriptionWorkBudget, ConsumerDeliveryPacingWidth,
    DeliveryBackpressurePolicy, QuerySubscriptionAdmissionBudget,
    QuerySubscriptionAdmissionDimensions, QuerySubscriptionBridgeLoweringBudget,
    QuerySubscriptionSliceBudget, QuerySubscriptionWorkBudget, SubscriptionActivationInput,
    SubscriptionConsumerAttachmentBudget, SubscriptionConsumerAttachmentRequest,
};

const LIVE_AGGREGATE_VIEW_NAME: &str = "tasks.live-aggregate";
const RUNTIME_SUBSCRIPTION_FAMILY_BUDGET_POLICY: &str =
    "runtime-live-subscription-family:scratch_buffer_only:canonical=64:relationship=64:policy=64:projection=512:tenant=1";
const RUNTIME_SUBSCRIPTION_SLICE_BUDGET_POLICY: &str =
    "runtime-live-subscription-slice:scratch_buffer_only:all-widths=64";
const RUNTIME_SUBSCRIPTION_BRIDGE_BUDGET_POLICY: &str =
    "runtime-live-subscription-bridge:admitted:bridge=1:slice=64:policy=64:basis=64:signal=64";
const RUNTIME_SUBSCRIPTION_ADMISSION_BUDGET_POLICY: &str =
    "runtime-live-subscription-admission:admitted:all-widths=64";
const RUNTIME_ACTIVE_LIFECYCLE_BUDGET_POLICY: &str =
    "runtime-live-active-lifecycle:registry=1:fanout=1:allocation=1:lifecycle_arena";
const RUNTIME_CONSUMER_ATTACHMENT_BUDGET_POLICY: &str =
    "runtime-live-consumer-attachment:fanout=1:pacing=1:allocation=1:retain_within_window";

pub(crate) fn representative_public_live_view_declaration_row(
) -> Result<RepresentativeArtifacts, IdentityDerivationDenial> {
    let fixture = live_aggregate_fixture()?;
    Ok(aggregate_route_artifacts(
        WorthQueryLowerRuntimeSeamKey::PublicLiveViewDeclaration,
        "Public live view declaration",
        public_live_declaration_evidence(&fixture)?,
    )?)
}

pub(crate) fn representative_runtime_live_installation_orchestration_row(
) -> Result<RepresentativeArtifacts, IdentityDerivationDenial> {
    let fixture = live_aggregate_fixture()?;
    Ok(aggregate_route_artifacts(
        WorthQueryLowerRuntimeSeamKey::RuntimeLiveInstallationOrchestration,
        "Runtime live installation orchestration",
        runtime_live_installation_evidence(&fixture)?,
    )?)
}

struct LiveAggregateFixture {
    declaration_boundary: LiveViewDeclarationAdmissionBoundaryReceipt,
    activation_boundary: SubscriptionActivationBoundaryReceipt,
    _installation: WorthQueryRuntimeLiveSubscriptionInstallation,
}

fn aggregate_route_artifacts(
    seam_key: WorthQueryLowerRuntimeSeamKey,
    capability_label: &'static str,
    evidence: WorthQueryEvidenceIdentity,
) -> Result<RepresentativeArtifacts, IdentityDerivationDenial> {
    let request = WorthQueryLowerRuntimeCapabilityRequest::new(
        seam_key,
        WorthQueryLowerRuntimeRouteKind::RoutePlanning,
        WorthQueryLowerRuntimeAuthorityOwner::Query,
        capability_label,
        WorthQueryLowerRuntimeSubjectIdentity::compose(capability_label)
            .field_evidence_identity(WorthQueryEvidenceTag::new("aggregate"), &evidence)
            .seal()?,
    )?;
    let eligibility = WorthQueryLowerRuntimeCapabilityEligibility::admitted_with_evidence_identity(
        request.clone(),
        &evidence,
    )?;
    let route_plan = WorthQueryLowerRuntimeRoutePlan::new(
        eligibility.clone(),
        WorthQueryLowerRuntimeRouteSubjectIdentity::from_evidence_identity(
            capability_label,
            &evidence,
        )?,
    )?;
    let retained_evidence =
        worth_query_lower_runtime_retained_evidence_identity(capability_label, &evidence)?;
    let boundary_receipt = WorthQueryLowerRuntimeBoundaryExecutionReceipt::from_route_plan(
        &route_plan,
        &retained_evidence,
    )?;
    let envelope = WorthQueryLowerRuntimeBoundaryEnvelope::from_route_plan(
        seam_key,
        &route_plan,
        &boundary_receipt,
        &retained_evidence,
    )?;
    Ok(RepresentativeArtifacts {
        seam_key,
        request,
        eligibility,
        route_plan: Some(route_plan),
        boundary_receipt,
        envelope,
        evidence_source: WorthQueryLowerRuntimeRepresentativeEvidenceSource::RuntimeBackedFixture,
    })
}

fn public_live_declaration_evidence(
    fixture: &LiveAggregateFixture,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("declaration_request"),
                fixture
                    .declaration_boundary
                    .boundary_envelope()
                    .request_identity(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("declaration_envelope"),
                fixture
                    .declaration_boundary
                    .boundary_envelope()
                    .envelope_identity(),
            )
            .seal()?,
    )
}

fn runtime_live_installation_evidence(
    fixture: &LiveAggregateFixture,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("declaration_envelope"),
                fixture
                    .declaration_boundary
                    .boundary_envelope()
                    .envelope_identity(),
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("activation_envelope"),
                fixture
                    .activation_boundary
                    .boundary_envelope()
                    .envelope_identity(),
            )
            .seal()?,
    )
}

fn live_aggregate_fixture() -> Result<LiveAggregateFixture, IdentityDerivationDenial> {
    let request = DeclarativeLiveQueryRequest::new("Task", DeclarativeLiveViewShape::table())
        .project(DeclarativeProjectionField::from_authoring_parts(
            "identity", "id",
        ))
        .project(DeclarativeProjectionField::from_authoring_parts(
            "status", "value",
        ));
    let schema_view = QuerySchemaView::new(
        "certification-live-aggregate",
        [
            SchemaFieldView::new(
                crate::authoring::AspectName::new("identity")
                    .expect("schema aspect literal must be valid"),
                crate::authoring::FieldName::new("id").expect("schema field literal must be valid"),
                ScalarAspectType::String,
            ),
            SchemaFieldView::new(
                crate::authoring::AspectName::new("status")
                    .expect("schema aspect literal must be valid"),
                crate::authoring::FieldName::new("value")
                    .expect("schema field literal must be valid"),
                ScalarAspectType::String,
            ),
        ],
        [],
    );
    let declaration_receipt =
        LiveViewDeclarationAdmissionReceipt::from_request(LIVE_AGGREGATE_VIEW_NAME, &request)?;
    let declaration_boundary = LiveViewDeclarationAdmissionBoundaryReceipt::from_request(
        LIVE_AGGREGATE_VIEW_NAME,
        &request,
        declaration_receipt,
    )?;
    let session = declare_runtime_live_query_session_with_grouped_baseline(
        request.clone(),
        schema_view,
        phase_six_snapshot_identity("live-aggregate-snapshot")?,
        None::<Vec<crate::view_shape_live::WorthQueryGroupedBaselineMember>>,
    )
    .expect("live aggregate fixture should declare a runtime live session");
    let live_admission =
        crate::subscription::LiveQueryAdmissionArtifact::from_live_promotion_with_view(
            session.live_view().core_live_plan().descriptor(),
            crate::basis_lifecycle::basis_lifecycle()
                .current_head()
                .declare_subscription()
                .expect("current subscription basis should admit"),
            session.live_view().lowering().family(),
            QuerySubscriptionAdmissionDimensions::collection_membership(
                NonZeroUsize::new(2).expect("projection width"),
                NonZeroUsize::new(1).expect("ordering width"),
            ),
        );
    let selection = select_query_subscription_family(live_admission, work_budget())
        .expect("live aggregate fixture should select a subscription family");
    let subscription_family = selection.family().clone();
    let declaration = declare_query_subscription(selection, slice_budget())
        .expect("live aggregate fixture should declare a subscription");
    let lowering = lower_query_subscription_to_bridge(declaration, lowering_budget())
        .expect("live aggregate fixture should lower a subscription");
    let admission = admit_query_subscription(lowering, admission_budget())
        .expect("live aggregate fixture should admit the lowered subscription");
    let bridge_declaration_identity = admission.bridge_declaration_identity().clone();
    let admission_identity = admission.evidence_identity().clone();
    let basis_binding_identity = admission.basis_binding_identity().clone();
    let signal_strategy_identity = admission.signal_strategy_identity().clone();
    let activation = prepare_subscription_activation(admission);
    let activation_receipt = crate::runtime::SubscriptionActivationReceipt::from_activation(
        LIVE_AGGREGATE_VIEW_NAME,
        &activation,
        crate::runtime::runtime_subscription_support_evidence_identity(
            "certified-live-aggregate-support",
        ),
        None,
    )?;
    let activation_identity = activation_receipt.activation_identity().clone();
    let support_identity = activation_receipt.support_identity().clone();
    let activation_boundary = SubscriptionActivationBoundaryReceipt::from_activation(
        LIVE_AGGREGATE_VIEW_NAME,
        &activation,
        activation_receipt,
    )?;
    let (attachment, active_lane_counters, consumer_attachment_counters) =
        installation_attachment(&activation);
    let installation = WorthQueryRuntimeLiveSubscriptionInstallation::new(
        LIVE_AGGREGATE_VIEW_NAME,
        crate::runtime::live_subscription_source_identity(
            "query",
            activation.query_declaration_identity(),
        )?,
        session.canonical().query().digest().clone(),
        crate::runtime::live_subscription_source_identity(
            "live_view",
            &crate::runtime::live_subscription_view_shape_source_identity(
                session.live_view().lowering().family(),
            )?,
        )?,
        session.canonical().result_shape().digest().clone(),
        subscription_family,
        crate::runtime::live_subscription_source_identity(
            "subscription_declaration",
            activation.query_declaration_identity(),
        )?,
        crate::runtime::live_subscription_source_identity(
            "bridge_declaration",
            &bridge_declaration_identity,
        )?,
        crate::runtime::live_subscription_source_identity("admission", &admission_identity)?,
        crate::runtime::live_subscription_source_identity(
            "activation",
            &activation_identity,
        )?,
        crate::runtime::live_subscription_source_identity("basis_binding", &basis_binding_identity)?,
        crate::runtime::live_subscription_source_identity(
            "signal_strategy",
            &signal_strategy_identity,
        )?,
        crate::runtime::live_subscription_source_identity(
            "active_lane",
            attachment.lane_digest().evidence_identity(),
        )?,
        &attachment,
        runtime_subscription_budget_policy()?,
        crate::runtime::WorthQueryRuntimeLiveSubscriptionBudgetPolicyIdentity::active_lifecycle_policy(
            RUNTIME_ACTIVE_LIFECYCLE_BUDGET_POLICY,
        )?,
        crate::runtime::WorthQueryRuntimeLiveSubscriptionBudgetPolicyIdentity::consumer_attachment_policy(
            RUNTIME_CONSUMER_ATTACHMENT_BUDGET_POLICY,
        )?,
        active_lane_counters,
        consumer_attachment_counters,
        crate::runtime::live_subscription_source_identity(
            "support",
            &support_identity,
        )?,
        activation.counters().clone(),
    )?;

    Ok(LiveAggregateFixture {
        declaration_boundary,
        activation_boundary,
        _installation: installation,
    })
}

fn phase_six_snapshot_identity(
    label: &'static str,
) -> Result<WorthQuerySnapshotIdentity, IdentityDerivationDenial> {
    Ok(WorthQuerySnapshotIdentity::preview(
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::WriteReceiptSnapshotIdentity)
            .field_shape(WorthQueryEvidenceTag::new("phase_six_fixture"), label)
            .seal()?,
    ))
}

fn installation_attachment(
    activation: &SubscriptionActivationInput,
) -> (
    crate::subscription::SubscriptionConsumerAttachment,
    crate::subscription::ActiveSubscriptionCounters,
    crate::subscription::ActiveSubscriptionCounters,
) {
    let mut active_runtime = ActiveSubscriptionRuntime::new();
    let active_lane_admission =
        admit_active_subscription_lane(activation.clone(), active_lifecycle_budget())
            .expect("live aggregate fixture should admit an active lane");
    let handle = open_active_subscription_lane(&mut active_runtime, active_lane_admission)
        .expect("live aggregate fixture should open an active lane");
    let active_lane_counters = active_runtime.counters().clone();
    let attachment = attach_subscription_consumer(
        &mut active_runtime,
        &handle,
        SubscriptionConsumerAttachmentRequest::admitted(
            "runtime-live-aggregate",
            activation.activation_projection().label(),
        ),
        consumer_attachment_budget(),
    )
    .expect("live aggregate fixture should attach a subscription consumer");
    let consumer_attachment_counters = active_runtime.counters().clone();
    (
        attachment,
        active_lane_counters,
        consumer_attachment_counters,
    )
}
