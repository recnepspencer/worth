//! Logical custody of explicitly process-local application resources.
use worth_foundational::facade::{AspectValue, OwnedAsyncObligationIdentity};
use worth_relational::facade::identity::EntityId;
use worth_runtime_bridge::facade::BridgeOwnedAsyncDeclarationDescription;
use worth_signal::facade::{InFlightResourceRequest, ResourceRequestHandle};
pub(super) mod codec;
mod fold;

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct QueryResourceOriginal {
    pub binding: String,
    pub package: [u8; 32],
    pub schema: [u8; 32],
    pub branch: String,
    pub commit: u64,
    pub principal: EntityId,
    pub scope_subject: EntityId,
    pub scope_aspect: String,
    pub scope_fields: Vec<String>,
    pub scope_value: AspectValue,
    pub query: [u8; 32],
    pub parameters: Vec<(String, AspectValue)>,
    pub input: AspectValue,
    pub declaration: BridgeOwnedAsyncDeclarationDescription,
}
/// Unverified ingress stays bytes; it is never promoted to a verified result.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize, Default)]
pub struct QueryResourceHandoff {
    pub original_request: Option<ResourceRequestHandle>,
    pub request: Option<ResourceRequestHandle>,
    pub ingress: Vec<Vec<u8>>,
    pub result: Option<(QueryResourceResultKind, AspectValue)>,
    pub release: Option<QueryResourceRelease>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum QueryResourceResultKind {
    Result,
    DomainFailure,
    EffectsIndeterminate,
}
/// Created from actual live release, never reconstructed as Native authority.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct QueryResourceRelease {
    pub terminal: Option<InFlightResourceRequest>,
    pub predecessor: Option<InFlightResourceRequest>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryResourceObligation {
    pub identity: OwnedAsyncObligationIdentity,
    pub original: QueryResourceOriginal,
    pub handoff: QueryResourceHandoff,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueryResourceRecord {
    Retain(QueryResourceObligation),
    Handoff {
        identity: OwnedAsyncObligationIdentity,
        handoff: QueryResourceHandoff,
    },
    Discharge {
        identity: OwnedAsyncObligationIdentity,
    },
}
