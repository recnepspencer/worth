use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(super) struct PlatformPulseVisualContractManifest {
    pub(super) schema: String,
    pub(super) identity: String,
    pub(super) spacing: SpacingContract,
    pub(super) layouts: Vec<LayoutContract>,
    pub(super) semantic_identities: Vec<SemanticIdentityContract>,
    pub(super) tokens: Vec<TokenContract>,
    pub(super) typography: Vec<TypographyContract>,
    pub(super) contrast_pairs: Vec<ContrastPairContract>,
    pub(super) limits: VisualLimitsContract,
    pub(super) inspection: VisualInspectionContract,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(super) struct SpacingContract {
    pub(super) outer_gutter: u32,
    pub(super) rhythm: u32,
    pub(super) region_gap: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(super) struct LayoutContract {
    pub(super) name: String,
    pub(super) logical_client_extent: [u32; 2],
    pub(super) regions: Vec<RegionContract>,
    pub(super) text_bounds: Vec<TextBoundsContract>,
    pub(super) minimum_targets: Vec<TargetContract>,
    pub(super) control_points: Vec<ControlPointContract>,
    pub(super) pixel_regions: Vec<PixelRegionContract>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(super) struct RegionContract {
    pub(super) identity: String,
    pub(super) rect: [u32; 4],
    pub(super) hit_test: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(super) struct SemanticIdentityContract {
    pub(super) identity: String,
    pub(super) truth_sources: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(super) struct TokenContract {
    pub(super) role: String,
    pub(super) rgba: [u8; 4],
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(super) struct TypographyContract {
    pub(super) role: String,
    pub(super) size: u32,
    pub(super) line_height: u32,
    pub(super) weight: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(super) struct ContrastPairContract {
    pub(super) foreground: String,
    pub(super) background: String,
    pub(super) minimum_ratio_milli: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(super) struct TextBoundsContract {
    pub(super) identity: String,
    pub(super) region: String,
    pub(super) role: String,
    pub(super) maximum_lines: u32,
    pub(super) rect: [u32; 4],
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(super) struct TargetContract {
    pub(super) identity: String,
    pub(super) action_identity: String,
    pub(super) label_identity: String,
    pub(super) token_role: String,
    pub(super) rect: [u32; 4],
    pub(super) minimum_extent: [u32; 2],
    pub(super) visibility_gate: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(super) struct ControlPointContract {
    pub(super) identity: String,
    pub(super) logical_point: [u32; 2],
    pub(super) token_role: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(super) struct PixelRegionContract {
    pub(super) identity: String,
    pub(super) rect: [u32; 4],
    pub(super) corner_radius: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(super) struct VisualLimitsContract {
    pub(super) maximum_mounted_nodes: u32,
    pub(super) maximum_capture_scale: u32,
    pub(super) maximum_capture_extent: [u32; 2],
    pub(super) maximum_capture_rgba_bytes: u64,
    pub(super) maximum_retained_capture_rgba_bytes: u64,
    pub(super) channel_tolerance: u8,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(super) struct VisualInspectionContract {
    pub(super) visible_region_count: u64,
    pub(super) hit_test_region_count: u64,
    pub(super) target_authored_name: String,
}

pub(super) fn inside(rect: [u32; 4], extent: [u32; 2]) -> bool {
    rect[2] > 0
        && rect[3] > 0
        && rect[0]
            .checked_add(rect[2])
            .is_some_and(|right| right <= extent[0])
        && rect[1]
            .checked_add(rect[3])
            .is_some_and(|bottom| bottom <= extent[1])
}

pub(super) fn contains(outer: [u32; 4], inner: [u32; 4]) -> bool {
    let Some(outer_right) = outer[0].checked_add(outer[2]) else {
        return false;
    };
    let Some(outer_bottom) = outer[1].checked_add(outer[3]) else {
        return false;
    };
    let Some(inner_right) = inner[0].checked_add(inner[2]) else {
        return false;
    };
    let Some(inner_bottom) = inner[1].checked_add(inner[3]) else {
        return false;
    };
    inner[2] > 0
        && inner[3] > 0
        && inner[0] >= outer[0]
        && inner[1] >= outer[1]
        && inner_right <= outer_right
        && inner_bottom <= outer_bottom
}

pub(super) fn overlaps(left: [u32; 4], right: [u32; 4]) -> bool {
    let Some(left_right) = left[0].checked_add(left[2]) else {
        return true;
    };
    let Some(left_bottom) = left[1].checked_add(left[3]) else {
        return true;
    };
    let Some(right_right) = right[0].checked_add(right[2]) else {
        return true;
    };
    let Some(right_bottom) = right[1].checked_add(right[3]) else {
        return true;
    };
    left[0] < right_right
        && left_right > right[0]
        && left[1] < right_bottom
        && left_bottom > right[1]
}
