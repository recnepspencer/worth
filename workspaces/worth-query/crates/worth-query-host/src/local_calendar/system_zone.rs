//! Operating-system zone boundary: the `TZ` environment variable, the zone
//! database under `/usr/share/zoneinfo`, and the platform's default zone:
//! `/etc/localtime`, or on Windows the registry's zone record.
use super::posix_rule::PosixRule;
use super::{WorthQueryLocalCalendarDenial as Denial, WorthQueryTimeZone};
use std::env::VarError;
use std::io::Read;
use std::path::{Component, Path, PathBuf};

const LOCALTIME: &str = "/etc/localtime";
const ZONEINFO: &str = "/usr/share/zoneinfo";
/// Real TZif files are a few KiB; a larger or special file is refused.
const MAX_TZIF_BYTES: u64 = 1 << 20;

pub(super) fn load() -> Result<WorthQueryTimeZone, Denial> {
    match std::env::var("TZ") {
        Ok(variable) => from_environment(Some(&variable)),
        Err(VarError::NotPresent) => from_environment(None),
        Err(VarError::NotUnicode(_)) => Err(Denial::MalformedZone),
    }
}

/// A `TZ` value, or the platform default when `TZ` is unset.
pub(super) fn from_environment(variable: Option<&str>) -> Result<WorthQueryTimeZone, Denial> {
    variable.map_or_else(platform_default, from_variable)
}

/// The zone the operating system is set to when `TZ` names none.
#[cfg(not(windows))]
fn platform_default() -> Result<WorthQueryTimeZone, Denial> {
    read_zone_file(Path::new(LOCALTIME))
}

/// The zone the operating system is set to when `TZ` names none.
#[cfg(windows)]
fn platform_default() -> Result<WorthQueryTimeZone, Denial> {
    super::windows_zone::load()
}

/// Resolves a `TZ` value as `tzset(3)` does: empty is UTC; `:` alone is the
/// platform default; `:name` is only a zone file or database name; a bare
/// value is a zone file or database name, else a POSIX rule.
pub(super) fn from_variable(variable: &str) -> Result<WorthQueryTimeZone, Denial> {
    if variable.is_empty() {
        return Ok(WorthQueryTimeZone::UTC);
    }
    let (name, file_only) = match variable.strip_prefix(':') {
        Some("") => return platform_default(),
        Some(name) => (name, true),
        None => (variable, false),
    };
    let file = if Path::new(name).is_absolute() {
        read_zone_file(Path::new(name))
    } else {
        zoneinfo_path(name).map_or(Err(Denial::MalformedZone), |path| read_zone_file(&path))
    };
    match file {
        Ok(zone) => Ok(zone),
        Err(denial) if file_only => Err(denial),
        Err(denial) => PosixRule::parse(name)
            .map(WorthQueryTimeZone::posix)
            .map_err(|_| denial),
    }
}

/// A zone database path for `name`; names that climb out of it are refused.
fn zoneinfo_path(name: &str) -> Option<PathBuf> {
    let relative = Path::new(name);
    relative
        .components()
        .all(|component| matches!(component, Component::Normal(_)))
        .then(|| Path::new(ZONEINFO).join(relative))
}

fn read_zone_file(path: &Path) -> Result<WorthQueryTimeZone, Denial> {
    let unreadable = |cause: std::io::Error| Denial::SystemZoneUnreadable(cause.kind());
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(unreadable)?
        .take(MAX_TZIF_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(unreadable)?;
    if bytes.len() as u64 > MAX_TZIF_BYTES {
        return Err(Denial::MalformedZone);
    }
    WorthQueryTimeZone::from_tzif(&bytes)
}
