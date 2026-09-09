use worth_ui_host_contract::{
    UiHostSurfacePresentationMode, UiMountedContentGeneration, UiMountedLogicalDamage,
    UiMountedPaintCommand, UiMountedPaintCommandIdentity, UiMountedPaintOrderIdentity,
    UiMountedPaintOrderIntegrity, UiMountedPortalOverlayMechanic, UiMountedPresentationSample,
    UiMountedPresentationSampleChange, UiMountedPresentationSampleInput,
    UiMountedSurfaceBindingRequirement, WorthUiHostCapabilityObservationGeneration,
};

use super::raster::{raster_damage_for_basis, UiNativeRasterBasis};
use super::{UiNativeRasterOperation, UiNativeRetainedDrawList};

const PUBLISHED_SUCCESSOR_SENTINEL_RGBA8: [u8; 4] = [0x5a, 0x7d, 0xa3, 0xff];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiNativePortalSampleReplayCertificationDenial {
    InvalidRetainedPresentation,
    InvalidSample,
    InvalidReplay,
    InvalidRasterBasis,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiNativePortalSampleReplayCertification {
    normalized_damage: Box<[worth_ui_host_contract::UiMountedCanonicalBox]>,
    published_successor_top_pixel: [u8; 4],
}

#[doc(hidden)]
pub fn certify_portal_sample_replay(
    portal: UiMountedPortalOverlayMechanic,
    change: UiMountedPresentationSampleChange,
    damage: &[UiMountedLogicalDamage],
    physical_extent: [u32; 2],
    scale_factor: f32,
) -> Result<UiNativePortalSampleReplayCertification, UiNativePortalSampleReplayCertificationDenial>
{
    let requirement = UiMountedSurfaceBindingRequirement::new(
        portal.surface(),
        portal.anchor_presentation().host_surface(),
        portal.binding(),
        WorthUiHostCapabilityObservationGeneration::new(1),
        1,
        UiHostSurfacePresentationMode::NativeDisplay,
    );
    let content = UiMountedContentGeneration::mint_unbound()
        .map_err(|_| UiNativePortalSampleReplayCertificationDenial::InvalidSample)?;
    let identity = UiMountedPaintCommandIdentity::portal_overlay(&portal);
    if change.command() != identity {
        return Err(UiNativePortalSampleReplayCertificationDenial::InvalidSample);
    }
    let command = UiMountedPaintCommand::PortalOverlay {
        identity,
        mechanic: portal,
    };
    let order = [UiMountedPaintOrderIdentity::for_command(identity)];
    let mut retained = UiNativeRetainedDrawList::from_complete(
        portal.frame(),
        portal.surface(),
        portal.binding(),
        content,
        requirement.baseline(),
        std::slice::from_ref(&command),
        &order,
        UiMountedPaintOrderIntegrity::for_order(&order),
        &[],
    )
    .map_err(|_| UiNativePortalSampleReplayCertificationDenial::InvalidRetainedPresentation)?;
    let sample =
        UiMountedPresentationSample::from_inert_mechanics(UiMountedPresentationSampleInput {
            frame: portal.frame(),
            surface: portal.surface(),
            binding: portal.binding(),
            content,
            baseline: requirement.baseline(),
            changes: vec![change],
            damage: damage.to_vec(),
            production_cost: Default::default(),
        })
        .map_err(|_| UiNativePortalSampleReplayCertificationDenial::InvalidSample)?;
    let basis = UiNativeRasterBasis::new(physical_extent, scale_factor);
    let atlas = crate::native::text_atlas::UiNativeTextAtlas::new();
    retained
        .initialize_physical_coverage(basis, &atlas)
        .map_err(|_| UiNativePortalSampleReplayCertificationDenial::InvalidReplay)?;
    let (mut replay, mut undo) = retained
        .stage_sample(&sample)
        .map_err(|_| UiNativePortalSampleReplayCertificationDenial::InvalidReplay)?;
    retained
        .refresh_physical_sample(&sample, &mut undo, basis, &mut replay)
        .map_err(|_| UiNativePortalSampleReplayCertificationDenial::InvalidReplay)?;
    let normalized_damage = replay
        .regions
        .iter()
        .map(|region| region.damage.bounds())
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let plan = super::retained_raster::build_plan(
        basis,
        &retained,
        replay,
        0,
        &crate::native::text_atlas::UiNativeTextAtlas::new(),
    )
    .map_err(|_| UiNativePortalSampleReplayCertificationDenial::InvalidReplay)?;
    let published = raster_damage_for_basis(portal.bounds(), basis)
        .map_err(|_| UiNativePortalSampleReplayCertificationDenial::InvalidRasterBasis)?
        .ok_or(UiNativePortalSampleReplayCertificationDenial::InvalidRasterBasis)?;
    let [left, top, width, _] = published.physical_bounds();
    let probe = [left + (width / 2.0).floor(), top];
    let mut pixel = PUBLISHED_SUCCESSOR_SENTINEL_RGBA8;
    for operation in &plan.operations {
        match operation {
            UiNativeRasterOperation::Clear(rect) if contains(*rect, probe) => {
                pixel = [0, 0, 0, 0];
            }
            UiNativeRasterOperation::FilledRect { rect, source_rgba8 }
                if contains(*rect, probe) =>
            {
                pixel = *source_rgba8;
            }
            UiNativeRasterOperation::Clear(_)
            | UiNativeRasterOperation::FilledRect { .. }
            | UiNativeRasterOperation::Glyph(_) => {}
        }
    }
    Ok(UiNativePortalSampleReplayCertification {
        normalized_damage,
        published_successor_top_pixel: pixel,
    })
}

impl UiNativePortalSampleReplayCertification {
    pub fn normalized_damage(&self) -> &[worth_ui_host_contract::UiMountedCanonicalBox] {
        &self.normalized_damage
    }

    pub const fn published_successor_top_pixel(&self) -> [u8; 4] {
        self.published_successor_top_pixel
    }
}

fn contains(rect: super::RasterRect, point: [f32; 2]) -> bool {
    let [left, top, width, height] = rect.physical_bounds();
    point[0] >= left && point[0] < left + width && point[1] >= top && point[1] < top + height
}
