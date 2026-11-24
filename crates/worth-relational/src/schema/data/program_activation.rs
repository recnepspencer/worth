//! Portable facts for an exact recovered program activation.
/// Exact program activation change authenticated by Query's installed owner.
/// This description is joined to that owner's occurrence proof and the
/// recovered native source root before it permits any protected-row change.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RecoveredCheckpointProgramActivationUpdate {
    pub entity_id: crate::identity::data::EntityId,
    #[serde(with = "crate::aspect_wire::serde_canonical_aspect_field_locator")]
    pub field_locator: worth_foundational::facade::AspectFieldLocator,
    #[serde(with = "crate::aspect_wire::serde_canonical_aspect_value")]
    pub expected: worth_foundational::facade::AspectValue,
    #[serde(with = "crate::aspect_wire::serde_canonical_aspect_value")]
    pub replacement: worth_foundational::facade::AspectValue,
}
