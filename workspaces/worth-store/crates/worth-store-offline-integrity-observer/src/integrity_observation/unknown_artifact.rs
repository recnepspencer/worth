use std::path::Path;

use worth_foundational::{PhysicalArtifactGeneration, PhysicalArtifactIdentity};

use super::{
    sha256::sha256, BoundedMediaWalk, OfflineArtifactDuplicateEvidence, OfflineArtifactFamily,
    OfflineArtifactObservation,
};

pub(crate) fn unknown_artifact(
    root: &Path,
    path: &Path,
    depth: u32,
    walk: &mut BoundedMediaWalk,
) -> OfflineArtifactObservation {
    let relative = relative_path(root, path);
    let classification = walk.classify_unrecognized(path, depth);
    let mut observation = OfflineArtifactObservation::new(
        relative.clone(),
        OfflineArtifactFamily::Unrecognized,
        PhysicalArtifactIdentity::new(format!("unrecognized:{}", path_digest(root, path)))
            .expect("fixed-size unrecognized identity"),
        PhysicalArtifactGeneration::NotEncoded,
        None,
        classification.outcome,
    );
    if let Some(first_path) = classification.physical_alias_of {
        observation = observation.with_duplicate(OfflineArtifactDuplicateEvidence::PhysicalAlias {
            first_path: relative_path(root, &first_path).into_boxed_str(),
        });
    }
    observation
}

pub(crate) fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .map(|component| encode_component(component.as_os_str()))
        .collect::<Vec<_>>()
        .join("/")
}

fn encode_component(component: &std::ffi::OsStr) -> String {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        encode_native_units(component.as_bytes(), "~b")
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        let units: Vec<u16> = component.encode_wide().collect();
        encode_windows_units(&units)
    }
    #[cfg(not(any(unix, windows)))]
    encode_native_units(component.to_string_lossy().as_bytes(), "~b")
}

#[cfg(windows)]
fn encode_windows_units(units: &[u16]) -> String {
    if units.iter().all(|unit| {
        u8::try_from(*unit).is_ok_and(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte))
    }) && units.first() != Some(&u16::from(b'~'))
    {
        return units
            .iter()
            .map(|unit| char::from_u32(u32::from(*unit)).expect("safe path unit is ASCII"))
            .collect();
    }
    let mut encoded = String::from("~w");
    for unit in units {
        use std::fmt::Write;
        let _ = write!(encoded, "{unit:04x}");
    }
    encoded
}

#[cfg(not(windows))]
fn encode_native_units(units: &[u8], prefix: &str) -> String {
    if units
        .iter()
        .all(|unit| unit.is_ascii_alphanumeric() || b"._-".contains(unit))
        && units.first() != Some(&b'~')
    {
        return String::from_utf8(units.to_vec()).expect("safe native path component is ASCII");
    }
    let mut encoded = String::from(prefix);
    for unit in units {
        use std::fmt::Write;
        let _ = write!(encoded, "{unit:02x}");
    }
    encoded
}

fn path_digest(root: &Path, path: &Path) -> String {
    let relative = path.strip_prefix(root).unwrap_or(path).as_os_str();
    #[cfg(unix)]
    let bytes = {
        use std::os::unix::ffi::OsStrExt;
        relative.as_bytes().to_vec()
    };
    #[cfg(windows)]
    let bytes = {
        use std::os::windows::ffi::OsStrExt;
        relative
            .encode_wide()
            .flat_map(u16::to_le_bytes)
            .collect::<Vec<_>>()
    };
    #[cfg(not(any(unix, windows)))]
    let bytes = relative.to_string_lossy().as_bytes().to_vec();
    hex_bytes(&sha256(&bytes))
}

fn hex_bytes(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        result.push(HEX[(byte >> 4) as usize] as char);
        result.push(HEX[(byte & 0x0f) as usize] as char);
    }
    result
}

#[cfg(all(test, windows))]
mod windows_path_tests {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;

    use super::encode_component;

    #[test]
    fn hostile_windows_units_are_lossless_and_distinct() {
        let left = OsString::from_wide(&[u16::from(b'x'), 0xd800]);
        let right = OsString::from_wide(&[u16::from(b'x'), 0xd801]);
        assert_eq!(encode_component(&left), "~w0078d800");
        assert_eq!(encode_component(&right), "~w0078d801");
        assert_ne!(encode_component(&left), encode_component(&right));
    }
}
