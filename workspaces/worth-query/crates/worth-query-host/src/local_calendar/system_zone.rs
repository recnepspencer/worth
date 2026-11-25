//! Operating-system zone boundary: the `TZ` environment variable, the zone
//! database under `/usr/share/zoneinfo`, and `/etc/localtime`.
use super::posix_rule::PosixRule;
use super::{WorthQueryLocalCalendarDenial as Denial, WorthQueryTimeZone};
use std::path::{Component, Path, PathBuf};

const LOCALTIME: &str = "/etc/localtime";
const ZONEINFO: &str = "/usr/share/zoneinfo";

pub(super) fn load() -> Result<WorthQueryTimeZone, Denial> {
    match std::env::var("TZ") {
        Ok(variable) => from_variable(&variable),
        Err(_) => read_zone_file(Path::new(LOCALTIME)),
    }
}

/// `TZ` is empty (UTC), a `:`-prefixed or plain zone name or path, or a
/// POSIX rule.
fn from_variable(variable: &str) -> Result<WorthQueryTimeZone, Denial> {
    let name = variable.strip_prefix(':').unwrap_or(variable);
    if name.is_empty() {
        return Ok(WorthQueryTimeZone::UTC);
    }
    if Path::new(name).is_absolute() {
        return read_zone_file(Path::new(name));
    }
    match zoneinfo_path(name).map(|path| read_zone_file(&path)) {
        Some(Ok(zone)) => Ok(zone),
        Some(Err(unreadable)) => PosixRule::parse(name)
            .map(WorthQueryTimeZone::posix)
            .map_err(|_| unreadable),
        None => PosixRule::parse(name).map(WorthQueryTimeZone::posix),
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
    let bytes = std::fs::read(path).map_err(|cause| Denial::SystemZoneUnreadable(cause.kind()))?;
    WorthQueryTimeZone::from_tzif(&bytes)
}
