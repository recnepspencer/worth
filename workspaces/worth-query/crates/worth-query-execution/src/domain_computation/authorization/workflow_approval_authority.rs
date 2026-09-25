//! Descriptive durable approval inputs, never an authorization permit by themselves.

use serde::{Deserialize, Serialize};
use worth_relational::facade::identity::EntityId;

use super::{
    delegation_admission::WorthQueryCapabilityObservationPosture,
    WorkflowApprovalRequestEncodingDenial, WorthQueryRetainedCapabilityAuthorization,
    WorthQueryRetainedCapabilityRequest, MAXIMUM_PORTABLE_BYTES,
};
use crate::domain_computation::primary_graph::WorthQueryDurablePrincipalCurrentness;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PortableApprovalSupport {
    request: String,
    grant: EntityId,
    capability_authority_identity: String,
    posture: PortableSupportPosture,
    role: PortableSupportRole,
    lineage: super::WorthQueryDurableCapabilityLineage,
    timeline: String,
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
    request: WorthQueryRetainedCapabilityRequest,
    grant: EntityId,
    capability_authority_identity: String,
    posture: WorthQueryCapabilityObservationPosture,
    role: super::WorthQueryCapabilitySupportRole,
    lineage: super::WorthQueryDurableCapabilityLineage,
    timeline: worth_query_declaration::facade::application_capability::ApplicationCapabilityValidityTimeline,
    expiry: u64,
}

impl WorthQueryWorkflowApprovalSupportBasis {
    pub(super) fn request(&self) -> &WorthQueryRetainedCapabilityRequest {
        &self.request
    }

    pub(super) const fn grant(&self) -> EntityId {
        self.grant
    }

    pub(super) fn capability_authority_identity(&self) -> &str {
        &self.capability_authority_identity
    }

    pub(super) const fn posture(&self) -> WorthQueryCapabilityObservationPosture {
        self.posture
    }

    pub(super) const fn role(&self) -> super::WorthQueryCapabilitySupportRole {
        self.role
    }

    pub(super) fn lineage(&self) -> &super::WorthQueryDurableCapabilityLineage {
        &self.lineage
    }

    pub(super) const fn timeline(&self) -> worth_query_declaration::facade::application_capability::ApplicationCapabilityValidityTimeline{
        self.timeline
    }

    pub(super) const fn expiry(&self) -> u64 {
        self.expiry
    }
}

pub(in crate::domain_computation) fn encode_workflow_approval_support(
    authorization: &WorthQueryRetainedCapabilityAuthorization,
) -> Result<String, WorkflowApprovalRequestEncodingDenial> {
    let portable = authorization
        .supporting()
        .map(|support| {
            Ok(PortableApprovalSupport {
                request: support.request().encode_workflow_approval_request()?,
                grant: support.grant(),
                capability_authority_identity: support.capability_authority_identity().to_owned(),
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
                lineage: support.decision().durable_lineage(),
                timeline: support.timeline().canonical_name().to_owned(),
                expiry: match support.expiry() {
                    worth_foundational::facade::AspectValue::UInt64(value) => *value,
                    _ => return Err(WorkflowApprovalRequestEncodingDenial::InvalidRequest),
                },
            })
        })
        .transpose()?;
    let encoded = serde_json::to_string(&portable)
        .map_err(|_| WorkflowApprovalRequestEncodingDenial::InvalidRequest)?;
    (encoded.len() <= MAXIMUM_PORTABLE_BYTES)
        .then_some(encoded)
        .ok_or(WorkflowApprovalRequestEncodingDenial::TooLarge)
}

pub(in crate::domain_computation) fn decode_workflow_approval_support(
    encoded: &str,
) -> Result<Option<WorthQueryWorkflowApprovalSupportBasis>, ()> {
    if encoded.len() > MAXIMUM_PORTABLE_BYTES {
        return Err(());
    }
    let portable: Option<PortableApprovalSupport> =
        serde_json::from_str(encoded).map_err(|_| ())?;
    portable
        .map(|support| {
            if support.capability_authority_identity.is_empty() {
                return Err(());
            }
            Ok(WorthQueryWorkflowApprovalSupportBasis {
                request: WorthQueryRetainedCapabilityRequest::decode_workflow_approval_request(
                    &support.request,
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
                    _ => return Err(()),
                },
                expiry: support.expiry,
            })
        })
        .transpose()
}

#[derive(Clone)]
pub(in crate::domain_computation) struct WorthQueryWorkflowApprovalAuthorityBasis {
    request: WorthQueryRetainedCapabilityRequest,
    principal: WorthQueryDurablePrincipalCurrentness,
    grant: EntityId,
    capability_authority_identity: String,
    lineage: super::WorthQueryDurableCapabilityLineage,
    support: Option<WorthQueryWorkflowApprovalSupportBasis>,
    dependencies: super::WorthQueryWorkflowApprovalDependencies,
    timeline: worth_query_declaration::facade::application_capability::ApplicationCapabilityValidityTimeline,
    expiry: u64,
}

impl WorthQueryWorkflowApprovalAuthorityBasis {
    pub(in crate::domain_computation) fn from_retained_approval(
        request: WorthQueryRetainedCapabilityRequest,
        principal: WorthQueryDurablePrincipalCurrentness,
        grant: EntityId,
        capability_authority_identity: String,
        lineage: super::WorthQueryDurableCapabilityLineage,
        support: Option<WorthQueryWorkflowApprovalSupportBasis>,
        dependencies: super::WorthQueryWorkflowApprovalDependencies,
        timeline: worth_query_declaration::facade::application_capability::ApplicationCapabilityValidityTimeline,
        expiry: u64,
    ) -> Self {
        Self {
            request,
            principal,
            grant,
            capability_authority_identity,
            lineage,
            support,
            dependencies,
            timeline,
            expiry,
        }
    }

    pub(super) fn request(&self) -> &WorthQueryRetainedCapabilityRequest {
        &self.request
    }

    pub(super) fn principal(&self) -> &WorthQueryDurablePrincipalCurrentness {
        &self.principal
    }

    pub(super) const fn grant(&self) -> EntityId {
        self.grant
    }

    pub(super) fn capability_authority_identity(&self) -> &str {
        &self.capability_authority_identity
    }

    pub(super) fn lineage(&self) -> &super::WorthQueryDurableCapabilityLineage {
        &self.lineage
    }

    pub(super) fn support(&self) -> Option<&WorthQueryWorkflowApprovalSupportBasis> {
        self.support.as_ref()
    }

    pub(super) fn dependencies(&self) -> &super::WorthQueryWorkflowApprovalDependencies {
        &self.dependencies
    }

    pub(super) const fn timeline(
        &self,
    ) -> worth_query_declaration::facade::application_capability::ApplicationCapabilityValidityTimeline{
        self.timeline
    }

    pub(super) const fn expiry(&self) -> u64 {
        self.expiry
    }
}
