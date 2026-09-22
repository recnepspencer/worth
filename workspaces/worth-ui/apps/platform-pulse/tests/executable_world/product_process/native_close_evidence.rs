use std::fmt;
use std::fs;
use std::path::Path;

use serde::Deserialize;

pub(crate) const NATIVE_CLOSE_EVIDENCE_FILE_NAME: &str = ".worth-ui-native-close-evidence.json";
pub(crate) const NATIVE_CLOSE_EVIDENCE_PATH_ENVIRONMENT: &str =
    "WORTH_UI_EXECUTABLE_WORLD_NATIVE_CLOSE_EVIDENCE";

#[derive(Debug, Deserialize)]
pub(crate) struct PlatformPulseNativeCloseEvidence {
    schema_version: u8,
    sample_frames: Vec<PlatformPulseNativeSampleFrameEvidence>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(crate) struct PlatformPulseNativeSampleFrameEvidence {
    sampled_chrome: Vec<NativeChromeSampleEvidence>,
    frame: u64,
    presentation_epoch: Option<u64>,
    presentation_attempt: Option<u64>,
    retained_baseline_rgba8: [u8; 4],
    retained_center_rgba8: [u8; 4],
    logical_damage_regions: u64,
    rendered_pixels: u64,
    queue_submissions: u64,
    presents: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(crate) struct NativeChromeSampleEvidence {
    owner: u64,
    inline: bool,
    thumb: bool,
    bounds_milli: [i64; 4],
}

impl NativeChromeSampleEvidence {
    pub(crate) const fn inline(&self) -> bool {
        self.inline
    }
    pub(crate) const fn thumb(&self) -> bool {
        self.thumb
    }
    pub(crate) const fn bounds_milli(&self) -> [i64; 4] {
        self.bounds_milli
    }
}

#[derive(Debug)]
pub(crate) enum PlatformPulseNativeCloseEvidenceFailure {
    Read(std::io::Error),
    Decode(serde_json::Error),
    Schema(u8),
}

impl PlatformPulseNativeCloseEvidence {
    pub(crate) fn read(
        source_root: &Path,
    ) -> Result<Self, PlatformPulseNativeCloseEvidenceFailure> {
        let bytes = fs::read(source_root.join(NATIVE_CLOSE_EVIDENCE_FILE_NAME))
            .map_err(PlatformPulseNativeCloseEvidenceFailure::Read)?;
        let evidence: Self = serde_json::from_slice(&bytes)
            .map_err(PlatformPulseNativeCloseEvidenceFailure::Decode)?;
        if evidence.schema_version != 1 {
            return Err(PlatformPulseNativeCloseEvidenceFailure::Schema(
                evidence.schema_version,
            ));
        }
        Ok(evidence)
    }

    pub(crate) fn sample_frames(&self) -> &[PlatformPulseNativeSampleFrameEvidence] {
        &self.sample_frames
    }
}

impl PlatformPulseNativeSampleFrameEvidence {
    pub(crate) fn sampled_chrome(&self) -> &[NativeChromeSampleEvidence] {
        &self.sampled_chrome
    }
    pub(crate) const fn frame(&self) -> u64 {
        self.frame
    }

    pub(crate) const fn presentation_epoch(&self) -> Option<u64> {
        self.presentation_epoch
    }

    pub(crate) const fn presentation_attempt(&self) -> Option<u64> {
        self.presentation_attempt
    }

    pub(crate) const fn logical_damage_regions(&self) -> u64 {
        self.logical_damage_regions
    }

    pub(crate) const fn rendered_pixels(&self) -> u64 {
        self.rendered_pixels
    }

    pub(crate) const fn queue_submissions(&self) -> u64 {
        self.queue_submissions
    }

    pub(crate) const fn presents(&self) -> u64 {
        self.presents
    }
}

impl fmt::Display for PlatformPulseNativeCloseEvidenceFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read(error) => write!(formatter, "read native close evidence: {error}"),
            Self::Decode(error) => write!(formatter, "decode native close evidence: {error}"),
            Self::Schema(version) => {
                write!(
                    formatter,
                    "unsupported native close evidence schema {version}"
                )
            }
        }
    }
}
