use std::collections::{BTreeMap, BTreeSet};

use super::{contrast, model::*};

const SCHEMA: &str = "platform-pulse-visual-contract-v2";
const DEFAULT_EXTENT: [u32; 2] = [960, 600];
const RESIZED_EXTENT: [u32; 2] = [1_120, 700];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlatformPulseVisualContractFailure {
    Decode,
    Identity,
    Geometry,
    SemanticTruth,
    Token,
    Typography,
    Contrast,
    TextContainment,
    Target,
    ControlPoint,
    CaptureBudget,
}

pub(super) fn validate(
    manifest: &PlatformPulseVisualContractManifest,
) -> Result<(), PlatformPulseVisualContractFailure> {
    validate_identity(manifest)?;
    let layouts = validate_geometry(manifest)?;
    validate_semantic_truth(manifest, &layouts)?;
    let tokens = validate_tokens(manifest)?;
    let typography = validate_typography(manifest)?;
    validate_contrast(manifest, &tokens)?;
    super::interactive_content_validation::validate(manifest, &layouts, &typography, &tokens)?;
    super::control_point_validation::validate(manifest, &tokens)?;
    super::pixel_region_validation::validate(manifest)?;
    validate_inspection(manifest)?;
    validate_capture_budget(manifest)
}

fn validate_identity(
    manifest: &PlatformPulseVisualContractManifest,
) -> Result<(), PlatformPulseVisualContractFailure> {
    (manifest.schema == SCHEMA
        && manifest.identity == "platform-pulse-3.15-independent-visual-oracle-v1")
        .then_some(())
        .ok_or(PlatformPulseVisualContractFailure::Identity)
}

fn validate_geometry(
    manifest: &PlatformPulseVisualContractManifest,
) -> Result<BTreeMap<String, BTreeMap<String, [u32; 4]>>, PlatformPulseVisualContractFailure> {
    if manifest.spacing.outer_gutter != 24
        || manifest.spacing.rhythm != 8
        || manifest.spacing.region_gap != 24
        || manifest.layouts.len() != 2
    {
        return Err(PlatformPulseVisualContractFailure::Geometry);
    }
    let mut layouts = BTreeMap::new();
    let mut layout_names = BTreeSet::new();
    for layout in &manifest.layouts {
        if !layout_names.insert(layout.name.as_str())
            || ![DEFAULT_EXTENT, RESIZED_EXTENT].contains(&layout.logical_client_extent)
            || layout.regions.len() != 4
        {
            return Err(PlatformPulseVisualContractFailure::Geometry);
        }
        let mut region_names = BTreeSet::new();
        let expected = expected_regions(layout.logical_client_extent)
            .ok_or(PlatformPulseVisualContractFailure::Geometry)?;
        for (index, region) in layout.regions.iter().enumerate() {
            if !region_names.insert(region.identity.as_str())
                || region.hit_test
                || !inside(region.rect, layout.logical_client_extent)
                || expected.get(region.identity.as_str()) != Some(&region.rect)
                || layout.regions[index + 1..]
                    .iter()
                    .any(|other| overlaps(region.rect, other.rect))
            {
                return Err(PlatformPulseVisualContractFailure::Geometry);
            }
        }
        if layouts
            .insert(layout.name.clone(), expected_regions_owned(expected))
            .is_some()
        {
            return Err(PlatformPulseVisualContractFailure::Geometry);
        }
    }
    (layout_names.contains("default")
        && layout_names.contains("resized")
        && layouts.values().all(|regions| regions.len() == 4))
    .then_some(layouts)
    .ok_or(PlatformPulseVisualContractFailure::Geometry)
}

fn expected_regions_owned(regions: BTreeMap<&'static str, [u32; 4]>) -> BTreeMap<String, [u32; 4]> {
    regions
        .into_iter()
        .map(|(identity, rect)| (identity.to_owned(), rect))
        .collect()
}

