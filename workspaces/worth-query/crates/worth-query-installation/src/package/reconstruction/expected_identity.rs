//! Caller-supplied expected identity, separate from untrusted record claims.

use crate::package::WorthQueryPortableDomainPackageIdentity;

/// Descriptive identity the caller expects fresh reconstruction to produce.
///
/// This value carries no package validation, installation, host trust, or
/// activation authority. It exists to keep the caller expectation distinct
/// from the identity claimed by the decoded manifest.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryExpectedPortablePackageIdentity {
    identity: WorthQueryPortableDomainPackageIdentity,
}

impl WorthQueryExpectedPortablePackageIdentity {
    pub const fn new(identity: WorthQueryPortableDomainPackageIdentity) -> Self {
        Self { identity }
    }

    pub const fn identity(&self) -> &WorthQueryPortableDomainPackageIdentity {
        &self.identity
    }
}
