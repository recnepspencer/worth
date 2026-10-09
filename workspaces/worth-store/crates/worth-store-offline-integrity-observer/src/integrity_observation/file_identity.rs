use std::fs::{File, Metadata, OpenOptions};
use std::io;
use std::path::Path;
use std::time::Duration;

#[cfg(windows)]
pub(crate) const WINDOWS_HIGH_RES_IDENTITY_BYTES: u64 = 24;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum PhysicalFileIdentity {
    #[cfg(unix)]
    Unix { device: u64, inode: u64 },
    #[cfg(windows)]
    Windows {
        volume_serial_number: u64,
        file_identifier: [u8; 16],
    },
    #[cfg(not(any(unix, windows)))]
    Portable { bytes: u64, modified_nanos: u128 },
}

pub(crate) fn open_observed_file(path: &Path) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(unix_no_follow_flag());
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.share_mode(0x0000_0001).custom_flags(0x0020_0000);
    }
    options.open(path)
}

pub(crate) fn open_directory_guard(path: &Path) -> io::Result<File> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(unix_no_follow_flag());
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options
            .share_mode(0x0000_0001 | 0x0000_0002)
            .custom_flags(0x0200_0000 | 0x0020_0000);
    }
    options.open(path)
}

#[cfg(any(
    target_os = "macos",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "dragonfly"
))]
const fn unix_no_follow_flag() -> i32 {
    0x0000_0100
}

#[cfg(all(
    unix,
    not(any(
        target_os = "macos",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly"
    ))
))]
const fn unix_no_follow_flag() -> i32 {
    0x0002_0000
}

pub(crate) fn open_directory_identity(
    path: &Path,
    maximum_output_bytes: u64,
    maximum_elapsed: Duration,
) -> io::Result<(File, PhysicalFileIdentity)> {
    let directory = open_directory_guard(path)?;
    let identity = identity_from_file(&directory, path, maximum_output_bytes, maximum_elapsed)?;
    Ok((directory, identity))
}

#[cfg(unix)]
pub(crate) fn identity_from_file(
    file: &File,
    _path: &Path,
    _maximum_output_bytes: u64,
    _maximum_elapsed: Duration,
) -> io::Result<PhysicalFileIdentity> {
    use std::os::unix::fs::MetadataExt;
    let metadata = file.metadata()?;
    Ok(PhysicalFileIdentity::Unix {
        device: metadata.dev(),
        inode: metadata.ino(),
    })
}

#[cfg(unix)]
pub(crate) fn identity_from_path(
    _file: &File,
    path: &Path,
    _maximum_output_bytes: u64,
    _maximum_elapsed: Duration,
) -> io::Result<PhysicalFileIdentity> {
    use std::os::unix::fs::MetadataExt;
    let metadata = path.symlink_metadata()?;
    Ok(PhysicalFileIdentity::Unix {
        device: metadata.dev(),
        inode: metadata.ino(),
    })
}

#[cfg(windows)]
pub(crate) fn identity_from_file(
    file: &File,
    path: &Path,
    maximum_output_bytes: u64,
    maximum_elapsed: Duration,
) -> io::Result<PhysicalFileIdentity> {
    use std::time::Instant;

    if maximum_output_bytes < WINDOWS_HIGH_RES_IDENTITY_BYTES || maximum_elapsed.is_zero() {
        return Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "identity budget exhausted",
        ));
    }
    file.metadata()?;
    let started = Instant::now();
    // The media walk keeps its read handle and all directory guards open with
    // sharing modes that deny deletion. The pathname therefore cannot be
    // replaced while this independent, high-resolution path query is made.
    let identity = file_id::get_high_res_file_id(path)?;
    if started.elapsed() >= maximum_elapsed {
        return Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "identity query timed out",
        ));
    }
    match identity {
        file_id::FileId::HighRes {
            volume_serial_number,
            file_id,
        } => Ok(PhysicalFileIdentity::Windows {
            volume_serial_number,
            file_identifier: file_id.to_le_bytes(),
        }),
        _ => Err(io::Error::other(
            "high-resolution Windows file identity unavailable",
        )),
    }
}

#[cfg(not(any(unix, windows)))]
pub(crate) fn identity_from_file(
    file: &File,
    _path: &Path,
    _maximum_output_bytes: u64,
    _maximum_elapsed: Duration,
) -> io::Result<PhysicalFileIdentity> {
    Ok(portable_identity(&file.metadata()?))
}

#[cfg(not(any(unix, windows)))]
pub(crate) fn identity_from_path(
    _file: &File,
    path: &Path,
    _maximum_output_bytes: u64,
    _maximum_elapsed: Duration,
) -> io::Result<PhysicalFileIdentity> {
    Ok(portable_identity(&path.symlink_metadata()?))
}

#[cfg(not(any(unix, windows)))]
fn portable_identity(metadata: &Metadata) -> PhysicalFileIdentity {
    let modified_nanos = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map_or(0, |duration| duration.as_nanos());
    PhysicalFileIdentity::Portable {
        bytes: metadata.len(),
        modified_nanos,
    }
}

pub(crate) fn same_snapshot(left: &Metadata, right: &Metadata) -> bool {
    left.len() == right.len()
        && left.modified().ok() == right.modified().ok()
        && left.created().ok() == right.created().ok()
}
