use std::collections::BTreeMap;

use super::{
    model::PlatformPulseVisualContractManifest, validation::PlatformPulseVisualContractFailure,
};

pub(super) fn validate(
    manifest: &PlatformPulseVisualContractManifest,
    tokens: &BTreeMap<String, [u8; 4]>,
) -> Result<(), PlatformPulseVisualContractFailure> {
    for layout in &manifest.layouts {
        let expected = expected_points(layout.logical_client_extent)
            .ok_or(PlatformPulseVisualContractFailure::ControlPoint)?;
        let actual = layout
            .control_points
            .iter()
            .map(|point| {
                (
                    point.identity.as_str(),
                    (point.logical_point, point.token_role.as_str()),
                )
            })
            .collect::<BTreeMap<_, _>>();
        if layout.control_points.len() != 10
            || actual.len() != layout.control_points.len()
            || actual != expected
            || layout
                .control_points
                .iter()
                .any(|point| !tokens.contains_key(&point.token_role))
        {
            return Err(PlatformPulseVisualContractFailure::ControlPoint);
        }
    }
    Ok(())
}

fn expected_points(extent: [u32; 2]) -> Option<BTreeMap<&'static str, ([u32; 2], &'static str)>> {
    let (source_y, right_x) = match extent {
        [960, 600] => (486, 664),
        [1_120, 700] => (586, 824),
        _ => return None,
    };
    Some(BTreeMap::from([
        ("canvas", ([8, 8], "canvas")),
        ("evidence-tile", ([32, 112], "raised-surface")),
        ("hero-tile", ([272, 112], "elevated-surface")),
        ("live-action", ([304, 424], "action-fill")),
        ("masthead-canvas", ([32, 32], "canvas")),
        ("native-tile", ([right_x, 336], "raised-surface")),
        ("query-control", ([right_x + 24, 184], "elevated-surface")),
        ("query-tile", ([right_x, 112], "raised-surface")),
        ("status-divider", ([32, extent[1] - 48], "structural-rule")),
        (
            "source-signal-active",
            ([64, source_y], "source-signal-blue"),
        ),
    ]))
}
