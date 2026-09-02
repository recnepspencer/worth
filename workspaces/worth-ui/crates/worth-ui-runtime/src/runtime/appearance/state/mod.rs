mod axis_demand;
mod coherent_basis;
#[allow(
    dead_code,
    reason = "Gate 0 exposes a read-only coherent owner snapshot before consumption"
)]
mod owner_snapshot;
mod vector;

pub(crate) use axis_demand::UiAppearanceStateAxisDemand;
pub(crate) use coherent_basis::{UiAppearanceCoherentBasis, UiAppearanceTarget};
pub use owner_snapshot::UiAppearanceOwnerSnapshot;
pub(crate) use vector::{UiAppearanceStateVector, UiAppearanceStateVectorDenial};
