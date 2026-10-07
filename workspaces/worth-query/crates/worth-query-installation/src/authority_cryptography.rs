use std::fmt::{Debug, Formatter};

use hmac::{Hmac, Mac};
use sha2::Sha256;
use subtle::ConstantTimeEq;
use zeroize::{Zeroize, ZeroizeOnDrop};

type HmacSha256 = Hmac<Sha256>;

const REDACTED_KEY: &str = "[REDACTED installation authority key]";

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub(crate) struct InstallationAuthorityRootKey([u8; 32]);

impl InstallationAuthorityRootKey {
    pub(crate) fn generate() -> Result<Self, ()> {
        Self::generate_with(|bytes| getrandom::fill(bytes).map_err(|_| ()))
    }

    fn generate_with(fill: impl FnOnce(&mut [u8; 32]) -> Result<(), ()>) -> Result<Self, ()> {
        let mut bytes = [0_u8; 32];
        fill(&mut bytes)?;
        Ok(Self(bytes))
    }

    #[cfg(test)]
    pub(crate) fn repeated_byte_for_test(byte: u8) -> Self {
        Self([byte; 32])
    }

    pub(crate) fn lineage(&self) -> InstallationAuthorityLineage {
        let transcript =
            AuthorityTranscript::with_key(&self.0, AuthoritySealDomain::InstallationRootLineage);
        InstallationAuthorityLineage(transcript.finish_bytes())
    }

    pub(crate) fn derive_package_key(
        &self,
        runtime_ordinal: u64,
        generation_ordinal: u64,
        package_identity: &[u8],
        admission_identity: &[u8],
    ) -> PackageAuthorityKey {
        let mut transcript =
            AuthorityTranscript::with_key(&self.0, AuthoritySealDomain::InstalledPackageKey);
        transcript.u64("runtime", runtime_ordinal);
        transcript.u64("generation", generation_ordinal);
        transcript.bytes("package", package_identity);
        transcript.bytes("admission", admission_identity);
        PackageAuthorityKey(transcript.finish_bytes())
    }
}

impl Debug for InstallationAuthorityRootKey {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(REDACTED_KEY)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct InstallationAuthorityLineage([u8; 32]);

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub(crate) struct PackageAuthorityKey([u8; 32]);

impl PackageAuthorityKey {
    pub(crate) fn matches(&self, other: &Self) -> bool {
        bool::from(self.0.ct_eq(&other.0))
    }
}

impl Debug for PackageAuthorityKey {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(REDACTED_KEY)
    }
}

impl PartialEq for PackageAuthorityKey {
    fn eq(&self, other: &Self) -> bool {
        self.matches(other)
    }
}

impl Eq for PackageAuthorityKey {}

#[derive(Clone, Eq, PartialEq)]
pub(crate) struct AuthoritySeal {
    bytes: [u8; 32],
    text: String,
}

impl AuthoritySeal {
    fn from_bytes(bytes: [u8; 32]) -> Self {
        let mut text = String::with_capacity(64);
        for byte in bytes {
            use std::fmt::Write as _;
            write!(&mut text, "{byte:02x}").expect("writing to a String cannot fail");
        }
        Self { bytes, text }
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.text
    }

    pub(crate) const fn bytes(&self) -> &[u8; 32] {
        &self.bytes
    }
}

impl Debug for AuthoritySeal {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_tuple("AuthoritySeal")
            .field(&self.text)
            .finish()
    }
}

#[derive(Clone, Copy)]
pub(crate) enum AuthoritySealDomain {
    InstallationRootLineage,
    InstalledPackageKey,
    InstalledApplicationQuery,
    InstalledApplicationOperation,
    InstalledApplicationCapability,
    InstalledAbility,
    InstalledPrincipalBinding,
    InstalledConditionalDependency,
    InstalledConditionalApplicationOperation,
    InstalledConditionalNode,
}

impl AuthoritySealDomain {
    pub(crate) const fn label(self) -> &'static [u8] {
        match self {
            Self::InstallationRootLineage => b"worth-query-installation-root-lineage-v1",
            Self::InstalledPackageKey => b"worth-query-installed-package-key-v1",
            Self::InstalledApplicationQuery => {
                b"worth-query-installed-application-query-authority-v2"
            }
            Self::InstalledApplicationOperation => {
                b"worth-query-installed-application-operation-authority-v2"
            }
            Self::InstalledApplicationCapability => {
                b"worth-query-installed-application-capability-authority-v1"
            }
            Self::InstalledAbility => b"worth-query-installed-ability-authority-v3",
            Self::InstalledPrincipalBinding => {
                b"worth-query-installed-principal-binding-authority-v2"
            }
            Self::InstalledConditionalDependency => {
                b"worth-query-installed-conditional-dependency-authority-v1"
            }
            Self::InstalledConditionalApplicationOperation => {
                b"worth-query-installed-conditional-application-operation-authority-v1"
            }
            Self::InstalledConditionalNode => {
                b"worth-query-installed-conditional-node-authority-v1"
            }
        }
    }
}

pub(crate) struct AuthorityTranscript {
    mac: HmacSha256,
}

impl AuthorityTranscript {
    pub(crate) fn new(key: &PackageAuthorityKey, domain: AuthoritySealDomain) -> Self {
        Self::with_key(&key.0, domain)
    }

    fn with_key(key: &[u8; 32], domain: AuthoritySealDomain) -> Self {
        let mut mac =
            HmacSha256::new_from_slice(key).expect("HMAC-SHA-256 accepts a 32-byte authority key");
        write_bytes(&mut mac, b"domain");
        write_bytes(&mut mac, domain.label());
        Self { mac }
    }

