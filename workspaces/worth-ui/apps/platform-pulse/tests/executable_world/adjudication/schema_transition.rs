use std::fmt;

use worth_ui_platform_pulse::observation_contract::{
    PlatformPulseProjectionSchemaField, PlatformPulseProjectionSchemaTransitionKind,
    PlatformPulseProjectionSchemaTransitionObservation, PlatformPulseQueryProjectionEvidence,
    PlatformPulseQueryProjectionPosture,
};

use crate::external_observation::NativeClientPixelCapture;

use super::ExecutableReplacementEvidence;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ExpectedSchemaTransition {
    Stopped,
    Recovered,
}

#[derive(Debug)]
pub(crate) struct ExecutableSchemaTransitionEvidence<Kind> {
    replacement: ExecutableReplacementEvidence<Kind>,
    transition: PlatformPulseProjectionSchemaTransitionObservation,
    query_basis: PlatformPulseQueryProjectionEvidence,
    retained_control_pixel_bytes: usize,
    changed_posture_pixel_bytes: usize,
    canonical_current_restored: bool,
}

#[derive(Debug)]
pub(crate) enum ExecutableSchemaTransitionFailure {
    MissingTransition,
    WrongTransitionKind,
    WrongFieldProgression,
    PredecessorValueNotPreserved,
    QueryValueNotCurrent,
    QueryOwnerDidNotReachSecondCurrent,
    QueryGenerationMissing,
    CaptureExtentChanged,
    VisualContract,
    StableControlRegionChanged { differing_bytes: usize },
    PostureRegionDidNotChange,
    CanonicalCurrentPostureNotRestored,
}

pub(crate) fn adjudicate_schema_transition<Kind>(
    replacement: ExecutableReplacementEvidence<Kind>,
    predecessor_pixels: &NativeClientPixelCapture,
    canonical_current_pixels: Option<&NativeClientPixelCapture>,
    query_basis: &PlatformPulseQueryProjectionEvidence,
    expected: ExpectedSchemaTransition,
) -> Result<ExecutableSchemaTransitionEvidence<Kind>, ExecutableSchemaTransitionFailure> {
    let transition = replacement
        .replacement()
        .schema_transition()
        .copied()
        .ok_or(ExecutableSchemaTransitionFailure::MissingTransition)?;
    require_transition(transition, expected)?;
    require_query_basis(query_basis)?;
    let (retained_control_pixel_bytes, changed_posture_pixel_bytes) =
        require_visible_preservation(predecessor_pixels, replacement.pixels())?;
    let canonical_current_restored = match (expected, canonical_current_pixels) {
        (ExpectedSchemaTransition::Recovered, Some(canonical)) => {
            if !schema_posture_matches(canonical, replacement.pixels())? {
                return Err(ExecutableSchemaTransitionFailure::CanonicalCurrentPostureNotRestored);
            }
            true
        }
        (ExpectedSchemaTransition::Recovered, None) => {
            return Err(ExecutableSchemaTransitionFailure::CanonicalCurrentPostureNotRestored)
        }
        (ExpectedSchemaTransition::Stopped, _) => false,
    };
    Ok(ExecutableSchemaTransitionEvidence {
        replacement,
        transition,
        query_basis: query_basis.clone(),
        retained_control_pixel_bytes,
        changed_posture_pixel_bytes,
        canonical_current_restored,
    })
}

fn require_transition(
    transition: PlatformPulseProjectionSchemaTransitionObservation,
    expected: ExpectedSchemaTransition,
) -> Result<(), ExecutableSchemaTransitionFailure> {
    let expected_fields = match expected {
        ExpectedSchemaTransition::Stopped => (
            PlatformPulseProjectionSchemaTransitionKind::Stopped,
            PlatformPulseProjectionSchemaField::Status,
            PlatformPulseProjectionSchemaField::Revision,
        ),
        ExpectedSchemaTransition::Recovered => (
            PlatformPulseProjectionSchemaTransitionKind::Recovered,
            PlatformPulseProjectionSchemaField::Revision,
            PlatformPulseProjectionSchemaField::Status,
        ),
    };
    if transition.kind() != expected_fields.0 {
        return Err(ExecutableSchemaTransitionFailure::WrongTransitionKind);
    }
    if transition.predecessor_selected_field() != expected_fields.1
        || transition.candidate_selected_field() != expected_fields.2
        || transition.installed_selected_field() != PlatformPulseProjectionSchemaField::Status
    {
        return Err(ExecutableSchemaTransitionFailure::WrongFieldProgression);
    }
    if !transition.predecessor_value_preserved() {
        return Err(ExecutableSchemaTransitionFailure::PredecessorValueNotPreserved);
    }
    Ok(())
}

