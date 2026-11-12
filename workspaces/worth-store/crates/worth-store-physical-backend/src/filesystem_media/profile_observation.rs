#[cfg(not(target_os = "macos"))]
use std::num::NonZeroU64;
use std::path::Path;

use sha2::{Digest, Sha256};

#[cfg(not(target_os = "macos"))]
use super::capability_profile::ObservedFilesystemProfile;
use super::FilesystemBackendProfile;
#[cfg(not(target_os = "macos"))]
use super::{CapabilitySupport, FilesystemLocation, MediaCapability};

mod admission_policy;
#[cfg(all(test, target_os = "macos"))]
mod mac_profile_tests;
#[cfg(target_os = "macos")]
mod mounted_filesystem;
mod root_binding;

pub(super) use admission_policy::deny_profile;
pub(super) use root_binding::derive as profile_binding;
pub use root_binding::filesystem_media_build_identity;

pub(super) fn observe_profile(
    root: &super::NamespaceDirectoryHandle,
    boundary: &super::fault_interposition::MediaFaultInterposer,
) -> Result<FilesystemBackendProfile, std::io::ErrorKind> {
    let attempt = boundary.begin(super::MediaOperationRole::ObserveRootProfile, 0);
    if let Some(error) = attempt.fail_before_error() {
        attempt.denied();
        return Err(error.kind());
    }
    match observe_profile_at_handle(root) {
        Ok(profile) => {
            attempt.completed(0);
            Ok(profile)
        }
        Err(kind) => {
            attempt.denied();
            Err(kind)
        }
    }
}

pub(super) fn observe_admission_profile(
    root: &Path,
    boundary: &super::fault_interposition::MediaFaultInterposer,
) -> Result<FilesystemBackendProfile, std::io::ErrorKind> {
    let target = if root.exists() {
        root
    } else {
        root.parent()
            .filter(|parent| parent.exists())
            .unwrap_or(root)
    };
    observe_ambient_profile(target, boundary)
}

fn observe_ambient_profile(
    root: &Path,
    boundary: &super::fault_interposition::MediaFaultInterposer,
) -> Result<FilesystemBackendProfile, std::io::ErrorKind> {
    let attempt = boundary.begin(super::MediaOperationRole::ObserveRootProfile, 0);
    if let Some(error) = attempt.fail_before_error() {
        attempt.denied();
        return Err(error.kind());
    }
    match observe_profile_at_path(root) {
        Ok(profile) => {
            attempt.completed(0);
            Ok(profile)
        }
        Err(kind) => {
            attempt.denied();
            Err(kind)
        }
    }
}

#[cfg(target_os = "macos")]
fn observe_profile_at_path(root: &Path) -> Result<FilesystemBackendProfile, std::io::ErrorKind> {
    let canonical = root.canonicalize().map_err(|error| error.kind())?;
    let directory = cap_std::fs::Dir::open_ambient_dir(&canonical, cap_std::ambient_authority())
        .map_err(|error| error.kind())?;
    mounted_filesystem::observe(&directory.into_std_file())
}

#[cfg(target_os = "macos")]
fn observe_profile_at_handle(
    root: &super::NamespaceDirectoryHandle,
) -> Result<FilesystemBackendProfile, std::io::ErrorKind> {
    let file = root
        .directory()
        .try_clone()
        .map(cap_std::fs::Dir::into_std_file)
        .map_err(|error| error.kind())?;
    mounted_filesystem::observe(&file)
}

