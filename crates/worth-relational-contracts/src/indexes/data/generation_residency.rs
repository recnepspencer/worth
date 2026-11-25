//! Serving-catalog residency observation carries no generation payload or read proof.
/// No-load state of the generation catalog for one exact observation.
/// DeferredImage does not assert that its source is healthy or that no export
/// has read it; the checkpoint image has not been admitted into this catalog.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DerivedIndexGenerationResidency {
    DeferredImage,
    Absent,
    Published,
}
