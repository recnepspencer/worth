use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequestProjection;
use worth_relational::facade::identity::EntityId;

use super::super::operation_progression::WorthQueryResolvedCapabilityRequest;
use super::super::retained_capability_request::WorthQueryRetainedCapabilityRequest;

#[derive(Clone)]
pub struct WorthQueryElevationUpperBound {
    request: WorthQueryRetainedCapabilityRequest,
    grant: EntityId,
}

impl std::fmt::Debug for WorthQueryElevationUpperBound {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryElevationUpperBound")
            .field("capability_identity", &self.request.capability_identity())
            .field("requester", &self.request.principal())
            .field("resource", &self.request.resource())
            .field("grant", &self.grant)
            .field("cardinality", &self.request.cardinality())
            .finish_non_exhaustive()
    }
}

impl WorthQueryElevationUpperBound {
    pub fn capture<Schema, Scope, Context>(
        capability_identity: [u8; 32],
        principal: EntityId,
        projection: &ApplicationCapabilityRequestProjection<Schema, Scope, Context>,
        resolved: &WorthQueryResolvedCapabilityRequest<Schema, Scope>,
        grant: EntityId,
    ) -> Self {
        Self {
            request: WorthQueryRetainedCapabilityRequest::capture(
                capability_identity,
                principal,
                projection,
                resolved,
            ),
            grant,
        }
    }

    pub const fn capability_identity(&self) -> [u8; 32] {
        self.request.capability_identity()
    }

    pub const fn requester(&self) -> EntityId {
        self.request.principal()
    }

    pub const fn resource(&self) -> EntityId {
        self.request.resource()
    }

    pub const fn grant(&self) -> EntityId {
        self.grant
    }

    pub fn action(&self) -> &AspectValue {
        self.request.action()
    }

    pub fn purpose(&self) -> &AspectValue {
        self.request.purpose()
    }

    pub fn field(&self) -> Option<&AspectValue> {
        self.request.field()
    }

    pub fn magnitude(&self) -> Option<&AspectValue> {
        self.request.magnitude()
    }

    pub fn cardinality(&self) -> u32 {
        self.request.cardinality()
    }

    pub fn matches_active_request(
        &self,
        request: &WorthQueryRetainedCapabilityRequest,
        elevation: EntityId,
        grant: EntityId,
    ) -> bool {
        self.request.matches_elevated_request(request, elevation) && grant == self.grant
    }
}