#[cfg(not(target_os = "macos"))]
fn observe_profile_at_path(root: &Path) -> Result<FilesystemBackendProfile, std::io::ErrorKind> {
    let canonical = root.canonicalize().map_err(|error| error.kind())?;
    let directory = cap_std::fs::Dir::open_ambient_dir(&canonical, cap_std::ambient_authority())
        .map_err(|error| error.kind())?;
    let file = directory.into_std_file();
    let metadata = file.metadata().map_err(|error| error.kind())?;
    let root_identity = opened_root_identity(&file, &metadata)?;
    let volume_identity = opened_volume_identity(&file, &metadata)?;
    let disks = sysinfo::Disks::new_with_refreshed_list();
    let disk = disks
        .list()
        .iter()
        .filter(|disk| path_is_on_mount(&canonical, disk.mount_point()))
        .max_by_key(|disk| disk.mount_point().components().count())
        .ok_or(std::io::ErrorKind::NotFound)?;
    let filesystem_type = disk.file_system().to_string_lossy().into_owned();
    let allocation_granularity =
        NonZeroU64::new(fs4::allocation_granularity(&canonical).map_err(|error| error.kind())?)
            .ok_or(std::io::ErrorKind::InvalidData)?;
    let support = MediaCapability::ALL.map(|capability| match capability {
        MediaCapability::MemoryMap
        | MediaCapability::DirectIo
        | MediaCapability::SparseAllocation
        | MediaCapability::EagerAllocation => CapabilitySupport::Unsupported,
        _ => CapabilitySupport::Supported,
    });
    let location = if is_remote_path(&canonical, &filesystem_type) {
        FilesystemLocation::Remote
    } else {
        FilesystemLocation::Local
    };
    Ok(FilesystemBackendProfile::from_root_observation(
        ObservedFilesystemProfile {
            support_by_capability: support,
            root_identity,
            volume_identity,
            filesystem_type: filesystem_type.into_boxed_str(),
            allocation_granularity,
            location,
            removable: disk.is_removable(),
            read_only: disk.is_read_only(),
        },
    ))
}

#[cfg(not(target_os = "macos"))]
fn observe_profile_at_handle(
    root: &super::NamespaceDirectoryHandle,
) -> Result<FilesystemBackendProfile, std::io::ErrorKind> {
    let file = root
        .directory()
        .try_clone()
        .map(cap_std::fs::Dir::into_std_file)
        .map_err(|error| error.kind())?;
    let metadata = file.metadata().map_err(|error| error.kind())?;
    let root_identity = opened_root_identity(&file, &metadata)?;
    let volume_identity = opened_volume_identity(&file, &metadata)?;
    let disks = sysinfo::Disks::new_with_refreshed_list();
    let mut matches = disks.list().iter().filter(|disk| {
        mount_volume_identity(disk.mount_point())
            .is_some_and(|candidate| candidate == volume_identity)
    });
    let disk = matches.next().ok_or(std::io::ErrorKind::NotFound)?;
    super::profile_candidate_consistency::require_material_agreement(
        mount_profile_key(disk)?,
        matches
            .map(mount_profile_key)
            .collect::<Result<Vec<_>, _>>()?,
    )?;
    let filesystem_type = disk.file_system().to_string_lossy().into_owned();
    let allocation_granularity = NonZeroU64::new(
        fs4::allocation_granularity(disk.mount_point()).map_err(|error| error.kind())?,
    )
    .ok_or(std::io::ErrorKind::InvalidData)?;
    let support = MediaCapability::ALL.map(|capability| match capability {
        MediaCapability::MemoryMap
        | MediaCapability::DirectIo
        | MediaCapability::SparseAllocation
        | MediaCapability::EagerAllocation => CapabilitySupport::Unsupported,
        _ => CapabilitySupport::Supported,
    });
    let location = if is_remote_path(disk.mount_point(), &filesystem_type) {
        FilesystemLocation::Remote
    } else {
        FilesystemLocation::Local
    };
    Ok(FilesystemBackendProfile::from_root_observation(
        ObservedFilesystemProfile {
            support_by_capability: support,
            root_identity,
            volume_identity,
            filesystem_type: filesystem_type.into_boxed_str(),
            allocation_granularity,
            location,
            removable: disk.is_removable(),
            read_only: disk.is_read_only(),
        },
    ))
}

#[cfg(not(target_os = "macos"))]
fn mount_profile_key(
    disk: &sysinfo::Disk,
) -> Result<(Box<str>, u64, bool, bool), std::io::ErrorKind> {
    let granularity =
        fs4::allocation_granularity(disk.mount_point()).map_err(|error| error.kind())?;
    Ok((
        disk.file_system()
            .to_string_lossy()
            .into_owned()
            .into_boxed_str(),
        granularity,
        disk.is_removable(),
        disk.is_read_only(),
    ))
}

