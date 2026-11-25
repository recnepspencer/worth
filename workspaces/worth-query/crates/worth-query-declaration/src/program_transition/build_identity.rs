//! Deterministic build identity resolution with explicit environment override.
//!
//! Applications normally embed a CI/release UUID via `WORTH_IMPLEMENTATION_BUILD_ID`.
//! When that environment variable is absent during local development, this helper
//! derives a genuine RFC 4122 §4.3 UUIDv5 (SHA-1 over the fixed WORTH namespace
//! UUID concatenated with `<CARGO_PKG_NAME>@<CARGO_PKG_VERSION>`) without
//! requiring a hand-written `build.rs`.

use worth_foundational::facade::{ApplicationBuildIdentity, InvalidApplicationBuildIdentity};

use super::sha1::uuid_v5;

/// Fixed RFC 4122 namespace UUID for WORTH package-default build identities:
/// `7c3b5e12-4a91-4d8e-9b63-2f8a1e5c7d40`.
pub const WORTH_PACKAGE_BUILD_ID_NAMESPACE: [u8; 16] = [
    0x7c, 0x3b, 0x5e, 0x12, 0x4a, 0x91, 0x4d, 0x8e, 0x9b, 0x63, 0x2f, 0x8a, 0x1e, 0x5c, 0x7d, 0x40,
];

/// Pure constructor namespace for deterministic Home build identities.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryApplicationHomeBuildIdentity;

impl WorthQueryApplicationHomeBuildIdentity {
    /// Resolve an [`ApplicationBuildIdentity`] honoring an explicit
    /// `WORTH_IMPLEMENTATION_BUILD_ID` override when present and falling back to
    /// a deterministic RFC 4122 §4.3 UUIDv5 when unset.
    ///
    /// Fails closed if `override_env` is present but invalid (empty, malformed,
    /// or all-zero), or if `override_env` is `None` and `package_descriptor` is empty.
    pub fn package_default(
        override_env: Option<&str>,
        package_descriptor: &str,
    ) -> Result<ApplicationBuildIdentity, InvalidApplicationBuildIdentity> {
        if let Some(explicit) = override_env {
            return ApplicationBuildIdentity::parse(explicit);
        }
        Self::from_package_descriptor(package_descriptor)
    }

    /// Resolve from separate Cargo package name and version strings.
    pub fn resolve(
        override_env: Option<&str>,
        package_name: &str,
        package_version: &str,
    ) -> Result<ApplicationBuildIdentity, InvalidApplicationBuildIdentity> {
        if let Some(explicit) = override_env {
            return ApplicationBuildIdentity::parse(explicit);
        }
        if package_name.is_empty() || package_version.is_empty() {
            return Err(InvalidApplicationBuildIdentity::Missing);
        }
        let descriptor = format!("{package_name}@{package_version}");
        Self::from_package_descriptor(&descriptor)
    }

    /// Derive a deterministic nonzero RFC 4122 §4.3 UUIDv5 from a non-empty
    /// package descriptor such as `"daymark@0.3.0"`.
    pub fn from_package_descriptor(
        package_descriptor: &str,
    ) -> Result<ApplicationBuildIdentity, InvalidApplicationBuildIdentity> {
        if package_descriptor.is_empty() {
            return Err(InvalidApplicationBuildIdentity::Missing);
        }
        Self::from_uuid_v5(
            WORTH_PACKAGE_BUILD_ID_NAMESPACE,
            package_descriptor.as_bytes(),
        )
    }

    /// Derive a deterministic RFC 4122 §4.3 UUIDv5 under an explicit 16-byte
    /// namespace UUID and name bytes.
    pub fn from_uuid_v5(
        namespace: [u8; 16],
        name: &[u8],
    ) -> Result<ApplicationBuildIdentity, InvalidApplicationBuildIdentity> {
        if name.is_empty() {
            return Err(InvalidApplicationBuildIdentity::Missing);
        }
        let bytes = uuid_v5(namespace, name);
        let formatted = format_uuid(bytes);
        ApplicationBuildIdentity::parse(&formatted)
    }
}

fn format_uuid(bytes: [u8; 16]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(36);
    for (index, byte) in bytes.into_iter().enumerate() {
        if matches!(index, 4 | 6 | 8 | 10) {
            out.push('-');
        }
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::WorthQueryApplicationHomeBuildIdentity as BuildId;
    use worth_foundational::facade::{ApplicationBuildIdentity, InvalidApplicationBuildIdentity};

    /// RFC 4122 Appendix C DNS namespace UUID (`6ba7b810-9dad-11d1-80b4-00c04fd430c8`).
    const RFC4122_DNS_NAMESPACE: [u8; 16] = [
        0x6b, 0xa7, 0xb8, 0x10, 0x9d, 0xad, 0x11, 0xd1, 0x80, 0xb4, 0x00, 0xc0, 0x4f, 0xd4, 0x30,
        0xc8,
    ];

    #[test]
    fn rfc4122_uuid_v5_matches_known_answer_vectors() {
        let python = BuildId::from_uuid_v5(RFC4122_DNS_NAMESPACE, b"python.org").unwrap();
        assert_eq!(
            python,
            ApplicationBuildIdentity::parse("886313e1-3b8a-5372-9b90-0c9aee199e5d").unwrap()
        );
        let example = BuildId::from_uuid_v5(RFC4122_DNS_NAMESPACE, b"www.example.com").unwrap();
        assert_eq!(
            example,
            ApplicationBuildIdentity::parse("2ed6657d-e927-568b-95e1-2665a8aea6a2").unwrap()
        );
    }

    #[test]
    fn explicit_env_override_takes_precedence_and_fails_closed_when_invalid() {
        let explicit = "12345678-1234-4321-8123-123456789abc";
        let resolved = BuildId::package_default(Some(explicit), "daymark@0.2.0").unwrap();
        assert_eq!(resolved, ApplicationBuildIdentity::parse(explicit).unwrap());

        assert_eq!(
            BuildId::package_default(Some("not-a-uuid"), "daymark@0.2.0"),
            Err(InvalidApplicationBuildIdentity::Malformed)
        );
        assert_eq!(
            BuildId::package_default(Some(""), "daymark@0.2.0"),
            Err(InvalidApplicationBuildIdentity::Missing)
        );
        assert_eq!(
            BuildId::package_default(
                Some("00000000-0000-0000-0000-000000000000"),
                "daymark@0.2.0",
            ),
            Err(InvalidApplicationBuildIdentity::Zero)
        );
    }

    #[test]
    fn package_default_is_deterministic_distinct_per_version_and_nonzero() {
        let v2_a = BuildId::package_default(None, "daymark@0.2.0").unwrap();
        let v2_b = BuildId::resolve(None, "daymark", "0.2.0").unwrap();
        let v3 = BuildId::package_default(None, "daymark@0.3.0").unwrap();

        assert_eq!(v2_a, v2_b);
        assert_ne!(v2_a, v3);
        assert_ne!(v2_a.bytes(), [0u8; 16]);
        assert_eq!(v2_a.bytes()[6] >> 4, 5);
        assert_eq!(v2_a.bytes()[8] >> 6, 0b10);
        assert_eq!(
            BuildId::package_default(None, ""),
            Err(InvalidApplicationBuildIdentity::Missing)
        );
        assert_eq!(
            BuildId::resolve(None, "", "0.2.0"),
            Err(InvalidApplicationBuildIdentity::Missing)
        );
    }
}
