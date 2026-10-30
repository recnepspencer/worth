use std::path::{Path, PathBuf};

const MAX_OPERATIONAL_PATH_BYTES: usize = 32 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OperationalMediaPathDenial {
    Empty,
    TooLarge,
    AllocationFailed,
    UnsupportedPlatform,
    InvalidEncoding,
}

pub(crate) fn resolve_operational_media_path(path: &Path) -> Result<PathBuf, std::io::Error> {
    validate_operational_media_path_size(path).map_err(path_size_io_error)?;
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut existing = absolute.clone();
    let mut suffix = Vec::new();
    while !existing.exists() {
        let Some(name) = existing.file_name().map(ToOwned::to_owned) else {
            break;
        };
        suffix
            .try_reserve(1)
            .map_err(|_| std::io::Error::from(std::io::ErrorKind::OutOfMemory))?;
        suffix.push(name);
        if !existing.pop() {
            break;
        }
    }
    let mut resolved = std::fs::canonicalize(existing)?;
    for component in suffix.into_iter().rev() {
        resolved.push(component);
    }
    Ok(resolved)
}

#[cfg(unix)]
pub(crate) fn encode_operational_media_path(
    path: &Path,
) -> Result<(u8, Vec<u8>), OperationalMediaPathDenial> {
    use std::os::unix::ffi::OsStrExt;

    copy_path_bytes(1, path.as_os_str().as_bytes())
}

#[cfg(windows)]
pub(crate) fn encode_operational_media_path(
    path: &Path,
) -> Result<(u8, Vec<u8>), OperationalMediaPathDenial> {
    use std::os::windows::ffi::OsStrExt;

    let units = path.as_os_str().encode_wide().count();
    let byte_count = units
        .checked_mul(2)
        .ok_or(OperationalMediaPathDenial::TooLarge)?;
    if byte_count > MAX_OPERATIONAL_PATH_BYTES {
        return Err(OperationalMediaPathDenial::TooLarge);
    }
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(byte_count)
        .map_err(|_| OperationalMediaPathDenial::AllocationFailed)?;
    for unit in path.as_os_str().encode_wide() {
        bytes.extend_from_slice(&unit.to_le_bytes());
    }
    validate_encoded_path(2, bytes)
}

#[cfg(not(any(unix, windows)))]
pub(crate) fn encode_operational_media_path(
    _path: &Path,
) -> Result<(u8, Vec<u8>), OperationalMediaPathDenial> {
    Err(OperationalMediaPathDenial::UnsupportedPlatform)
}

#[cfg(unix)]
pub(crate) fn decode_operational_media_path(
    platform: u8,
    bytes: &[u8],
) -> Result<PathBuf, OperationalMediaPathDenial> {
    use std::os::unix::ffi::OsStringExt;

    validate_path_bytes(platform, bytes, 1)?;
    let mut owned = Vec::new();
    owned
        .try_reserve_exact(bytes.len())
        .map_err(|_| OperationalMediaPathDenial::AllocationFailed)?;
    owned.extend_from_slice(bytes);
    Ok(PathBuf::from(std::ffi::OsString::from_vec(owned)))
}

#[cfg(windows)]
pub(crate) fn decode_operational_media_path(
    platform: u8,
    bytes: &[u8],
) -> Result<PathBuf, OperationalMediaPathDenial> {
    use std::os::windows::ffi::OsStringExt;

    validate_path_bytes(platform, bytes, 2)?;
    if !bytes.len().is_multiple_of(2) {
        return Err(OperationalMediaPathDenial::InvalidEncoding);
    }
    let mut units = Vec::new();
    units
        .try_reserve_exact(bytes.len() / 2)
        .map_err(|_| OperationalMediaPathDenial::AllocationFailed)?;
    units.extend(
        bytes
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]])),
    );
    Ok(PathBuf::from(std::ffi::OsString::from_wide(&units)))
}

#[cfg(not(any(unix, windows)))]
pub(crate) fn decode_operational_media_path(
    _platform: u8,
    _bytes: &[u8],
) -> Result<PathBuf, OperationalMediaPathDenial> {
    Err(OperationalMediaPathDenial::UnsupportedPlatform)
}

#[cfg(unix)]
fn copy_path_bytes(
    platform: u8,
    source: &[u8],
) -> Result<(u8, Vec<u8>), OperationalMediaPathDenial> {
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(source.len())
        .map_err(|_| OperationalMediaPathDenial::AllocationFailed)?;
    bytes.extend_from_slice(source);
    validate_encoded_path(platform, bytes)
}

fn validate_encoded_path(
    platform: u8,
    bytes: Vec<u8>,
) -> Result<(u8, Vec<u8>), OperationalMediaPathDenial> {
    validate_path_bytes(platform, &bytes, platform)?;
    Ok((platform, bytes))
}

fn validate_path_bytes(
    platform: u8,
    bytes: &[u8],
    expected_platform: u8,
) -> Result<(), OperationalMediaPathDenial> {
    if platform != expected_platform {
        return Err(OperationalMediaPathDenial::UnsupportedPlatform);
    }
    if bytes.is_empty() {
        return Err(OperationalMediaPathDenial::Empty);
    }
    if bytes.len() > MAX_OPERATIONAL_PATH_BYTES {
        return Err(OperationalMediaPathDenial::TooLarge);
    }
    Ok(())
}

#[cfg(unix)]
fn validate_operational_media_path_size(path: &Path) -> Result<(), OperationalMediaPathDenial> {
    use std::os::unix::ffi::OsStrExt;

    validate_path_bytes(1, path.as_os_str().as_bytes(), 1)
}

#[cfg(windows)]
fn validate_operational_media_path_size(path: &Path) -> Result<(), OperationalMediaPathDenial> {
    use std::os::windows::ffi::OsStrExt;

    let units = path.as_os_str().encode_wide().count();
    let bytes = units
        .checked_mul(2)
        .ok_or(OperationalMediaPathDenial::TooLarge)?;
    if bytes == 0 {
        Err(OperationalMediaPathDenial::Empty)
    } else if bytes > MAX_OPERATIONAL_PATH_BYTES {
        Err(OperationalMediaPathDenial::TooLarge)
    } else {
        Ok(())
    }
}

#[cfg(not(any(unix, windows)))]
fn validate_operational_media_path_size(_path: &Path) -> Result<(), OperationalMediaPathDenial> {
    Err(OperationalMediaPathDenial::UnsupportedPlatform)
}

fn path_size_io_error(denial: OperationalMediaPathDenial) -> std::io::Error {
    let kind = match denial {
        OperationalMediaPathDenial::AllocationFailed => std::io::ErrorKind::OutOfMemory,
        _ => std::io::ErrorKind::InvalidInput,
    };
    std::io::Error::new(kind, "operational media path is not representable")
}
