use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use super::{
    LoweredNonCurrentStagingPlan, NonCurrentStagingArtifact, PhysicalRecoveryStagingOwner,
};

#[derive(Debug)]
pub enum NonCurrentStagingExecutionDenial {
    Io(std::io::Error),
    StagingIdentityConflict,
    SourceArtifactMismatch { output_name: String },
    ClosedMediaConflict,
    AllocationFailed,
    CounterOverflow,
    ContinuationDenied { boundary: NonCurrentStagingBoundary },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NonCurrentStagingBoundary {
    Allocation,
    Artifact { index: u64 },
    OwnerEffect,
    OwnerEffectApplied,
    Finalization,
}

impl From<std::io::Error> for NonCurrentStagingExecutionDenial {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClosedNonCurrentStagingMedia {
    root: PathBuf,
    plan_fingerprint: [u8; 32],
    content_fingerprint: [u8; 32],
}

impl ClosedNonCurrentStagingMedia {
    pub fn root(&self) -> &Path {
        &self.root
    }
    pub const fn plan_fingerprint(&self) -> [u8; 32] {
        self.plan_fingerprint
    }
    pub const fn content_fingerprint(&self) -> [u8; 32] {
        self.content_fingerprint
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NonCurrentStagingExecutionReceipt {
    plan_fingerprint: [u8; 32],
    bytes_copied: u64,
    artifacts_materialized: u64,
    maximum_resident_buffer_bytes: u64,
    media: ClosedNonCurrentStagingMedia,
}

impl NonCurrentStagingExecutionReceipt {
    pub const fn plan_fingerprint(&self) -> [u8; 32] {
        self.plan_fingerprint
    }
    pub const fn bytes_copied(&self) -> u64 {
        self.bytes_copied
    }
    pub const fn artifacts_materialized(&self) -> u64 {
        self.artifacts_materialized
    }
    pub const fn maximum_resident_buffer_bytes(&self) -> u64 {
        self.maximum_resident_buffer_bytes
    }
    pub const fn media(&self) -> &ClosedNonCurrentStagingMedia {
        &self.media
    }
}

impl PhysicalRecoveryStagingOwner {
    pub fn execute(
        request: super::NonCurrentStagingPlanRequest,
    ) -> Result<NonCurrentStagingExecutionReceipt, NonCurrentStagingExecutionDenial> {
        let plan = Self::lower_for_execution(request)
            .map_err(|_| NonCurrentStagingExecutionDenial::StagingIdentityConflict)?;
        Self::execute_lowered(plan)
    }

    pub fn execute_lowered(
        plan: LoweredNonCurrentStagingPlan,
    ) -> Result<NonCurrentStagingExecutionReceipt, NonCurrentStagingExecutionDenial> {
        execute_lowered_plan(plan, &mut |_| true)
    }

    pub fn execute_lowered_guarded(
        plan: LoweredNonCurrentStagingPlan,
        mut continuation: impl FnMut(NonCurrentStagingBoundary) -> bool,
    ) -> Result<NonCurrentStagingExecutionReceipt, NonCurrentStagingExecutionDenial> {
        execute_lowered_plan(plan, &mut continuation)
    }
}

/// One compiled lowered-plan copy and finalization for every continuation.
#[inline(never)]
fn execute_lowered_plan(
    plan: LoweredNonCurrentStagingPlan,
    continuation: &mut dyn FnMut(NonCurrentStagingBoundary) -> bool,
) -> Result<NonCurrentStagingExecutionReceipt, NonCurrentStagingExecutionDenial> {
    let copied = copy_lowered(&plan, continuation)?;
    finalize_lowered(plan, copied, [0; 32], continuation)
}

#[inline(never)]
pub(super) fn copy_lowered(
    plan: &LoweredNonCurrentStagingPlan,
    continuation: &mut dyn FnMut(NonCurrentStagingBoundary) -> bool,
) -> Result<(u64, Sha256), NonCurrentStagingExecutionDenial> {
    admit_continuation(continuation, NonCurrentStagingBoundary::Allocation)?;
    std::fs::create_dir_all(plan.binding().staging_root())?;
    admit_staging_identity(plan.binding().staging_root(), plan.binding().fingerprint())?;
    let mut buffer = Vec::new();
    buffer
        .try_reserve_exact(plan.binding().copy_buffer_bytes())
        .map_err(|_| NonCurrentStagingExecutionDenial::AllocationFailed)?;
    buffer.resize(plan.binding().copy_buffer_bytes(), 0);
    let mut bytes_copied = 0_u64;
    let mut content = Sha256::new();
    for (index, (artifact, expected_name)) in plan
        .artifacts()
        .iter()
        .zip(plan.artifact_names())
        .enumerate()
    {
        admit_continuation(
            continuation,
            NonCurrentStagingBoundary::Artifact {
                index: u64::try_from(index)
                    .map_err(|_| NonCurrentStagingExecutionDenial::CounterOverflow)?,
            },
        )?;
        debug_assert_eq!(artifact.output_name(), expected_name);
        copy_or_verify_artifact(
            plan.binding().source_root(),
            plan.binding().staging_root(),
            artifact,
            &mut buffer,
        )?;
        bytes_copied = bytes_copied
            .checked_add(artifact.output_bytes())
            .ok_or(NonCurrentStagingExecutionDenial::CounterOverflow)?;
        content.update(artifact.output_digest());
    }
    Ok((bytes_copied, content))
}

#[inline(never)]
pub(super) fn finalize_lowered(
    plan: LoweredNonCurrentStagingPlan,
    copied: (u64, Sha256),
    effect_fingerprint: [u8; 32],
    continuation: &mut dyn FnMut(NonCurrentStagingBoundary) -> bool,
) -> Result<NonCurrentStagingExecutionReceipt, NonCurrentStagingExecutionDenial> {
    admit_continuation(continuation, NonCurrentStagingBoundary::Finalization)?;
    close_staging_media(
        plan.binding().staging_root(),
        plan.binding().fingerprint(),
        effect_fingerprint,
    )?;
    crate::directory_durability::sync_directory(plan.binding().staging_root())?;
    let media = ClosedNonCurrentStagingMedia {
        root: plan.binding().staging_root().to_path_buf(),
        plan_fingerprint: plan.binding().fingerprint(),
        content_fingerprint: staged_content_fingerprint(copied.1, effect_fingerprint),
    };
    Ok(NonCurrentStagingExecutionReceipt {
        plan_fingerprint: plan.binding().fingerprint(),
        bytes_copied: copied.0,
        artifacts_materialized: plan.binding().artifact_count(),
        maximum_resident_buffer_bytes: plan.binding().copy_buffer_bytes() as u64,
        media,
    })
}

fn staged_content_fingerprint(mut copied: Sha256, effect_fingerprint: [u8; 32]) -> [u8; 32] {
    copied.update(b"worth-store-staged-owner-effect-v1");
    copied.update(effect_fingerprint);
    copied.finalize().into()
}

pub(super) fn admit_continuation(
    continuation: &mut dyn FnMut(NonCurrentStagingBoundary) -> bool,
    boundary: NonCurrentStagingBoundary,
) -> Result<(), NonCurrentStagingExecutionDenial> {
    if continuation(boundary) {
        Ok(())
    } else {
        Err(NonCurrentStagingExecutionDenial::ContinuationDenied { boundary })
    }
}

fn admit_staging_identity(
    root: &Path,
    fingerprint: [u8; 32],
) -> Result<(), NonCurrentStagingExecutionDenial> {
    let marker = root.join(".staging-identity");
    if marker.exists() {
        return if std::fs::read(marker)? == fingerprint {
            Ok(())
        } else {
            Err(NonCurrentStagingExecutionDenial::StagingIdentityConflict)
        };
    }
    write_new_synced_file(&marker, &fingerprint)?;
    crate::directory_durability::sync_directory(root)?;
    Ok(())
}

fn copy_or_verify_artifact(
    source_root: &Path,
    staging_root: &Path,
    artifact: &NonCurrentStagingArtifact,
    buffer: &mut [u8],
) -> Result<(), NonCurrentStagingExecutionDenial> {
    let target = staging_root.join(artifact.output_name());
    if target.exists() {
        verify_file(&target, artifact)?;
        return verify_source_artifact(source_root, artifact, buffer);
    }
    let pending = staging_root.join(format!("{}.pending", artifact.output_name()));
    if let Some(bytes) = artifact.inline_bytes() {
        write_synced_file(&pending, bytes)?;
        publish_pending(&pending, &target, artifact)?;
        crate::directory_durability::sync_directory(staging_root)?;
        return Ok(());
    }
    let source = source_root.join(artifact.output_name());
    let mut input = File::open(&source)?;
    let mut output = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&pending)?;
    let mut source_digest = Sha256::new();
    let mut output_digest = Sha256::new();
    let mut source_bytes = 0_u64;
    let mut output_bytes = 0_u64;
    loop {
        let read = input.read(buffer)?;
        if read == 0 {
            break;
        }
        source_digest.update(&buffer[..read]);
        source_bytes = source_bytes
            .checked_add(read as u64)
            .ok_or(NonCurrentStagingExecutionDenial::CounterOverflow)?;
        let remaining = artifact.output_bytes().saturating_sub(output_bytes);
        let write = read.min(usize::try_from(remaining).unwrap_or(usize::MAX));
        if write > 0 {
            output.write_all(&buffer[..write])?;
            output_digest.update(&buffer[..write]);
            output_bytes = output_bytes
                .checked_add(write as u64)
                .ok_or(NonCurrentStagingExecutionDenial::CounterOverflow)?;
        }
    }
    output.sync_all()?;
    if source_bytes != artifact.source_bytes()
        || source_digest.finalize().as_slice() != artifact.source_digest()
        || output_bytes != artifact.output_bytes()
        || output_digest.finalize().as_slice() != artifact.output_digest()
    {
        return Err(NonCurrentStagingExecutionDenial::SourceArtifactMismatch {
            output_name: artifact.output_name().to_owned(),
        });
    }
    publish_pending(&pending, &target, artifact)?;
    crate::directory_durability::sync_directory(staging_root)?;
    Ok(())
}

fn verify_source_artifact(
    source_root: &Path,
    artifact: &NonCurrentStagingArtifact,
    buffer: &mut [u8],
) -> Result<(), NonCurrentStagingExecutionDenial> {
    if artifact.inline_bytes().is_some() {
        return Ok(());
    }
    let mut source = File::open(source_root.join(artifact.output_name()))?;
    let mut digest = Sha256::new();
    let mut bytes = 0_u64;
    loop {
        let read = source.read(buffer)?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
        bytes = bytes
            .checked_add(read as u64)
            .ok_or(NonCurrentStagingExecutionDenial::CounterOverflow)?;
    }
    if bytes == artifact.source_bytes() && digest.finalize().as_slice() == artifact.source_digest()
    {
        Ok(())
    } else {
        Err(NonCurrentStagingExecutionDenial::SourceArtifactMismatch {
            output_name: artifact.output_name().to_owned(),
        })
    }
}

fn publish_pending(
    pending: &Path,
    target: &Path,
    artifact: &NonCurrentStagingArtifact,
) -> Result<(), NonCurrentStagingExecutionDenial> {
    match std::fs::rename(pending, target) {
        Ok(()) => Ok(()),
        Err(_error) if target.exists() => {
            verify_file(target, artifact)?;
            std::fs::remove_file(pending)?;
            Ok(())
        }
        Err(error) => Err(error.into()),
    }
}

fn verify_file(
    path: &Path,
    artifact: &NonCurrentStagingArtifact,
) -> Result<(), NonCurrentStagingExecutionDenial> {
    let bytes = std::fs::read(path)?;
    if bytes.len() as u64 != artifact.output_bytes()
        || Sha256::digest(&bytes).as_slice() != artifact.output_digest()
    {
        Err(NonCurrentStagingExecutionDenial::SourceArtifactMismatch {
            output_name: artifact.output_name().to_owned(),
        })
    } else {
        Ok(())
    }
}

fn close_staging_media(
    root: &Path,
    fingerprint: [u8; 32],
    effect_fingerprint: [u8; 32],
) -> Result<(), NonCurrentStagingExecutionDenial> {
    let close = root.join(".closed-staging");
    let mut binding = [0_u8; 64];
    binding[..32].copy_from_slice(&fingerprint);
    binding[32..].copy_from_slice(&effect_fingerprint);
    if close.exists() {
        return if std::fs::read(close)? == binding {
            Ok(())
        } else {
            Err(NonCurrentStagingExecutionDenial::ClosedMediaConflict)
        };
    }
    write_new_synced_file(&close, &binding)?;
    Ok(())
}

fn write_new_synced_file(path: &Path, bytes: &[u8]) -> Result<(), std::io::Error> {
    let mut file = OpenOptions::new().create_new(true).write(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}

fn write_synced_file(path: &Path, bytes: &[u8]) -> Result<(), std::io::Error> {
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}