#[cfg(unix)]
fn opened_root_identity(
    _file: &std::fs::File,
    metadata: &std::fs::Metadata,
) -> Result<[u8; 32], std::io::ErrorKind> {
    use std::os::unix::fs::MetadataExt;
    Ok(digest_parts(&[
        &metadata.dev().to_le_bytes(),
        &metadata.ino().to_le_bytes(),
    ]))
}

#[cfg(windows)]
fn opened_root_identity(
    file: &std::fs::File,
    _metadata: &std::fs::Metadata,
) -> Result<[u8; 32], std::io::ErrorKind> {
    let handle = winapi_util::Handle::from_file(file.try_clone().map_err(|error| error.kind())?);
    let info = winapi_util::file::information(&handle).map_err(|error| error.kind())?;
    let volume = info.volume_serial_number();
    Ok(digest_parts(&[
        &volume.to_le_bytes(),
        &info.file_index().to_le_bytes(),
    ]))
}

#[cfg(unix)]
fn opened_volume_identity(
    _file: &std::fs::File,
    metadata: &std::fs::Metadata,
) -> Result<[u8; 32], std::io::ErrorKind> {
    use std::os::unix::fs::MetadataExt;
    Ok(digest_parts(&[&metadata.dev().to_le_bytes()]))
}

#[cfg(windows)]
fn opened_volume_identity(
    file: &std::fs::File,
    _metadata: &std::fs::Metadata,
) -> Result<[u8; 32], std::io::ErrorKind> {
    let handle = winapi_util::Handle::from_file(file.try_clone().map_err(|error| error.kind())?);
    let info = winapi_util::file::information(&handle).map_err(|error| error.kind())?;
    let volume = info.volume_serial_number();
    Ok(digest_parts(&[&volume.to_le_bytes()]))
}

#[cfg(all(unix, not(target_os = "macos")))]
fn mount_volume_identity(path: &Path) -> Option<[u8; 32]> {
    let identity = file_id::get_file_id(path).ok()?;
    let volume = match identity {
        file_id::FileId::Inode { device_id, .. } => device_id,
        file_id::FileId::LowRes {
            volume_serial_number,
            ..
        } => u64::from(volume_serial_number),
        file_id::FileId::HighRes {
            volume_serial_number,
            ..
        } => volume_serial_number,
    };
    Some(digest_parts(&[&volume.to_le_bytes()]))
}

#[cfg(windows)]
fn mount_volume_identity(path: &Path) -> Option<[u8; 32]> {
    let directory = cap_std::fs::Dir::open_ambient_dir(path, cap_std::ambient_authority()).ok()?;
    let handle = winapi_util::Handle::from_file(directory.into_std_file());
    let info = winapi_util::file::information(&handle).ok()?;
    let volume = info.volume_serial_number();
    Some(digest_parts(&[&volume.to_le_bytes()]))
}

#[cfg(not(target_os = "macos"))]
fn path_is_on_mount(path: &Path, mount: &Path) -> bool {
    #[cfg(windows)]
    {
        let normalize_components = |value: &Path| {
            value
                .components()
                .map(|component| {
                    component
                        .as_os_str()
                        .to_string_lossy()
                        .trim_start_matches(r"\\?\")
                        .to_ascii_lowercase()
                })
                .collect::<Vec<_>>()
        };
        let path = normalize_components(path);
        let mount = normalize_components(mount);
        mount.len() <= path.len() && path[..mount.len()] == mount
    }
    #[cfg(not(windows))]
    path.starts_with(mount)
}

fn digest_parts(parts: &[&[u8]]) -> [u8; 32] {
    let mut digest = Sha256::new();
    for part in parts {
        digest.update((part.len() as u64).to_le_bytes());
        digest.update(part);
    }
    digest.finalize().into()
}

fn is_remote_path(path: &Path, filesystem_type: &str) -> bool {
    let fs = filesystem_type.to_ascii_lowercase();
    if matches!(fs.as_str(), "nfs" | "nfs4" | "cifs" | "smbfs") {
        return true;
    }
    #[cfg(windows)]
    {
        use std::path::{Component, Prefix};
        matches!(
            path.components().next(),
            Some(Component::Prefix(prefix))
                if matches!(prefix.kind(), Prefix::UNC(_, _) | Prefix::VerbatimUNC(_, _))
        )
    }
    #[cfg(not(windows))]
    {
        let _ = path;
        false
    }
}