fn expected_regions(extent: [u32; 2]) -> Option<BTreeMap<&'static str, [u32; 4]>> {
    let rows = match extent {
        DEFAULT_EXTENT => [
            ("platform.pulse.region.identity_masthead", [24, 24, 912, 56]),
            ("platform.pulse.region.evidence_rail", [24, 104, 216, 424]),
            ("platform.pulse.region.service_stage", [264, 104, 672, 424]),
            (
                "platform.pulse.region.truthful_status_band",
                [24, 552, 912, 24],
            ),
        ],
        RESIZED_EXTENT => [
            (
                "platform.pulse.region.identity_masthead",
                [24, 24, 1_072, 56],
            ),
            ("platform.pulse.region.evidence_rail", [24, 104, 216, 524]),
            ("platform.pulse.region.service_stage", [264, 104, 832, 524]),
            (
                "platform.pulse.region.truthful_status_band",
                [24, 652, 1_072, 24],
            ),
        ],
        _ => return None,
    };
    Some(rows.into_iter().collect())
}

fn validate_semantic_truth(
    manifest: &PlatformPulseVisualContractManifest,
    layouts: &BTreeMap<String, BTreeMap<String, [u32; 4]>>,
) -> Result<(), PlatformPulseVisualContractFailure> {
    let regions = layouts
        .get("default")
        .ok_or(PlatformPulseVisualContractFailure::SemanticTruth)?;
    let expected = BTreeMap::from([
        (
            "platform.pulse.region.identity_masthead",
            BTreeSet::from(["application-identity", "native-process"]),
        ),
        (
            "platform.pulse.region.evidence_rail",
            BTreeSet::from([
                "intent-posture",
                "query-projection",
                "native-publication",
                "source-generation",
            ]),
        ),
        (
            "platform.pulse.region.service_stage",
            BTreeSet::from(["intent-provider", "native-process", "query-projection"]),
        ),
        (
            "platform.pulse.region.truthful_status_band",
            BTreeSet::from(["native-publication"]),
        ),
    ]);
    let mut actual = BTreeMap::new();
    let valid = manifest.semantic_identities.len() == regions.len()
        && manifest.semantic_identities.iter().all(|semantic| {
            let sources = semantic
                .truth_sources
                .iter()
                .map(String::as_str)
                .collect::<BTreeSet<_>>();
            sources.len() == semantic.truth_sources.len()
                && actual.insert(semantic.identity.as_str(), sources).is_none()
        })
        && actual == expected;
    valid
        .then_some(())
        .ok_or(PlatformPulseVisualContractFailure::SemanticTruth)
}

fn validate_tokens(
    manifest: &PlatformPulseVisualContractManifest,
) -> Result<BTreeMap<String, [u8; 4]>, PlatformPulseVisualContractFailure> {
    let mut tokens = BTreeMap::new();
    for token in &manifest.tokens {
        if token.role.is_empty()
            || token.rgba[3] != 255
            || tokens.insert(token.role.clone(), token.rgba).is_some()
        {
            return Err(PlatformPulseVisualContractFailure::Token);
        }
    }
    let expected = BTreeMap::from([
        ("action-fill".into(), [148, 64, 212, 255]),
        ("action-text".into(), [250, 251, 252, 255]),
        ("canvas".into(), [11, 15, 20, 255]),
        ("raised-surface".into(), [17, 22, 28, 255]),
        ("elevated-surface".into(), [23, 29, 37, 255]),
        ("structural-rule".into(), [95, 105, 119, 255]),
        ("primary-text".into(), [242, 244, 247, 255]),
        ("secondary-text".into(), [161, 169, 180, 255]),
        ("principal-accent".into(), [172, 103, 242, 255]),
        ("visual-inspection-overlay".into(), [255, 0, 255, 255]),
        ("positive".into(), [92, 201, 120, 255]),
        ("caution".into(), [224, 173, 98, 255]),
        ("source-signal-blue".into(), [47, 129, 247, 255]),
        ("source-signal-green".into(), [63, 185, 80, 255]),
    ]);
    (tokens == expected)
        .then_some(tokens)
        .ok_or(PlatformPulseVisualContractFailure::Token)
}

