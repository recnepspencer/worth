use std::fmt;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::Serialize;

use crate::external_observation::LifecycleFailureSnapshot;
use crate::native_platform::{current_platform_posture, NativePlatformPosture};

use super::report::{ExecutableWorldFailureTeardown, PulseExecutableWorldFailure};

const MAXIMUM_ARTIFACT_BYTES: usize = 64 * 1_024 * 1_024;
const MAXIMUM_ROOT_RESERVATION_ATTEMPTS: usize = 4_096;
static NEXT_ARTIFACT: AtomicU64 = AtomicU64::new(1);

#[derive(Debug)]
pub(crate) struct RetainedFailureArtifact {
    root: PathBuf,
    retained_bytes: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FailureArtifactDiscardEvidence {
    removed_owned_root: bool,
}

#[derive(Debug)]
pub(crate) enum FailureArtifactFailure {
    Encode(serde_json::Error),
    EncodeNativeCapture(xcap::image::ImageError),
    InvalidNativeCapture,
    BudgetExceeded(usize),
    CreateRoot(std::io::Error),
    WriteFile {
        path: PathBuf,
        error: std::io::Error,
    },
    SyncFile {
        path: PathBuf,
        error: std::io::Error,
    },
    Cleanup(std::io::Error),
    Residue(PathBuf),
}

pub(super) struct FailureArtifactInputs {
    pub(super) source_snapshot: Option<Box<[u8]>>,
    pub(super) lifecycle: Option<LifecycleFailureSnapshot>,
    pub(super) native_capture: Option<FailureNativeCaptureInput>,
}

pub(super) enum FailureNativeCaptureInput {
    Captured(crate::external_observation::NativeClientPixelCapture),
    Failed(String),
}

#[derive(Serialize)]
struct FailureManifest<'a> {
    schema: &'static str,
    primary: String,
    teardown: String,
    environment: FailureEnvironment,
    lifecycle: Option<FailureLifecycle<'a>>,
    source_snapshot: Option<&'static str>,
    native_capture: Option<FailureNativeCapture<'a>>,
    retained_by_default: bool,
    maximum_artifact_bytes: usize,
}

#[derive(Serialize)]
struct FailureEnvironment {
    operating_system: &'static str,
    architecture: &'static str,
    native_posture: &'static str,
}

#[derive(Serialize)]
struct FailureLifecycle<'a> {
    accepted_events: usize,
    accepted_bytes: usize,
    trace: &'a [crate::external_observation::LifecycleTraceEntry],
}

#[derive(Serialize)]
struct FailureNativeCapture<'a> {
    file: Option<&'static str>,
    failure: Option<&'a str>,
    width: Option<u32>,
    height: Option<u32>,
    capture_count: Option<u32>,
}

impl FailureArtifactInputs {
    pub(super) fn none() -> Self {
        Self {
            source_snapshot: None,
            lifecycle: None,
            native_capture: None,
        }
    }
}

impl RetainedFailureArtifact {
    pub(super) fn create(
        primary: &PulseExecutableWorldFailure,
        teardown: &ExecutableWorldFailureTeardown,
        inputs: FailureArtifactInputs,
    ) -> Result<Self, FailureArtifactFailure> {
        let native_capture_bytes = encode_native_capture(inputs.native_capture.as_ref())?;
        let manifest = manifest(primary, teardown, &inputs);
        let manifest_bytes =
            serde_json::to_vec_pretty(&manifest).map_err(FailureArtifactFailure::Encode)?;
        let source_bytes = inputs.source_snapshot.as_deref().unwrap_or_default();
        let retained_bytes = manifest_bytes
            .len()
            .saturating_add(source_bytes.len())
            .saturating_add(native_capture_bytes.as_ref().map_or(0, std::vec::Vec::len));
        if retained_bytes > MAXIMUM_ARTIFACT_BYTES {
            return Err(FailureArtifactFailure::BudgetExceeded(retained_bytes));
        }
        let root = reserve_artifact_root().map_err(FailureArtifactFailure::CreateRoot)?;
        if let Err(failure) = write_bundle(
            &root,
            &manifest_bytes,
            source_bytes,
            native_capture_bytes.as_deref(),
        ) {
            let cleanup = fs::remove_dir_all(&root);
            return match cleanup {
                Ok(()) => Err(failure),
                Err(error) => Err(FailureArtifactFailure::Cleanup(error)),
            };
        }
        Ok(Self {
            root,
            retained_bytes,
        })
    }

