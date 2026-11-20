//! Layout locators required to create a dispatch-outbox entity.

#[derive(Clone, Debug)]
pub struct WorthQueryDispatchOutboxLayout {
    pub entity_kind: worth_relational::facade::identity::KindId,
    pub correlation_locator: worth_foundational::facade::AspectFieldLocator,
    pub family_locator: worth_foundational::facade::AspectFieldLocator,
    pub effect_locator: worth_foundational::facade::AspectFieldLocator,
    pub protocol_identity_locator: worth_foundational::facade::AspectFieldLocator,
    pub protocol_version_locator: worth_foundational::facade::AspectFieldLocator,
    pub maximum_payload_bytes_locator: worth_foundational::facade::AspectFieldLocator,
    pub payload_locator: worth_foundational::facade::AspectFieldLocator,
    pub outcome_identity_locator: worth_foundational::facade::AspectFieldLocator,
    pub operation_slot_locator: worth_foundational::facade::AspectFieldLocator,
}
