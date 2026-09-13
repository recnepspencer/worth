mod adjudication_contract;
mod contrast;
mod control_point_validation;
mod interactive_content_validation;
mod model;
mod native_control_contract;
mod pixel_region_validation;
mod validation;

#[cfg(test)]
mod tests;

use model::PlatformPulseVisualContractManifest;

pub(super) use adjudication_contract::{
    checked_in_adjudication_contract, PlatformPulseVisualAdjudicationContract,
};
pub(super) use native_control_contract::{
    action_control, confirmation_control, portal_control, portal_control_for_extent,
    PlatformPulseNativeControlContract,
};
pub(super) use validation::PlatformPulseVisualContractFailure;

const SOURCE: &str = include_str!("platform_pulse_visual_contract.json");

fn checked_in(
) -> Result<PlatformPulseVisualContractManifest, validation::PlatformPulseVisualContractFailure> {
    let manifest = serde_json::from_str::<PlatformPulseVisualContractManifest>(SOURCE)
        .map_err(|_| validation::PlatformPulseVisualContractFailure::Decode)?;
    validation::validate(&manifest)?;
    Ok(manifest)
}
