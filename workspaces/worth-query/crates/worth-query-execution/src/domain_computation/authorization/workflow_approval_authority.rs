//! Descriptive durable approval inputs, never an authorization permit by themselves.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
pub(in crate::domain_computation) mod decoding;
mod lineage_inventory;
use crate::domain_computation::primary_graph::workflow::actor::{
    fund_actor_codec, ActorCodecFunding,
};
use decoding::{DecodedApprovalLineage, DecodedApprovalPrincipal};
use worth_relational::facade::identity::EntityId;

use super::{
    delegation_admission::WorthQueryCapabilityObservationPosture,
    WorkflowApprovalRequestEncodingDenial, WorthQueryRetainedCapabilityAuthorization,
    WorthQueryRetainedCapabilityRequest, MAXIMUM_PORTABLE_BYTES,
};
use crate::domain_computation::primary_graph::WorthQueryDurablePrincipalCurrentness;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PortableApprovalSupport<
    Request = String,
    Identity = String,
    Lineage = super::WorthQueryDurableCapabilityLineage,
    Timeline = String,
> {
    request: Request,
    grant: EntityId,
    capability_authority_identity: Identity,
    posture: PortableSupportPosture,
    role: PortableSupportRole,
    lineage: Lineage,
    timeline: Timeline,
    expiry: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum PortableSupportPosture {
    Active,
    UpperBound,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum PortableSupportRole {
    DelegationTarget,
    ElevationUpperBound,
}

#[derive(Clone)]
pub(in crate::domain_computation) struct WorthQueryWorkflowApprovalSupportBasis {
    data: Arc<ApprovalSupportData>,
}
struct ApprovalSupportData {
    request: WorthQueryRetainedCapabilityRequest,
    grant: EntityId,
    capability_authority_identity: String,
    posture: WorthQueryCapabilityObservationPosture,
    role: super::WorthQueryCapabilitySupportRole,
    lineage: super::WorthQueryDurableCapabilityLineage,
    timeline: worth_query_declaration::facade::application_capability::ApplicationCapabilityValidityTimeline,
    expiry: u64,
    // Keep the payload admission until all decoded fields have been dropped.
    _text_funding: Arc<ActorCodecFunding>,
    _node_funding: Arc<ActorCodecFunding>,
}

impl WorthQueryWorkflowApprovalSupportBasis {
    pub(super) fn request(&self) -> &WorthQueryRetainedCapabilityRequest {
        &self.data.request
    }

    pub(super) fn grant(&self) -> EntityId {
        self.data.grant
    }

    pub(super) fn capability_authority_identity(&self) -> &str {
        &self.data.capability_authority_identity
    }

    pub(super) fn posture(&self) -> WorthQueryCapabilityObservationPosture {
        self.data.posture
    }

    pub(super) fn role(&self) -> super::WorthQueryCapabilitySupportRole {
        self.data.role
    }

    pub(super) fn lineage(&self) -> &super::WorthQueryDurableCapabilityLineage {
        &self.data.lineage
    }

    pub(super) fn timeline(&self) -> worth_query_declaration::facade::application_capability::ApplicationCapabilityValidityTimeline{
        self.data.timeline
    }

    pub(super) fn expiry(&self) -> u64 {
        self.data.expiry
    }
}

pub(in crate::domain_computation) fn encode_workflow_approval_support(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    authorization: &WorthQueryRetainedCapabilityAuthorization,
) -> Result<
    crate::domain_computation::primary_graph::workflow::actor::encoding::FundedActorText,
    WorkflowApprovalRequestEncodingDenial,
> {
    let request = authorization
        .supporting()
        .map(|support| {
            support
                .request()
                .encode_workflow_approval_request(execution)
        })
        .transpose()?;
    let portable = authorization
        .supporting()
        .map(|support| {
            Ok(PortableApprovalSupport {
                request: request
                    .as_ref()
                    .expect("support request encoded above")
                    .as_str(),
                grant: support.grant(),
                capability_authority_identity: support.capability_authority_identity(),
                posture: match support.posture() {
                    WorthQueryCapabilityObservationPosture::Active => {
                        PortableSupportPosture::Active
                    }
                    WorthQueryCapabilityObservationPosture::UpperBound => {
                        PortableSupportPosture::UpperBound
                    }
                },
                role: match support.role() {
                    super::WorthQueryCapabilitySupportRole::DelegationTarget => {
                        PortableSupportRole::DelegationTarget
                    }
                    super::WorthQueryCapabilitySupportRole::ElevationUpperBound => {
                        PortableSupportRole::ElevationUpperBound
                    }
                },
                lineage: support.decision().durable_lineage_view(),
                timeline: support.timeline().canonical_name(),
                expiry: match support.expiry() {
                    worth_foundational::facade::AspectValue::UInt64(value) => *value,
                    _ => return Err(WorkflowApprovalRequestEncodingDenial::InvalidRequest),
                },
            })
        })
        .transpose()?;
    crate::domain_computation::primary_graph::workflow::actor::encoding::encode(
        execution,
        &portable,
        MAXIMUM_PORTABLE_BYTES,
    )
}

pub(in crate::domain_computation) fn decode_workflow_approval_support(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    encoded: &str,
) -> Result<
    Option<WorthQueryWorkflowApprovalSupportBasis>,
    super::retained_capability_request::WorkflowApprovalRequestDecodingDenial,
> {
    if encoded.len() > MAXIMUM_PORTABLE_BYTES {
        return Err(().into());
    }
    // serde unescape scratch <2*input and retained owned text <=input.
    let text_funding = fund_actor_codec(execution, encoded.len().checked_mul(3).ok_or(())?)?;
    type Ignored = serde::de::IgnoredAny;
    let inventory: Option<
        PortableApprovalSupport<Ignored, Ignored, lineage_inventory::LineageCount, Ignored>,
    > = serde_json::from_str(encoded).map_err(|_| ())?;
    let nodes = inventory.as_ref().map_or(0, |support| support.lineage.0);
    let node_funding = fund_actor_codec(
        execution,
        nodes
            .checked_mul(std::mem::size_of::<super::WorthQueryDurableCapabilityLineage>())
            .ok_or(())?,
    )?;
    let portable: Option<PortableApprovalSupport> =
        serde_json::from_str(encoded).map_err(|_| ())?;
    crate::domain_computation::primary_graph::workflow::actor::encoding::check_live(execution)?;
    portable
        .map(|support| {
            if support.capability_authority_identity.is_empty() {
                return Err(().into());
            }
            Ok(WorthQueryWorkflowApprovalSupportBasis { data: Arc::new(ApprovalSupportData {
                _text_funding: Arc::clone(&text_funding),
                _node_funding: Arc::clone(&node_funding),
                request: WorthQueryRetainedCapabilityRequest::decode_workflow_approval_request(
                    execution, &support.request,
                )?,
                grant: support.grant,
                capability_authority_identity: support.capability_authority_identity,
                posture: match support.posture {
                    PortableSupportPosture::Active => {
                        WorthQueryCapabilityObservationPosture::Active
                    }
                    PortableSupportPosture::UpperBound => {
                        WorthQueryCapabilityObservationPosture::UpperBound
                    }
                },
                role: match support.role {
                    PortableSupportRole::DelegationTarget => super::WorthQueryCapabilitySupportRole::DelegationTarget,
                    PortableSupportRole::ElevationUpperBound => super::WorthQueryCapabilitySupportRole::ElevationUpperBound,
                },
                lineage: support.lineage,
                timeline: match support.timeline.as_str() {
                    "unix-epoch-seconds" => worth_query_declaration::facade::application_capability::ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
                    "unix-epoch-milliseconds" => worth_query_declaration::facade::application_capability::ApplicationCapabilityValidityTimeline::UnixEpochMilliseconds,
                    _ => return Err(().into()),
                },
                expiry: support.expiry,
            }) })
        })
        .transpose()
}

#[derive(Clone)]
pub(in crate::domain_computation) struct WorthQueryWorkflowApprovalAuthorityBasis {
    data: Arc<ApprovalAuthorityData>,
}
struct ApprovalAuthorityData {
    request: WorthQueryRetainedCapabilityRequest,
    principal: DecodedApprovalPrincipal,
    grant: EntityId,
    capability_authority_identity: String,
    lineage: DecodedApprovalLineage,
    support: Option<WorthQueryWorkflowApprovalSupportBasis>,
    dependencies: super::WorthQueryWorkflowApprovalDependencies,
    timeline: worth_query_declaration::facade::application_capability::ApplicationCapabilityValidityTimeline,
    expiry: u64,
}

impl WorthQueryWorkflowApprovalAuthorityBasis {
    pub(in crate::domain_computation) fn from_retained_approval(
        request: WorthQueryRetainedCapabilityRequest,
        principal: DecodedApprovalPrincipal,
        grant: EntityId,
        capability_authority_identity: String,
        lineage: DecodedApprovalLineage,
        support: Option<WorthQueryWorkflowApprovalSupportBasis>,
        dependencies: super::WorthQueryWorkflowApprovalDependencies,
        timeline: worth_query_declaration::facade::application_capability::ApplicationCapabilityValidityTimeline,
        expiry: u64,
    ) -> Self {
        Self {
            data: Arc::new(ApprovalAuthorityData {
                request,
                principal,
                grant,
                capability_authority_identity,
                lineage,
                support,
                dependencies,
                timeline,
                expiry,
            }),
        }
    }

    pub(super) fn request(&self) -> &WorthQueryRetainedCapabilityRequest {
        &self.data.request
    }

    pub(super) fn principal(&self) -> &WorthQueryDurablePrincipalCurrentness {
        self.data.principal.value()
    }

    pub(super) fn grant(&self) -> EntityId {
        self.data.grant
    }

    pub(super) fn capability_authority_identity(&self) -> &str {
        &self.data.capability_authority_identity
    }

    pub(super) fn lineage(&self) -> &super::WorthQueryDurableCapabilityLineage {
        self.data.lineage.value()
    }

    pub(super) fn support(&self) -> Option<&WorthQueryWorkflowApprovalSupportBasis> {
        self.data.support.as_ref()
    }

    pub(super) fn dependencies(&self) -> &super::WorthQueryWorkflowApprovalDependencies {
        &self.data.dependencies
    }

    pub(super) fn timeline(
        &self,
    ) -> worth_query_declaration::facade::application_capability::ApplicationCapabilityValidityTimeline{
        self.data.timeline
    }

    pub(super) fn expiry(&self) -> u64 {
        self.data.expiry
    }
}