    pub(crate) fn text(&mut self, tag: &str, value: &str) {
        self.field_header(tag, b"text");
        write_bytes(&mut self.mac, value.as_bytes());
    }

    pub(crate) fn bytes(&mut self, tag: &str, value: &[u8]) {
        self.field_header(tag, b"bytes");
        write_bytes(&mut self.mac, value);
    }

    pub(crate) fn optional_text(&mut self, tag: &str, value: Option<&str>) {
        self.field_header(tag, b"optional-text");
        match value {
            Some(value) => {
                self.mac.update(&[1]);
                write_bytes(&mut self.mac, value.as_bytes());
            }
            None => self.mac.update(&[0]),
        }
    }

    pub(crate) fn u64(&mut self, tag: &str, value: u64) {
        self.field_header(tag, b"u64");
        self.mac.update(&value.to_le_bytes());
    }

    pub(crate) fn usize(&mut self, tag: &str, value: usize) {
        let value = u64::try_from(value).expect("authority transcript usize fits u64");
        self.u64(tag, value);
    }

    pub(crate) fn finish(self) -> AuthoritySeal {
        AuthoritySeal::from_bytes(self.finish_bytes())
    }

    pub(crate) fn verifies(self, seal: &AuthoritySeal) -> bool {
        self.mac.verify_slice(&seal.bytes).is_ok()
    }

    fn finish_bytes(self) -> [u8; 32] {
        self.mac.finalize().into_bytes().into()
    }

    fn field_header(&mut self, tag: &str, kind: &[u8]) {
        write_bytes(&mut self.mac, tag.as_bytes());
        write_bytes(&mut self.mac, kind);
    }
}

fn write_bytes(mac: &mut HmacSha256, value: &[u8]) {
    let length = u64::try_from(value.len()).expect("authority transcript field length fits u64");
    mac.update(&length.to_le_bytes());
    mac.update(value);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entropy_failure_is_explicit() {
        assert!(InstallationAuthorityRootKey::generate_with(|_| Err(())).is_err());
    }

    #[test]
    fn secret_debug_output_is_redacted() {
        let key = InstallationAuthorityRootKey::generate_with(|bytes| {
            bytes.fill(0x5a);
            Ok(())
        })
        .unwrap();
        let package = key.derive_package_key(1, 1, b"package", b"admission");

        assert_eq!(format!("{key:?}"), REDACTED_KEY);
        assert_eq!(format!("{package:?}"), REDACTED_KEY);
        assert!(!format!("{package:?}").contains("5a"));
    }

    #[test]
    fn hmac_sha256_matches_the_rfc_4231_case_one_vector() {
        let mut mac = HmacSha256::new_from_slice(&[0x0b; 20]).unwrap();
        mac.update(b"Hi There");

        assert_eq!(
            format!("{:x}", mac.finalize().into_bytes()),
            "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7"
        );
    }

    #[test]
    fn authority_transcripts_deny_wrong_key_family_field_and_option_collision() {
        let first_root = InstallationAuthorityRootKey::repeated_byte_for_test(0x11);
        let second_root = InstallationAuthorityRootKey::repeated_byte_for_test(0x22);
        let first_key = first_root.derive_package_key(7, 3, b"package", b"admission");
        let second_key = second_root.derive_package_key(7, 3, b"package", b"admission");

        let mut original =
            AuthorityTranscript::new(&first_key, AuthoritySealDomain::InstalledAbility);
        original.text("ability", "read");
        original.optional_text("policy", None);
        let seal = original.finish();

        let mut exact = AuthorityTranscript::new(&first_key, AuthoritySealDomain::InstalledAbility);
        exact.text("ability", "read");
        exact.optional_text("policy", None);
        assert!(exact.verifies(&seal));

        let mut wrong_key =
            AuthorityTranscript::new(&second_key, AuthoritySealDomain::InstalledAbility);
        wrong_key.text("ability", "read");
        wrong_key.optional_text("policy", None);
        assert!(!wrong_key.verifies(&seal));

        let mut wrong_family =
            AuthorityTranscript::new(&first_key, AuthoritySealDomain::InstalledApplicationQuery);
        wrong_family.text("ability", "read");
        wrong_family.optional_text("policy", None);
        assert!(!wrong_family.verifies(&seal));

        let mut changed_field =
            AuthorityTranscript::new(&first_key, AuthoritySealDomain::InstalledAbility);
        changed_field.text("ability", "write");
        changed_field.optional_text("policy", None);
        assert!(!changed_field.verifies(&seal));

        let mut colliding_sentinel =
            AuthorityTranscript::new(&first_key, AuthoritySealDomain::InstalledAbility);
        colliding_sentinel.text("ability", "read");
        colliding_sentinel.optional_text("policy", Some("unbound-policy"));
        assert!(!colliding_sentinel.verifies(&seal));
    }

    #[test]
    fn package_keys_are_stable_within_lineage_and_rotate_by_generation() {
        let root = InstallationAuthorityRootKey::repeated_byte_for_test(0x33);
        let same = root.derive_package_key(7, 3, b"package", b"admission");
        let rebuilt = root.derive_package_key(7, 3, b"package", b"admission");
        let successor = root.derive_package_key(7, 4, b"package", b"admission");

        assert!(same.matches(&rebuilt));
        assert!(!same.matches(&successor));
    }
}
