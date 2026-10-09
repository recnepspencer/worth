use super::{canonical_bytes_for_key, AdmittedConcretePhysicalKey, AdmittedPhysicalKeyDomain};

const MAX_CANONICAL_REQUEST_KEY_BYTES: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdmittedPhysicalAccessIdentity {
    key_domain: AdmittedPhysicalKeyDomain,
    byte_len: u8,
    canonical_key: [u8; MAX_CANONICAL_REQUEST_KEY_BYTES],
}

impl AdmittedPhysicalAccessIdentity {
    /// Derives comparison bytes only from a key admitted for its family domain.
    /// Callers cannot supply unchecked bytes or a comparator with this entry.
    pub fn admit(key: AdmittedConcretePhysicalKey) -> Self {
        let key_domain = key.domain();
        let canonical = canonical_bytes_for_key(key_domain.comparator(), key.into_raw())
            .expect("an admitted physical key must encode in its admitted key domain");
        assert!(
            canonical.as_bytes().len() <= MAX_CANONICAL_REQUEST_KEY_BYTES,
            "canonical physical request identity exceeds its fixed-shape bound"
        );
        let mut canonical_key = [0; MAX_CANONICAL_REQUEST_KEY_BYTES];
        canonical_key[..canonical.as_bytes().len()].copy_from_slice(canonical.as_bytes());
        Self {
            key_domain,
            byte_len: canonical.as_bytes().len() as u8,
            canonical_key,
        }
    }

    pub const fn key_domain(self) -> AdmittedPhysicalKeyDomain {
        self.key_domain
    }

    pub fn canonical_key(&self) -> &[u8] {
        &self.canonical_key[..self.byte_len as usize]
    }
}
