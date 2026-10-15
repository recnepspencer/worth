#[cfg(test)]
mod tests;

mod bundles;
mod classifications;
mod fixtures;
mod matrix;
mod requirements;
mod rows;

pub use bundles::MilestoneNineCertificationBundle;
pub use classifications::MilestoneNineFailureClass;
pub(crate) use fixtures::phase_three_test_narrowed_artifact;
pub use matrix::MilestoneNineCertificationAdapter;
pub use requirements::{
    MILESTONE_NINE_REQUIRED_CANONICAL_ROW_NAMES, MILESTONE_NINE_REQUIRED_REJECTION_ROW_NAMES,
};