fn validate_typography(
    manifest: &PlatformPulseVisualContractManifest,
) -> Result<BTreeSet<String>, PlatformPulseVisualContractFailure> {
    let expected = BTreeSet::from([
        ("action", 13, 20, 600),
        ("body", 13, 20, 450),
        ("display", 44, 52, 500),
        ("masthead", 16, 20, 600),
        ("meta", 12, 16, 500),
        ("section", 11, 16, 650),
    ]);
    let actual = manifest
        .typography
        .iter()
        .map(|style| {
            (
                style.role.as_str(),
                style.size,
                style.line_height,
                style.weight,
            )
        })
        .collect::<BTreeSet<_>>();
    (actual == expected && actual.len() == manifest.typography.len())
        .then(|| actual.into_iter().map(|style| style.0.to_owned()).collect())
        .ok_or(PlatformPulseVisualContractFailure::Typography)
}

fn validate_contrast(
    manifest: &PlatformPulseVisualContractManifest,
    tokens: &BTreeMap<String, [u8; 4]>,
) -> Result<(), PlatformPulseVisualContractFailure> {
    let expected = BTreeSet::from([
        ("primary-text", "action-fill", 4_500),
        ("action-text", "action-fill", 4_500),
        ("caution", "raised-surface", 4_500),
        ("caution", "elevated-surface", 4_500),
        ("positive", "canvas", 4_500),
        ("positive", "raised-surface", 4_500),
        ("positive", "elevated-surface", 4_500),
        ("primary-text", "canvas", 4_500),
        ("primary-text", "raised-surface", 4_500),
        ("primary-text", "elevated-surface", 4_500),
        ("principal-accent", "canvas", 4_500),
        ("secondary-text", "canvas", 4_500),
        ("secondary-text", "raised-surface", 4_500),
        ("secondary-text", "elevated-surface", 4_500),
        ("structural-rule", "canvas", 3_000),
        ("structural-rule", "raised-surface", 3_000),
        ("structural-rule", "elevated-surface", 3_000),
    ]);
    let actual = manifest
        .contrast_pairs
        .iter()
        .map(|pair| {
            (
                pair.foreground.as_str(),
                pair.background.as_str(),
                pair.minimum_ratio_milli,
            )
        })
        .collect::<BTreeSet<_>>();
    (actual == expected
        && actual.len() == manifest.contrast_pairs.len()
        && manifest.contrast_pairs.iter().all(|pair| {
            let Some(foreground) = tokens.get(&pair.foreground) else {
                return false;
            };
            let Some(background) = tokens.get(&pair.background) else {
                return false;
            };
            contrast::ratio_milli(*foreground, *background) >= pair.minimum_ratio_milli
        }))
    .then_some(())
    .ok_or(PlatformPulseVisualContractFailure::Contrast)
}

fn validate_capture_budget(
    manifest: &PlatformPulseVisualContractManifest,
) -> Result<(), PlatformPulseVisualContractFailure> {
    let limits = &manifest.limits;
    let calculated = u64::from(limits.maximum_capture_extent[0])
        * u64::from(limits.maximum_capture_extent[1])
        * u64::from(limits.maximum_capture_scale).pow(2)
        * 4;
    (limits.maximum_mounted_nodes == 128
        && limits.maximum_capture_scale == 4
        && limits.maximum_capture_extent == RESIZED_EXTENT
        && limits.maximum_capture_rgba_bytes == calculated
        && limits.maximum_retained_capture_rgba_bytes == 2 * calculated
        && limits.channel_tolerance > 0)
        .then_some(())
        .ok_or(PlatformPulseVisualContractFailure::CaptureBudget)
}

fn validate_inspection(
    manifest: &PlatformPulseVisualContractManifest,
) -> Result<(), PlatformPulseVisualContractFailure> {
    (manifest.inspection.visible_region_count == 10
        && manifest.inspection.hit_test_region_count == 4
        && manifest.inspection.target_authored_name
            == "component:platform.pulse.component.identity_target")
        .then_some(())
        .ok_or(PlatformPulseVisualContractFailure::ControlPoint)
}
