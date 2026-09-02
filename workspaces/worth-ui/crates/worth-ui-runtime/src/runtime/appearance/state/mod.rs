mod axis_demand;
mod backdrop_vector;
mod coherent_basis;
#[allow(
    dead_code,
    reason = "Gate 0 exposes a read-only coherent owner snapshot before consumption"
)]
mod owner_snapshot;
mod role_binding;
mod vector;

pub(crate) use axis_demand::UiAppearanceStateAxisDemand;
pub(crate) use backdrop_vector::UiBackdropAppearanceStateVector;
pub(crate) use coherent_basis::{UiAppearanceCoherentBasis, UiAppearanceTarget};
pub use owner_snapshot::UiAppearanceOwnerSnapshot;
pub(crate) use role_binding::{
    UiAppearanceNodeRoleBinding, UiAppearanceNodeRoleBindingDenial, UiAppearanceRoleBindingBasis,
};
pub(crate) use vector::UiAppearanceStateVector;
