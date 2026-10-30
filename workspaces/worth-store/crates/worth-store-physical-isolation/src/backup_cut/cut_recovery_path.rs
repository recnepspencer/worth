use std::ffi::OsString;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RecoveryPathCodecDenial {
    InvalidEncoding,
    UnsupportedPlatform,
    AllocationFailed,
}

#[cfg(unix)]
pub(super) fn encode_recovery_path(path: &Path) -> Result<(u8, Vec<u8>), RecoveryPathCodecDenial> {
    use std::os::unix::ffi::OsStrExt;

    copy_bytes(path.as_os_str().as_bytes()).map(|bytes| (1, bytes))
}

#[cfg(windows)]
pub(super) fn encode_recovery_path(path: &Path) -> Result<(u8, Vec<u8>), RecoveryPathCodecDenial> {
    use std::os::windows::ffi::OsStrExt;

    let unit_count = path.as_os_str().encode_wide().count();
    let byte_count = unit_count
        .checked_mul(2)
        .ok_or(RecoveryPathCodecDenial::InvalidEncoding)?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(byte_count)
        .map_err(|_| RecoveryPathCodecDenial::AllocationFailed)?;
    for unit in path.as_os_str().encode_wide() {
        bytes.extend_from_slice(&unit.to_le_bytes());
    }
    Ok((2, bytes))
}

#[cfg(not(any(unix, windows)))]
pub(super) fn encode_recovery_path(_path: &Path) -> Result<(u8, Vec<u8>), RecoveryPathCodecDenial> {
    Err(RecoveryPathCodecDenial::UnsupportedPlatform)
}

pub(super) fn decode_recovery_path(
    platform: u8,
    bytes: &[u8],
) -> Result<PathBuf, RecoveryPathCodecDenial> {
    match platform {
        1 => decode_unix_path(bytes),
        2 => decode_windows_path(bytes),
        _ => Err(RecoveryPathCodecDenial::InvalidEncoding),
    }
}

#[cfg(unix)]
fn decode_unix_path(bytes: &[u8]) -> Result<PathBuf, RecoveryPathCodecDenial> {
    use std::os::unix::ffi::OsStringExt;

    Ok(PathBuf::from(OsString::from_vec(copy_bytes(bytes)?)))
}

#[cfg(not(unix))]
fn decode_unix_path(_bytes: &[u8]) -> Result<PathBuf, RecoveryPathCodecDenial> {
    Err(RecoveryPathCodecDenial::UnsupportedPlatform)
}

#[cfg(windows)]
fn decode_windows_path(bytes: &[u8]) -> Result<PathBuf, RecoveryPathCodecDenial> {
    use std::os::windows::ffi::OsStringExt;

    if !bytes.len().is_multiple_of(2) {
        return Err(RecoveryPathCodecDenial::InvalidEncoding);
    }
    let mut units = Vec::new();
    units
        .try_reserve_exact(bytes.len() / 2)
        .map_err(|_| RecoveryPathCodecDenial::AllocationFailed)?;
    units.extend(
        bytes
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]])),
    );
    Ok(PathBuf::from(OsString::from_wide(&units)))
}

#[cfg(not(windows))]
fn decode_windows_path(_bytes: &[u8]) -> Result<PathBuf, RecoveryPathCodecDenial> {
    Err(RecoveryPathCodecDenial::UnsupportedPlatform)
}

#[cfg(unix)]
fn copy_bytes(bytes: &[u8]) -> Result<Vec<u8>, RecoveryPathCodecDenial> {
    let mut copied = Vec::new();
    copied
        .try_reserve_exact(bytes.len())
        .map_err(|_| RecoveryPathCodecDenial::AllocationFailed)?;
    copied.extend_from_slice(bytes);
    Ok(copied)
}
