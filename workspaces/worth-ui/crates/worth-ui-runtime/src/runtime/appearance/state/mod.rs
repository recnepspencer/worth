mod adapter;
mod axis_demand;
mod backdrop_vector;
mod coherent_basis;
mod consumer;
mod focus;
mod hover;
mod operability;
#[allow(
    dead_code,
    reason = "Gate 0 exposes a read-only coherent owner snapshot before consumption"
)]
mod owner_snapshot;
mod pressed;
mod role_binding;
mod selection;
mod target;
mod validation;
mod vector;

pub(crate) use adapter::UiAppearanceStateAdapterDenial;
pub(crate) use axis_demand::UiAppearanceStateAxisDemand;
pub(crate) use backdrop_vector::UiBackdropAppearanceStateVector;
#[cfg(test)]
pub(crate) use coherent_basis::validate_presentation_for_test;
pub(crate) use coherent_basis::{
    UiAppearanceCoherentBasis, UiAppearanceCoherentBasisDenial, UiAppearanceCoherentBasisInput,
};
pub(crate) use consumer::{
    UiAppearanceSelectionSelector, UiAppearanceStateConsumer, UiAppearanceStateConsumerSelection,
    UiAppearanceStateConsumerSelectionCost,
};
pub(crate) use focus::UiFocusAppearanceState;
pub(crate) use hover::UiHoverAppearanceState;
pub(crate) use operability::UiOperabilityAppearanceState;
pub use owner_snapshot::UiAppearanceOwnerSnapshot;
pub(crate) use pressed::UiPressedAppearanceState;
pub(crate) use role_binding::{
    UiAppearanceNodeRoleBinding, UiAppearanceNodeRoleBindingDenial, UiAppearanceRoleBindingBasis,
};
pub(crate) use selection::UiSelectionAppearanceState;
#[allow(
    unused_imports,
    reason = "Gate 1 retains sealed appearance target denials for later admission consumers"
)]
pub(crate) use target::{UiAppearanceTarget, UiAppearanceTargetDenial};
pub(crate) use validation::UiValidationAppearanceState;
pub(crate) use vector::{UiAppearanceStateVector, UiAppearanceStateVectorDenial};

#[cfg(test)]
#[path = "adapter_tests.rs"]
mod adapter_tests;
#[cfg(test)]
#[path = "vector_tests.rs"]
mod vector_tests;
