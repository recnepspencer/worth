//! The observer this build compiles declares its visibility-transition
//! mechanism; the qualified profile record this build certifies under
//! declares one too. They are two spellings of one fact, so a lane whose
//! observer changes mechanism without the record following (or the reverse)
//! fails here, at the compiler's choice of arm, not in a courtroom deadline.
use crate::external_observation::NativeWindowVisibilityTransitionMechanism;

use super::{CertifiedNativePlatform, CERTIFIED_SCALE_MILLI};

/// Every qualified profile, so the vocabulary check below is target-invariant:
/// a Linux run still proves the Windows record names a known mechanism.
pub(super) const QUALIFIED_PROFILE_MANIFESTS: [(&str, &str); 4] = [
    (
        "worth-ui-windows-dx12-v2",
        include_str!(concat!(
            "../../../../../crates/worth-ui-host-native/profiles",
            "/worth-ui-windows-dx12-v2.toml"
        )),
    ),
    (
        "worth-ui-linux-wayland-vulkan-v1",
        include_str!(concat!(
            "../../../../../crates/worth-ui-host-native/profiles",
            "/worth-ui-linux-wayland-vulkan-v1.toml"
        )),
    ),
    (
        "worth-ui-linux-x11-vulkan-v1",
        include_str!(concat!(
            "../../../../../crates/worth-ui-host-native/profiles",
            "/worth-ui-linux-x11-vulkan-v1.toml"
        )),
    ),
    (
        "worth-ui-linux-x11-vulkan-software-v1",
        include_str!(concat!(
            "../../../../../crates/worth-ui-host-native/profiles",
            "/worth-ui-linux-x11-vulkan-software-v1.toml"
        )),
    ),
];

/// The record for the certified arm, chosen the way `build.rs` chooses the
/// alias: Windows certifies under DX12 v2, Linux under the X11 software
/// profile (the only Linux profile that presents under Xvfb).
#[cfg(target_os = "windows")]
const CERTIFIED_PROFILE: &str = "worth-ui-windows-dx12-v2";
#[cfg(target_os = "linux")]
const CERTIFIED_PROFILE: &str = "worth-ui-linux-x11-vulkan-software-v1";

const VISIBILITY_AXIS: &str = "client_visibility_transition_observation";
const SCALES_AXIS: &str = "appearance_qualified_scales";
const UNOBSERVED: &str = "unobserved";

pub(super) fn manifest_string(manifest: &str, key: &str) -> Option<String> {
    manifest.lines().find_map(|line| {
        let (name, value) = line.split_once('=')?;
        (name.trim() == key).then(|| value.trim().trim_matches('"').to_owned())
    })
}

/// A manifest scale spelling (`1.25`) in milli, digit by digit: the record
/// is exact text and must not be compared through a float.
fn scale_milli(spelling: &str) -> Option<u32> {
    let (whole, fraction) = spelling.split_once('.').unwrap_or((spelling, ""));
    if fraction.len() > 3 {
        return None;
    }
    let whole: u32 = whole.parse().ok()?;
    let fraction: u32 = if fraction.is_empty() {
        0
    } else {
        format!("{fraction:0<3}").parse().ok()?
    };
    Some(whole * 1_000 + fraction)
}

pub(super) fn certified_manifest() -> &'static str {
    QUALIFIED_PROFILE_MANIFESTS
        .iter()
        .find(|(identity, _)| *identity == CERTIFIED_PROFILE)
        .map(|(_, manifest)| *manifest)
        .expect("the certified arm's profile is qualified")
}

#[test]
fn the_observer_declares_the_visibility_mechanism_the_qualified_record_carries() {
    let declared = manifest_string(certified_manifest(), VISIBILITY_AXIS)
        .expect("the certified profile records a visibility transition observation");
    let mechanism = CertifiedNativePlatform::VISIBILITY_TRANSITION_MECHANISM.declared_as();
    assert!(
        declared.ends_with(&format!(";{mechanism}")),
        "record says {declared:?}, observer actuates {mechanism:?}"
    );
}

/// The axis is a closed vocabulary: every profile either declares the
/// transition unobserved or names exactly one mechanism an observer can
/// actuate. A record naming a mechanism no observer implements would
/// certify a transition nothing can produce.
#[test]
fn every_qualified_record_names_a_known_mechanism_or_declares_it_unobserved() {
    for (identity, manifest) in QUALIFIED_PROFILE_MANIFESTS {
        let declared = manifest_string(manifest, VISIBILITY_AXIS)
            .unwrap_or_else(|| panic!("{identity} declares {VISIBILITY_AXIS}"));
        let named = NativeWindowVisibilityTransitionMechanism::ALL
            .iter()
            .filter(|mechanism| declared.ends_with(&format!(";{}", mechanism.declared_as())))
            .count();
        assert!(
            (declared == UNOBSERVED && named == 0) || named == 1,
            "{identity}: {declared:?} names {named} known mechanism(s)"
        );
    }
    let spellings: std::collections::BTreeSet<&str> =
        NativeWindowVisibilityTransitionMechanism::ALL
            .iter()
            .map(|mechanism| mechanism.declared_as())
            .collect();
    assert_eq!(
        spellings.len(),
        NativeWindowVisibilityTransitionMechanism::ALL.len()
    );
}

#[test]
fn a_record_axis_is_read_by_exact_key_not_by_substring() {
    let manifest = "client_visibility_transition_observation_note = \"x\"\nclient_visibility_transition_observation = \"a;b\"\n";
    assert_eq!(
        manifest_string(manifest, VISIBILITY_AXIS).as_deref(),
        Some("a;b")
    );
    assert_eq!(manifest_string(manifest, "absent"), None);
}

/// The lane executes one scale row; it must be a row the record certifies,
/// or the courtroom would be asserting pixels no profile qualified.
#[test]
fn the_certified_scale_is_a_qualified_row_of_the_certified_record() {
    let declared = manifest_string(certified_manifest(), SCALES_AXIS)
        .expect("the certified profile records its qualified scales");
    let rows: Vec<u32> = declared
        .split(';')
        .map(|row| scale_milli(row).unwrap_or_else(|| panic!("{row:?} is not a scale")))
        .collect();
    assert!(
        rows.contains(&CERTIFIED_SCALE_MILLI),
        "record rows {rows:?} lack the certified {CERTIFIED_SCALE_MILLI}"
    );
}

#[test]
fn scale_spellings_convert_to_milli_exactly() {
    assert_eq!(scale_milli("1.0"), Some(1_000));
    assert_eq!(scale_milli("1.25"), Some(1_250));
    assert_eq!(scale_milli("1.5"), Some(1_500));
    assert_eq!(scale_milli("2.0"), Some(2_000));
    assert_eq!(scale_milli("2"), Some(2_000));
    assert_eq!(scale_milli("1.2345"), None);
    assert_eq!(scale_milli("x"), None);
}
