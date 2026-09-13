use std::fs::{File, Metadata, OpenOptions};
use std::io;
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum PhysicalFileIdentity {
    #[cfg(unix)]
    Unix { device: u64, inode: u64 },
    #[cfg(windows)]
    Windows {
        volume_serial_number: u32,
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
    use std::path::{Component, Prefix};
    use std::process::Command;
    use std::time::Instant;

    file.metadata()?;
    let canonical = path.canonicalize()?;
    let drive = match canonical.components().next() {
        Some(Component::Prefix(prefix)) => match prefix.kind() {
            Prefix::Disk(letter) | Prefix::VerbatimDisk(letter) => {
                format!("{}:", char::from(letter))
            }
            _ => return Err(io::Error::other("unsupported Windows volume path")),
        },
        _ => return Err(io::Error::other("Windows path has no volume prefix")),
    };
    let started = Instant::now();
    let file_id = bounded_command_output(
        Command::new("fsutil")
            .args(["file", "queryFileID"])
            .arg(&canonical),
        maximum_output_bytes,
        maximum_elapsed,
    )?;
    let elapsed = maximum_elapsed.saturating_sub(started.elapsed());
    let volume = bounded_command_output(
        Command::new("cmd").args(["/d", "/c", "vol", &drive]),
        maximum_output_bytes,
        elapsed,
    )?;
    Ok(PhysicalFileIdentity::Windows {
        volume_serial_number: parse_volume_serial(&volume)?,
        file_identifier: parse_file_identifier(&file_id)?,
    })
}

#[cfg(windows)]
fn bounded_command_output(
    command: &mut std::process::Command,
    maximum_output_bytes: u64,
    maximum_elapsed: Duration,
) -> io::Result<Vec<u8>> {
    use std::io::Read;
    use std::process::Stdio;
    use std::time::Instant;

    if maximum_output_bytes == 0 || maximum_elapsed.is_zero() {
        return Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "identity budget exhausted",
        ));
    }
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| io::Error::other("stdout unavailable"))?;
    let read_limit = maximum_output_bytes.min(4_096).saturating_add(1);
    let reader = std::thread::spawn(move || {
        let mut output = Vec::new();
        stdout
            .take(read_limit)
            .read_to_end(&mut output)
            .map(|_| output)
    });
    let started = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if started.elapsed() >= maximum_elapsed {
            let _ = child.kill();
            let _ = child.wait();
            let _ = reader.join();
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "identity query timed out",
            ));
        }
        std::thread::sleep(Duration::from_millis(2));
    };
    let output = reader
        .join()
        .map_err(|_| io::Error::other("identity reader failed"))??;
    if !status.success() || output.len() as u64 > maximum_output_bytes.min(4_096) {
        return Err(io::Error::other(
            "identity query failed or exceeded output bound",
        ));
    }
    Ok(output)
}

#[cfg(windows)]
fn parse_file_identifier(output: &[u8]) -> io::Result<[u8; 16]> {
    let text = String::from_utf8_lossy(output);
    let hex = text
        .split("0x")
        .nth(1)
        .map(str::trim)
        .filter(|value| value.len() == 32 && value.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .ok_or_else(|| io::Error::other("malformed Windows file identifier"))?;
    let mut identifier = [0_u8; 16];
    for (index, byte) in identifier.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16)
            .map_err(|_| io::Error::other("malformed Windows file identifier"))?;
    }
    Ok(identifier)
}

#[cfg(windows)]
fn parse_volume_serial(output: &[u8]) -> io::Result<u32> {
    let text = String::from_utf8_lossy(output);
    let serial = text
        .split_ascii_whitespace()
        .find(|value| {
            value.len() == 9
                && value.as_bytes().get(4) == Some(&b'-')
                && value
                    .bytes()
                    .enumerate()
                    .all(|(index, byte)| index == 4 || byte.is_ascii_hexdigit())
        })
        .ok_or_else(|| io::Error::other("malformed Windows volume serial"))?;
    u32::from_str_radix(&serial.replace('-', ""), 16)
        .map_err(|_| io::Error::other("malformed Windows volume serial"))
}

#[cfg(windows)]
pub(crate) fn identity_from_path(
    file: &File,
    path: &Path,
    maximum_output_bytes: u64,
    maximum_elapsed: Duration,
) -> io::Result<PhysicalFileIdentity> {
    identity_from_file(file, path, maximum_output_bytes, maximum_elapsed)
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