    pub(crate) fn path(&self) -> &Path {
        &self.root
    }

    pub(crate) fn retained_bytes(&self) -> usize {
        self.retained_bytes
    }

    pub(crate) fn maximum_bytes(&self) -> usize {
        MAXIMUM_ARTIFACT_BYTES
    }

    pub(crate) fn discard(self) -> Result<FailureArtifactDiscardEvidence, FailureArtifactFailure> {
        fs::remove_dir_all(&self.root).map_err(FailureArtifactFailure::Cleanup)?;
        if self.root.exists() {
            return Err(FailureArtifactFailure::Residue(self.root));
        }
        Ok(FailureArtifactDiscardEvidence {
            removed_owned_root: true,
        })
    }
}

impl FailureArtifactDiscardEvidence {
    pub(crate) fn removed_owned_root(self) -> bool {
        self.removed_owned_root
    }
}

fn manifest<'a>(
    primary: &PulseExecutableWorldFailure,
    teardown: &ExecutableWorldFailureTeardown,
    inputs: &'a FailureArtifactInputs,
) -> FailureManifest<'a> {
    let lifecycle = inputs.lifecycle.as_ref().map(|snapshot| {
        let measurement = snapshot.measurement();
        FailureLifecycle {
            accepted_events: measurement.accepted_events(),
            accepted_bytes: measurement.accepted_bytes(),
            trace: snapshot.trace(),
        }
    });
    let native_capture = inputs.native_capture.as_ref().map(|capture| match capture {
        FailureNativeCaptureInput::Captured(capture) => FailureNativeCapture {
            file: Some("native-client.png"),
            failure: None,
            width: Some(capture.width()),
            height: Some(capture.height()),
            capture_count: Some(capture.capture_count()),
        },
        FailureNativeCaptureInput::Failed(failure) => FailureNativeCapture {
            file: None,
            failure: Some(failure),
            width: None,
            height: None,
            capture_count: None,
        },
    });
    FailureManifest {
        schema: "worth-ui.platform-pulse.failure-artifact.v1",
        primary: primary.to_string(),
        teardown: teardown.to_string(),
        environment: FailureEnvironment {
            operating_system: std::env::consts::OS,
            architecture: std::env::consts::ARCH,
            native_posture: posture_name(current_platform_posture()),
        },
        lifecycle,
        source_snapshot: inputs.source_snapshot.as_ref().map(|_| "source.wui"),
        native_capture,
        retained_by_default: true,
        maximum_artifact_bytes: MAXIMUM_ARTIFACT_BYTES,
    }
}

fn posture_name(posture: NativePlatformPosture) -> &'static str {
    match posture {
        NativePlatformPosture::CertifiedExecutable => "certified_executable",
    }
}

fn reserve_artifact_root() -> Result<PathBuf, std::io::Error> {
    reserve_artifact_root_with(artifact_root_candidate)
}

fn reserve_artifact_root_with(
    mut next_candidate: impl FnMut() -> PathBuf,
) -> Result<PathBuf, std::io::Error> {
    for _ in 0..MAXIMUM_ROOT_RESERVATION_ATTEMPTS {
        let candidate = next_candidate();
        match fs::create_dir(&candidate) {
            Ok(()) => return Ok(candidate),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::AlreadyExists,
        format!(
            "all {MAXIMUM_ROOT_RESERVATION_ATTEMPTS} bounded failure-artifact root reservations already exist"
        ),
    ))
}

fn artifact_root_candidate() -> PathBuf {
    let ordinal = NEXT_ARTIFACT.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "worth-ui-platform-pulse-failure-{}-{ordinal}",
        std::process::id()
    ))
}

fn write_bundle(
    root: &Path,
    manifest_bytes: &[u8],
    source_bytes: &[u8],
    native_capture_bytes: Option<&[u8]>,
) -> Result<(), FailureArtifactFailure> {
    write_synced(&root.join("manifest.json"), manifest_bytes)?;
    if !source_bytes.is_empty() {
        write_synced(&root.join("source.wui"), source_bytes)?;
    }
    if let Some(bytes) = native_capture_bytes {
        write_synced(&root.join("native-client.png"), bytes)?;
    }
    Ok(())
}

