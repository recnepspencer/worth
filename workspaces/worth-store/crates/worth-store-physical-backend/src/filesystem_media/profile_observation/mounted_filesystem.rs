//! Ephemeral macOS mount selection from opened descriptors, never path prefixes.
use std::{fs::File, io::ErrorKind, num::NonZeroU64, os::unix::fs::MetadataExt};

use super::super::{
    capability_profile::ObservedFilesystemProfile, profile_candidate_consistency,
    CapabilitySupport, FilesystemBackendProfile, FilesystemLocation, MediaCapability,
};

#[cfg(test)]
mod tests;

pub(super) fn observe(file: &File) -> Result<FilesystemBackendProfile, ErrorKind> {
    let metadata = file.metadata().map_err(|error| error.kind())?;
    let target = FilesystemObservation::read(file, metadata.dev())?;
    let disks = sysinfo::Disks::new_with_refreshed_list();
    let material = select_material(
        disks
            .list()
            .iter()
            .map(|disk| observe_candidate(disk, target)),
    )?;
    Ok(profile(
        super::opened_root_identity(file, &metadata)?,
        super::opened_volume_identity(file, &metadata)?,
        material,
    ))
}

/// This selector exists only during this observation. It never enters a basis.
#[derive(Clone, Copy)]
struct FilesystemObservation {
    device: u64,
    fsid: u64,
    granularity: NonZeroU64,
    read_only: bool,
}

impl FilesystemObservation {
    fn read(file: &File, device: u64) -> Result<Self, ErrorKind> {
        let observed =
            rustix::fs::fstatvfs(file).map_err(|error| std::io::Error::from(error).kind())?;
        Ok(Self {
            device,
            fsid: observed.f_fsid,
            granularity: NonZeroU64::new(observed.f_frsize).ok_or(ErrorKind::InvalidData)?,
            read_only: observed
                .f_flag
                .contains(rustix::fs::StatVfsMountFlags::RDONLY),
        })
    }

    fn same_mount(self, candidate: Self) -> bool {
        self.device == candidate.device && self.fsid == candidate.fsid
    }
}

#[derive(Debug, PartialEq, Eq)]
struct MountMaterial {
    filesystem_type: Box<str>,
    granularity: NonZeroU64,
    location: FilesystemLocation,
    removable: bool,
    read_only: bool,
}

fn observe_candidate(
    disk: &sysinfo::Disk,
    target: FilesystemObservation,
) -> Result<Option<MountMaterial>, ErrorKind> {
    let directory =
        cap_std::fs::Dir::open_ambient_dir(disk.mount_point(), cap_std::ambient_authority())
            .map_err(|error| error.kind())?;
    let file = directory.into_std_file();
    let metadata = file.metadata().map_err(|error| error.kind())?;
    // A reliably different device needs no filesystem observation. All other
    // observation failures remain errors, including same-device aliases.
    if metadata.dev() != target.device {
        return Ok(None);
    }
    let candidate = FilesystemObservation::read(&file, metadata.dev())?;
    let filesystem_type = disk.file_system().to_string_lossy().into_owned();
    let location = if super::is_remote_path(disk.mount_point(), &filesystem_type) {
        FilesystemLocation::Remote
    } else {
        FilesystemLocation::Local
    };
    matching_material(
        target,
        candidate,
        MountMaterial {
            filesystem_type: filesystem_type.into_boxed_str(),
            granularity: candidate.granularity,
            location,
            removable: disk.is_removable(),
            read_only: disk.is_read_only(),
        },
    )
}

fn matching_material(
    target: FilesystemObservation,
    candidate: FilesystemObservation,
    material: MountMaterial,
) -> Result<Option<MountMaterial>, ErrorKind> {
    if !target.same_mount(candidate) {
        return Ok(None);
    }
    if target.granularity != candidate.granularity
        || target.read_only != candidate.read_only
        || candidate.read_only != material.read_only
    {
        return Err(ErrorKind::InvalidData);
    }
    Ok(Some(material))
}

fn select_material(
    candidates: impl IntoIterator<Item = Result<Option<MountMaterial>, ErrorKind>>,
) -> Result<MountMaterial, ErrorKind> {
    let mut selected = None;
    for candidate in candidates {
        let Some(candidate) = candidate? else {
            continue;
        };
        selected = Some(match selected {
            Some(first) => {
                profile_candidate_consistency::require_material_agreement(first, [candidate])?
            }
            None => candidate,
        });
    }
    selected.ok_or(ErrorKind::NotFound)
}

fn profile(
    root_identity: [u8; 32],
    volume_identity: [u8; 32],
    material: MountMaterial,
) -> FilesystemBackendProfile {
    let support = MediaCapability::ALL.map(|capability| match capability {
        MediaCapability::MemoryMap
        | MediaCapability::DirectIo
        | MediaCapability::SparseAllocation
        | MediaCapability::EagerAllocation => CapabilitySupport::Unsupported,
        _ => CapabilitySupport::Supported,
    });
    FilesystemBackendProfile::from_root_observation(ObservedFilesystemProfile {
        support_by_capability: support,
        root_identity,
        volume_identity,
        filesystem_type: material.filesystem_type,
        allocation_granularity: material.granularity,
        location: material.location,
        removable: material.removable,
        read_only: material.read_only,
    })
}