fn require_query_basis(
    query: &PlatformPulseQueryProjectionEvidence,
) -> Result<(), ExecutableSchemaTransitionFailure> {
    if query.posture() != PlatformPulseQueryProjectionPosture::Current
        || query.native_value() != Some("SYNCHRONIZED")
    {
        return Err(ExecutableSchemaTransitionFailure::QueryValueNotCurrent);
    }
    if query.owner_order() != 5 {
        return Err(ExecutableSchemaTransitionFailure::QueryOwnerDidNotReachSecondCurrent);
    }
    if query.source_generation().is_empty() || query.result_generation().is_empty() {
        return Err(ExecutableSchemaTransitionFailure::QueryGenerationMissing);
    }
    Ok(())
}

fn require_visible_preservation(
    predecessor: &NativeClientPixelCapture,
    successor: &NativeClientPixelCapture,
) -> Result<(usize, usize), ExecutableSchemaTransitionFailure> {
    if predecessor.width() != successor.width() || predecessor.height() != successor.height() {
        return Err(ExecutableSchemaTransitionFailure::CaptureExtentChanged);
    }
    let manifest = super::visual_contract_manifest::checked_in_adjudication_contract()
        .map_err(|_| ExecutableSchemaTransitionFailure::VisualContract)?;
    let stable_control_region = scaled_region(
        manifest.schema_stable_control_region(),
        manifest.logical_client_extent(),
        successor,
    )?;
    let predecessor_control = region_bytes(predecessor, stable_control_region)?;
    let successor_control = region_bytes(successor, stable_control_region)?;
    let differing_control_bytes = predecessor_control
        .iter()
        .zip(&successor_control)
        .filter(|(predecessor, successor)| predecessor != successor)
        .count();
    if differing_control_bytes != 0 {
        return Err(
            ExecutableSchemaTransitionFailure::StableControlRegionChanged {
                differing_bytes: differing_control_bytes,
            },
        );
    }
    let changed_posture_pixel_bytes = schema_posture_changed_pixel_bytes(predecessor, successor)?;
    if changed_posture_pixel_bytes == 0 {
        return Err(ExecutableSchemaTransitionFailure::PostureRegionDidNotChange);
    }
    Ok((successor_control.len(), changed_posture_pixel_bytes))
}

pub(crate) fn schema_posture_changed_pixel_bytes(
    predecessor: &NativeClientPixelCapture,
    successor: &NativeClientPixelCapture,
) -> Result<usize, ExecutableSchemaTransitionFailure> {
    if predecessor.width() != successor.width() || predecessor.height() != successor.height() {
        return Err(ExecutableSchemaTransitionFailure::CaptureExtentChanged);
    }
    let manifest = super::visual_contract_manifest::checked_in_adjudication_contract()
        .map_err(|_| ExecutableSchemaTransitionFailure::VisualContract)?;
    let posture_region = scaled_region(
        manifest.schema_posture_region(),
        manifest.logical_client_extent(),
        successor,
    )?;
    let predecessor_posture = region_bytes(predecessor, posture_region)?;
    let successor_posture = region_bytes(successor, posture_region)?;
    Ok(predecessor_posture
        .iter()
        .zip(&successor_posture)
        .filter(|(predecessor, successor)| predecessor != successor)
        .count())
}

pub(crate) fn schema_posture_matches(
    expected: &NativeClientPixelCapture,
    observed: &NativeClientPixelCapture,
) -> Result<bool, ExecutableSchemaTransitionFailure> {
    if expected.width() != observed.width() || expected.height() != observed.height() {
        return Err(ExecutableSchemaTransitionFailure::CaptureExtentChanged);
    }
    let manifest = super::visual_contract_manifest::checked_in_adjudication_contract()
        .map_err(|_| ExecutableSchemaTransitionFailure::VisualContract)?;
    let posture_region = scaled_region(
        manifest.schema_posture_region(),
        manifest.logical_client_extent(),
        observed,
    )?;
    Ok(region_bytes(expected, posture_region)? == region_bytes(observed, posture_region)?)
}

