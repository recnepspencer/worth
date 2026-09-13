use std::collections::{BTreeMap, BTreeSet};

use super::{
    model::{contains, overlaps, PlatformPulseVisualContractManifest},
    validation::PlatformPulseVisualContractFailure,
};

const DEFAULT_EXTENT: [u32; 2] = [960, 600];
const RESIZED_EXTENT: [u32; 2] = [1_120, 700];

pub(super) fn validate(
    manifest: &PlatformPulseVisualContractManifest,
    layouts: &BTreeMap<String, BTreeMap<String, [u32; 4]>>,
    typography: &BTreeSet<String>,
    tokens: &BTreeMap<String, [u8; 4]>,
) -> Result<(), PlatformPulseVisualContractFailure> {
    validate_text(manifest, layouts, typography)?;
    validate_targets(manifest, layouts, tokens)
}

fn validate_text(
    manifest: &PlatformPulseVisualContractManifest,
    layouts: &BTreeMap<String, BTreeMap<String, [u32; 4]>>,
    typography: &BTreeSet<String>,
) -> Result<(), PlatformPulseVisualContractFailure> {
    for layout in &manifest.layouts {
        let Some(regions) = layouts.get(&layout.name) else {
            return Err(PlatformPulseVisualContractFailure::TextContainment);
        };
        if layout.text_bounds.len() != 19 {
            return Err(PlatformPulseVisualContractFailure::TextContainment);
        }
        let mut identities = BTreeSet::new();
        for (index, text) in layout.text_bounds.iter().enumerate() {
            let Some(region) = regions.get(&text.region) else {
                return Err(PlatformPulseVisualContractFailure::TextContainment);
            };
            if !identities.insert(text.identity.as_str())
                || !typography.contains(&text.role)
                || !line_capacity_is_honest(text.role.as_str(), text.maximum_lines, text.rect[3])
                || !contains(*region, text.rect)
                || layout.text_bounds[index + 1..]
                    .iter()
                    .any(|other| overlaps(text.rect, other.rect))
            {
                return Err(PlatformPulseVisualContractFailure::TextContainment);
            }
        }
    }
    Ok(())
}

fn line_capacity_is_honest(role: &str, maximum_lines: u32, height: u32) -> bool {
    let line_height: u32 = match role {
        "display" => 52,
        "masthead" | "body" | "action" => 20,
        "section" | "meta" => 16,
        _ => return false,
    };
    if maximum_lines == 0 {
        return false;
    }
    let safety: u32 = if maximum_lines > 1 { 4 } else { 0 };
    line_height
        .checked_mul(maximum_lines)
        .and_then(|needed| needed.checked_add(safety))
        .is_some_and(|needed| height >= needed)
}

fn validate_targets(
    manifest: &PlatformPulseVisualContractManifest,
    layouts: &BTreeMap<String, BTreeMap<String, [u32; 4]>>,
    tokens: &BTreeMap<String, [u8; 4]>,
) -> Result<(), PlatformPulseVisualContractFailure> {
    for layout in &manifest.layouts {
        let stage = layouts
            .get(&layout.name)
            .and_then(|regions| regions.get("platform.pulse.region.service_stage"))
            .ok_or(PlatformPulseVisualContractFailure::Target)?;
        let expected = expected_targets(layout.logical_client_extent)?;
        let actual = layout
            .minimum_targets
            .iter()
            .map(|target| {
                (
                    target.identity.as_str(),
                    target.action_identity.as_str(),
                    target.label_identity.as_str(),
                    target.token_role.as_str(),
                    target.rect,
                )
            })
            .collect::<BTreeSet<_>>();
        let mut identities = BTreeSet::new();
        if layout.minimum_targets.len() != 3
            || actual != expected
            || !layout.minimum_targets.iter().all(|target| {
                let label = layout
                    .text_bounds
                    .iter()
                    .find(|text| text.identity == target.label_identity);
                identities.insert(target.identity.as_str())
                    && target.visibility_gate == "real-service-provider-installed"
                    && target.minimum_extent == [32, 32]
                    && target.rect[2] >= target.minimum_extent[0]
                    && target.rect[3] >= target.minimum_extent[1]
                    && contains(*stage, target.rect)
                    && tokens.contains_key(&target.token_role)
                    && label.is_some_and(|label| contains(target.rect, label.rect))
            })
        {
            return Err(PlatformPulseVisualContractFailure::Target);
        }
    }
    Ok(())
}

fn expected_targets(
    extent: [u32; 2],
) -> Result<
    BTreeSet<(
        &'static str,
        &'static str,
        &'static str,
        &'static str,
        [u32; 4],
    )>,
    PlatformPulseVisualContractFailure,
> {
    let confirmation = match extent {
        DEFAULT_EXTENT => [680, 176, 232, 104],
        RESIZED_EXTENT => [840, 176, 232, 104],
        _ => return Err(PlatformPulseVisualContractFailure::Target),
    };
    Ok(BTreeSet::from([
        (
            "platform.pulse.target.run_live_action",
            "intent:platform.pulse.action.route:activate",
            "platform.pulse.text.action_label",
            "action-fill",
            [296, 416, 216, 48],
        ),
        (
            "platform.pulse.target.open_portal",
            "intent:platform.pulse.portal.open.route:activate",
            "platform.pulse.text.portal_label",
            "elevated-surface",
            [528, 416, 112, 48],
        ),
        (
            "platform.pulse.target.confirm_live_action",
            "intent:platform.pulse.action.route:confirm",
            "platform.pulse.text.projected_status",
            "elevated-surface",
            confirmation,
        ),
    ]))
}
