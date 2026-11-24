//! Deterministic build identity resolution with explicit environment override.
//!
//! Applications normally embed a CI/release UUID via `WORTH_IMPLEMENTATION_BUILD_ID`.
//! When that environment variable is absent during local development, this helper
//! derives a deterministic nonzero RFC 4122 UUID from the Cargo package identity
//! (`<CARGO_PKG_NAME>@<CARGO_PKG_VERSION>`) without requiring a hand-written `build.rs`.

use sha2::{Digest, Sha256};
use worth_foundational::facade::{ApplicationBuildIdentity, InvalidApplicationBuildIdentity};

const PACKAGE_DEFAULT_DOMAIN: &[u8] = b"worth.application.build-identity.package-default.v1\0";

/// Pure constructor namespace for deterministic Home build identities.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryApplicationHomeBuildIdentity;

impl WorthQueryApplicationHomeBuildIdentity {
    /// Resolve an [`ApplicationBuildIdentity`] honoring an explicit
    /// `WORTH_IMPLEMENTATION_BUILD_ID` override when present and falling back to
    /// a deterministic package-derived UUID when unset.
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

    /// Derive a deterministic nonzero UUID from a non-empty package descriptor
    /// such as `"daymark@0.3.0"`.
    pub fn from_package_descriptor(
        package_descriptor: &str,
    ) -> Result<ApplicationBuildIdentity, InvalidApplicationBuildIdentity> {
        if package_descriptor.is_empty() {
            return Err(InvalidApplicationBuildIdentity::Missing);
        }
        let mut hasher = Sha256::new();
        hasher.update(PACKAGE_DEFAULT_DOMAIN);
        hasher.update((package_descriptor.len() as u64).to_le_bytes());
        hasher.update(package_descriptor.as_bytes());
        let digest = hasher.finalize();

        let mut bytes = [0u8; 16];
        bytes.copy_from_slice(&digest[..16]);
        // RFC 4122 version 5 (name-based) and standard variant bits guarantee nonzero bytes.
        bytes[6] = (bytes[6] & 0x0f) | 0x50;
        bytes[8] = (bytes[8] & 0x3f) | 0x80;

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
