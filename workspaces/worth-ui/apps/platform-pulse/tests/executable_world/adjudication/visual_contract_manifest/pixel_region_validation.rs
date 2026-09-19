use std::collections::BTreeMap;

use super::{
    model::{inside, PlatformPulseVisualContractManifest},
    validation::PlatformPulseVisualContractFailure,
};

pub(super) fn validate(
    manifest: &PlatformPulseVisualContractManifest,
) -> Result<(), PlatformPulseVisualContractFailure> {
    for layout in &manifest.layouts {
        let expected = expected_regions(layout.logical_client_extent)
            .ok_or(PlatformPulseVisualContractFailure::Geometry)?;
        let actual = layout
            .pixel_regions
            .iter()
            .map(|region| {
                (
                    region.identity.as_str(),
                    (region.rect, region.corner_radius),
                )
            })
            .collect::<BTreeMap<_, _>>();
        if actual.len() != layout.pixel_regions.len()
            || actual != expected
            || layout
                .pixel_regions
                .iter()
                .any(|region| !inside(region.rect, layout.logical_client_extent))
        {
            return Err(PlatformPulseVisualContractFailure::Geometry);
        }
    }
    Ok(())
}

fn expected_regions(extent: [u32; 2]) -> Option<BTreeMap<&'static str, ([u32; 4], u32)>> {
    let right_column_x = match extent {
        [960, 600] => 656,
        [1_120, 700] => 816,
        _ => return None,
    };
    Some(BTreeMap::from([
        ("portal-overlay", ([528, 88, 280, 320], 0)),
        ("portal-accent", ([528, 88, 280, 3], 0)),
        ("portal-title", ([552, 112, 232, 24], 0)),
        ("portal-body", ([552, 176, 232, 64], 0)),
        ("portal-cancel", ([552, 336, 104, 40], 0)),
        ("portal-cancel-label", ([577, 346, 64, 20], 0)),
        ("portal-primary", ([664, 336, 120, 40], 0)),
        ("portal-primary-label", ([682, 346, 88, 20], 0)),
        (
            "schema-stable-control",
            ([right_column_x + 232, 194, 24, 20], 0),
        ),
        ("schema-posture", ([right_column_x + 48, 214, 208, 56], 0)),
        ("query-card", ([right_column_x, 104, 280, 200], 24)),
    ]))
}