fn encode_native_capture(
    input: Option<&FailureNativeCaptureInput>,
) -> Result<Option<Vec<u8>>, FailureArtifactFailure> {
    let Some(FailureNativeCaptureInput::Captured(capture)) = input else {
        return Ok(None);
    };
    encode_native_capture_png(capture).map(Some)
}

pub(crate) fn encode_native_capture_png(
    capture: &crate::external_observation::NativeClientPixelCapture,
) -> Result<Vec<u8>, FailureArtifactFailure> {
    let image = xcap::image::RgbaImage::from_raw(
        capture.width(),
        capture.height(),
        capture.rgba().to_vec(),
    )
    .ok_or(FailureArtifactFailure::InvalidNativeCapture)?;
    let mut bytes = std::io::Cursor::new(Vec::new());
    xcap::image::DynamicImage::ImageRgba8(image)
        .write_to(&mut bytes, xcap::image::ImageFormat::Png)
        .map_err(FailureArtifactFailure::EncodeNativeCapture)?;
    Ok(bytes.into_inner())
}

fn write_synced(path: &Path, bytes: &[u8]) -> Result<(), FailureArtifactFailure> {
    let mut file = File::create(path).map_err(|error| FailureArtifactFailure::WriteFile {
        path: path.to_owned(),
        error,
    })?;
    file.write_all(bytes)
        .map_err(|error| FailureArtifactFailure::WriteFile {
            path: path.to_owned(),
            error,
        })?;
    file.sync_all()
        .map_err(|error| FailureArtifactFailure::SyncFile {
            path: path.to_owned(),
            error,
        })
}

impl fmt::Display for FailureArtifactFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Encode(error) => write!(formatter, "encode failure manifest: {error}"),
            Self::EncodeNativeCapture(error) => {
                write!(formatter, "encode native failure capture: {error}")
            }
            Self::InvalidNativeCapture => {
                formatter.write_str("native failure capture dimensions do not match its bytes")
            }
            Self::BudgetExceeded(bytes) => {
                write!(
                    formatter,
                    "failure artifact is {bytes} bytes; limit is {MAXIMUM_ARTIFACT_BYTES}"
                )
            }
            Self::CreateRoot(error) => write!(formatter, "create failure artifact root: {error}"),
            Self::WriteFile { path, error } => {
                write!(formatter, "write {}: {error}", path.display())
            }
            Self::SyncFile { path, error } => {
                write!(formatter, "sync {}: {error}", path.display())
            }
            Self::Cleanup(error) => write!(formatter, "remove failure artifact: {error}"),
            Self::Residue(path) => {
                write!(formatter, "failure artifact remained: {}", path.display())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeSet, VecDeque};
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::{reserve_artifact_root, reserve_artifact_root_with};

    static NEXT_RESERVATION_WORLD: AtomicU64 = AtomicU64::new(1);

    #[test]
    fn stale_retained_root_cannot_block_a_fresh_bounded_reservation() {
        let ordinal = NEXT_RESERVATION_WORLD.fetch_add(1, Ordering::Relaxed);
        let base = std::env::temp_dir().join(format!(
            "worth-ui-platform-pulse-artifact-reservation-test-{}-{ordinal}",
            std::process::id()
        ));
        let stale = base.with_extension("stale");
        let fresh = base.with_extension("fresh");
        std::fs::create_dir(&stale).expect("create stale collision root");
        let mut candidates = VecDeque::from([stale.clone(), fresh.clone()]);
        let reserved = reserve_artifact_root_with(|| {
            candidates
                .pop_front()
                .expect("bounded test supplies a fresh successor")
        })
        .expect("skip stale root");
        assert_eq!(reserved, fresh);
        std::fs::remove_dir(&stale).expect("remove stale test root");
        std::fs::remove_dir(&reserved).expect("remove reserved test root");
    }

    #[test]
    fn concurrent_reservations_are_unique_and_independently_owned() {
        let joins = (0..16)
            .map(|_| std::thread::spawn(reserve_artifact_root))
            .collect::<Vec<_>>();
        let roots = joins
            .into_iter()
            .map(|join| {
                join.join()
                    .expect("reservation thread joins")
                    .expect("reservation succeeds")
            })
            .collect::<Vec<_>>();
        assert_eq!(roots.iter().collect::<BTreeSet<_>>().len(), roots.len());
        for root in roots {
            std::fs::remove_dir(root).expect("remove independently owned reservation");
        }
    }
}
