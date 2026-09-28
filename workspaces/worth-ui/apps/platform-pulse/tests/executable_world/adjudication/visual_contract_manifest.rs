mod contrast;
mod control_point_validation;
mod interactive_content_validation;
mod model;
mod pixel_region_validation;
mod validation;

#[cfg(test)]
mod tests;

use model::PlatformPulseVisualContractManifest;

const SOURCE: &str = include_str!("platform_pulse_visual_contract.json");

fn checked_in(
) -> Result<PlatformPulseVisualContractManifest, validation::PlatformPulseVisualContractFailure> {
    let manifest = serde_json::from_str::<PlatformPulseVisualContractManifest>(SOURCE)
        .map_err(|_| validation::PlatformPulseVisualContractFailure::Decode)?;
    validation::validate(&manifest)?;
    Ok(manifest)
}
