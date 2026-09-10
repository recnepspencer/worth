use serde::Deserialize;

const SOURCE: &str = include_str!("platform_pulse_control_points.json");
const SCHEMA: &str = "platform-pulse-control-points-v1";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(super) struct PlatformPulseControlPointManifest {
    schema: String,
    identity: String,
    logical_client_extent: [u32; 2],
    background_logical_point: [u32; 2],
    source_signal_logical_point: [u32; 2],
    target_logical_point: [u32; 2],
    target_region: [u32; 4],
    portal_overlay_region: [u32; 4],
    portal_accent_region: [u32; 4],
    portal_icon_tile_region: [u32; 4],
    portal_icon_text_region: [u32; 4],
    portal_title_region: [u32; 4],
    portal_body_region: [u32; 4],
    portal_cancel_region: [u32; 4],
    portal_cancel_label_region: [u32; 4],
    portal_primary_region: [u32; 4],
    portal_primary_label_region: [u32; 4],
    schema_stable_control_region: [u32; 4],
    schema_posture_region: [u32; 4],
    brand_region: [u32; 4],
    query_card_region: [u32; 4],
    query_card_radius: u32,
    blue_rgba: [u8; 4],
    green_rgba: [u8; 4],
    canvas_rgba: [u8; 4],
    target_rgba: [u8; 4],
    overlay_rgba: [u8; 4],
    portal_overlay_rgba: [u8; 4],
    raised_surface_rgba: [u8; 4],
    structural_rule_rgba: [u8; 4],
    principal_accent_rgba: [u8; 4],
    primary_text_rgba: [u8; 4],
    secondary_text_rgba: [u8; 4],
    action_text_rgba: [u8; 4],
    channel_tolerance: u8,
    maximum_capture_scale: u32,
    maximum_pixel_bytes: u64,
    visible_region_count: u64,
    hit_test_region_count: u64,
    target_authored_name: String,
    environment: ControlPointEnvironment,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
struct ControlPointEnvironment {
    dpi: u32,
    text_profile: String,
    font_asset_sha256: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlatformPulseControlPointManifestFailure {
    Decode,
    Incomplete,
}

pub(super) fn checked_in(
) -> Result<PlatformPulseControlPointManifest, PlatformPulseControlPointManifestFailure> {
    let manifest: PlatformPulseControlPointManifest = serde_json::from_str(SOURCE)
        .map_err(|_| PlatformPulseControlPointManifestFailure::Decode)?;
    let complete = manifest.schema == SCHEMA
        && !manifest.identity.is_empty()
        && manifest
            .logical_client_extent
            .iter()
            .all(|value| *value > 0)
        && inside(
            manifest.background_logical_point,
            manifest.logical_client_extent,
        )
        && inside(
            manifest.source_signal_logical_point,
            manifest.logical_client_extent,
        )
        && inside(
            manifest.target_logical_point,
            manifest.logical_client_extent,
        )
        && valid_region(manifest.target_region, manifest.logical_client_extent)
        && valid_region(
            manifest.portal_overlay_region,
            manifest.logical_client_extent,
        )
        && [
            manifest.portal_accent_region,
            manifest.portal_icon_tile_region,
            manifest.portal_icon_text_region,
            manifest.portal_title_region,
            manifest.portal_body_region,
            manifest.portal_cancel_region,
            manifest.portal_cancel_label_region,
            manifest.portal_primary_region,
            manifest.portal_primary_label_region,
        ]
        .into_iter()
        .all(|region| {
            valid_region(region, manifest.logical_client_extent)
                && contains(manifest.portal_overlay_region, region)
        })
        && valid_region(
            manifest.schema_stable_control_region,
            manifest.logical_client_extent,
        )
        && valid_region(
            manifest.schema_posture_region,
            manifest.logical_client_extent,
        )
        && valid_region(manifest.brand_region, manifest.logical_client_extent)
        && valid_region(manifest.query_card_region, manifest.logical_client_extent)
        && manifest.query_card_radius > 0
        && manifest.blue_rgba[3] == 255
        && manifest.green_rgba[3] == 255
        && manifest.canvas_rgba[3] == 255
        && manifest.target_rgba[3] == 255
        && manifest.overlay_rgba[3] == 255
        && manifest.portal_overlay_rgba[3] > 0
        && manifest.raised_surface_rgba[3] == 255
        && manifest.structural_rule_rgba[3] == 255
        && manifest.principal_accent_rgba[3] == 255
        && manifest.primary_text_rgba[3] == 255
        && manifest.secondary_text_rgba[3] == 255
        && manifest.action_text_rgba[3] == 255
        && manifest.channel_tolerance > 0
        && manifest.maximum_capture_scale > 0
        && manifest.maximum_pixel_bytes > 0
        && manifest.visible_region_count > 0
        && manifest.hit_test_region_count > 0
        && !manifest.target_authored_name.is_empty()
        && manifest.environment.dpi > 0
        && !manifest.environment.text_profile.is_empty()
        && manifest.environment.font_asset_sha256.len() == 64;
    complete
        .then_some(manifest)
        .ok_or(PlatformPulseControlPointManifestFailure::Incomplete)
}

fn inside(point: [u32; 2], extent: [u32; 2]) -> bool {
    point[0] < extent[0] && point[1] < extent[1]
}

fn valid_region(region: [u32; 4], extent: [u32; 2]) -> bool {
    region[0] < region[2]
        && region[1] < region[3]
        && region[2] <= extent[0]
        && region[3] <= extent[1]
}

fn contains(outer: [u32; 4], inner: [u32; 4]) -> bool {
    inner[0] >= outer[0] && inner[1] >= outer[1] && inner[2] <= outer[2] && inner[3] <= outer[3]
}

macro_rules! accessors {
    ($($name:ident : $kind:ty),+ $(,)?) => {
        impl PlatformPulseControlPointManifest {
            $(pub(super) fn $name(&self) -> $kind { self.$name })+
        }
    };
}

accessors!(
    logical_client_extent: [u32; 2],
    background_logical_point: [u32; 2],
    source_signal_logical_point: [u32; 2],
    target_logical_point: [u32; 2],
    target_region: [u32; 4],
    portal_overlay_region: [u32; 4],
    portal_accent_region: [u32; 4],
    portal_icon_tile_region: [u32; 4],
    portal_icon_text_region: [u32; 4],
    portal_title_region: [u32; 4],
    portal_body_region: [u32; 4],
    portal_cancel_region: [u32; 4],
    portal_cancel_label_region: [u32; 4],
    portal_primary_region: [u32; 4],
    portal_primary_label_region: [u32; 4],
    schema_stable_control_region: [u32; 4],
    schema_posture_region: [u32; 4],
    brand_region: [u32; 4],
    query_card_region: [u32; 4],
    query_card_radius: u32,
    blue_rgba: [u8; 4],
    green_rgba: [u8; 4],
    canvas_rgba: [u8; 4],
    target_rgba: [u8; 4],
    overlay_rgba: [u8; 4],
    portal_overlay_rgba: [u8; 4],
    raised_surface_rgba: [u8; 4],
    structural_rule_rgba: [u8; 4],
    principal_accent_rgba: [u8; 4],
    primary_text_rgba: [u8; 4],
    secondary_text_rgba: [u8; 4],
    action_text_rgba: [u8; 4],
    channel_tolerance: u8,
    maximum_capture_scale: u32,
    maximum_pixel_bytes: u64,
    visible_region_count: u64,
    hit_test_region_count: u64,
);

impl PlatformPulseControlPointManifest {
    pub(super) fn target_authored_name(&self) -> &str {
        &self.target_authored_name
    }
}