fn scaled_region(
    logical: [u32; 4],
    logical_extent: [u32; 2],
    capture: &NativeClientPixelCapture,
) -> Result<[u32; 4], ExecutableSchemaTransitionFailure> {
    let scale = |value: u32, physical: u32, authored: u32| {
        ((u64::from(value) * u64::from(physical) + u64::from(authored / 2)) / u64::from(authored))
            as u32
    };
    let region = [
        scale(logical[0], capture.width(), logical_extent[0]),
        scale(logical[1], capture.height(), logical_extent[1]),
        scale(logical[0] + logical[2], capture.width(), logical_extent[0]),
        scale(logical[1] + logical[3], capture.height(), logical_extent[1]),
    ];
    (region[0] < region[2]
        && region[1] < region[3]
        && region[2] <= capture.width()
        && region[3] <= capture.height())
    .then_some(region)
    .ok_or(ExecutableSchemaTransitionFailure::CaptureExtentChanged)
}

fn region_bytes(
    capture: &NativeClientPixelCapture,
    region: [u32; 4],
) -> Result<Vec<u8>, ExecutableSchemaTransitionFailure> {
    let row_bytes = capture.width() as usize * 4;
    let left = region[0] as usize * 4;
    let right = region[2] as usize * 4;
    let mut bytes = Vec::with_capacity((right - left) * (region[3] - region[1]) as usize);
    for y in region[1] as usize..region[3] as usize {
        let start = y * row_bytes + left;
        let end = y * row_bytes + right;
        bytes.extend_from_slice(
            capture
                .rgba()
                .get(start..end)
                .ok_or(ExecutableSchemaTransitionFailure::CaptureExtentChanged)?,
        );
    }
    Ok(bytes)
}

impl<Kind> ExecutableSchemaTransitionEvidence<Kind> {
    pub(crate) fn replacement(&self) -> &ExecutableReplacementEvidence<Kind> {
        &self.replacement
    }

    pub(crate) const fn transition(&self) -> PlatformPulseProjectionSchemaTransitionObservation {
        self.transition
    }

    pub(crate) fn query_basis(&self) -> &PlatformPulseQueryProjectionEvidence {
        &self.query_basis
    }

    pub(crate) const fn retained_control_pixel_bytes(&self) -> usize {
        self.retained_control_pixel_bytes
    }

    pub(crate) const fn changed_posture_pixel_bytes(&self) -> usize {
        self.changed_posture_pixel_bytes
    }

    pub(crate) const fn canonical_current_restored(&self) -> bool {
        self.canonical_current_restored
    }
}

impl fmt::Display for ExecutableSchemaTransitionFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Self::StableControlRegionChanged { differing_bytes } = self {
            return write!(
                formatter,
                "schema transition changed {differing_bytes} byte(s) in stable Query-control chrome"
            );
        }
        formatter.write_str(match self {
            Self::MissingTransition => "replacement omitted its schema transition",
            Self::WrongTransitionKind => "schema transition carried the wrong kind",
            Self::WrongFieldProgression => "schema transition carried the wrong field progression",
            Self::PredecessorValueNotPreserved => {
                "schema transition did not preserve the predecessor value"
            }
            Self::QueryValueNotCurrent => "retained Query basis is not SYNCHRONIZED/current",
            Self::QueryOwnerDidNotReachSecondCurrent => {
                "retained Query basis is not owner order five"
            }
            Self::QueryGenerationMissing => "retained Query basis omitted generation identity",
            Self::CaptureExtentChanged => "schema transition changed native capture extent",
            Self::VisualContract => "schema transition visual contract is invalid",
            Self::StableControlRegionChanged { .. } => unreachable!(),
            Self::PostureRegionDidNotChange => {
                "schema transition did not visibly change the native posture region"
            }
            Self::CanonicalCurrentPostureNotRestored => {
                "schema recovery did not restore the canonical current posture pixels"
            }
        })
    }
}
