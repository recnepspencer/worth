mod adapter;
mod axis_demand;
mod coherent_basis;
mod consumer;
mod focus;
mod hover;
mod operability;
mod owner_snapshot;
mod pressed;
mod selection;
mod validation;
mod vector;

pub(crate) use adapter::UiAppearanceStateAdapterDenial;
pub(crate) use axis_demand::UiAppearanceStateAxisDemand;
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
pub(crate) use selection::UiSelectionAppearanceState;
pub(crate) use validation::UiValidationAppearanceState;
pub(crate) use vector::{UiAppearanceStateVector, UiAppearanceStateVectorDenial};

#[cfg(test)]
#[path = "adapter_tests.rs"]
mod adapter_tests;
#[cfg(test)]
#[path = "vector_tests.rs"]
mod vector_tests;
