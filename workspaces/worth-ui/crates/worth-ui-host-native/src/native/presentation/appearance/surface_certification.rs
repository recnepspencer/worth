use worth_ui_host_contract::UiMountedSurfaceAppearanceMechanic;

use super::{
    geometry::{UiNativeAppearanceScale, UiNativeGeometryDenial},
    surface_pipeline::UiNativeSurfacePipeline,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiNativeSurfaceSampleCertification {
    fill_coverage: u16,
    border_coverage: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiNativeSurfaceSampleCertificationDenial {
    UnsupportedScale,
    CoordinateOverflow,
}

pub fn certify_mounted_surface_sample(
    mechanic: &UiMountedSurfaceAppearanceMechanic,
    scale_milli: u16,
    pixel_x: i64,
    pixel_y: i64,
) -> Result<UiNativeSurfaceSampleCertification, UiNativeSurfaceSampleCertificationDenial> {
    let scale = UiNativeAppearanceScale::qualified(scale_milli).map_err(map_geometry_denial)?;
    let sample = UiNativeSurfacePipeline::prepare(mechanic, scale)
        .map_err(map_geometry_denial)?
        .sample(pixel_x, pixel_y);
    Ok(UiNativeSurfaceSampleCertification {
        fill_coverage: sample.fill_coverage.units(),
        border_coverage: sample.border_coverage.units(),
    })
}

impl UiNativeSurfaceSampleCertification {
    pub const fn fill_coverage(self) -> u16 {
        self.fill_coverage
    }

    pub const fn border_coverage(self) -> u16 {
        self.border_coverage
    }
}

const fn map_geometry_denial(
    denial: UiNativeGeometryDenial,
) -> UiNativeSurfaceSampleCertificationDenial {
    match denial {
        UiNativeGeometryDenial::UnsupportedScale => {
            UiNativeSurfaceSampleCertificationDenial::UnsupportedScale
        }
        UiNativeGeometryDenial::CoordinateOverflow => {
            UiNativeSurfaceSampleCertificationDenial::CoordinateOverflow
        }
    }
}
