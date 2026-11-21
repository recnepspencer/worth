use super::{WorthQueryEvidenceIdentityScheme, WorthQueryEvidenceScope};

/// Opaque, comparable key for operational identity correlation.
///
/// The key preserves scope and scheme while withholding the terminal text
/// projection. Consumers may index or compare it, but cannot reinterpret it as
/// authored identity or reporting data.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryEvidenceIdentityKey {
    scope: WorthQueryEvidenceScope,
    scheme: WorthQueryEvidenceIdentityScheme,
    digest: [u8; 32],
}

impl WorthQueryEvidenceIdentityKey {
    pub(super) const fn new(
        scope: WorthQueryEvidenceScope,
        scheme: WorthQueryEvidenceIdentityScheme,
        digest: [u8; 32],
    ) -> Self {
        Self {
            scope,
            scheme,
            digest,
        }
    }

    pub const fn scope(self) -> WorthQueryEvidenceScope {
        self.scope
    }

    pub const fn scheme(self) -> WorthQueryEvidenceIdentityScheme {
        self.scheme
    }

    /// Opaque correlation material for typed downstream host contracts.
    pub const fn correlation_digest(self) -> [u8; 32] {
        self.digest
    }
}

impl std::fmt::Debug for WorthQueryEvidenceIdentityKey {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryEvidenceIdentityKey")
            .field("scope", &self.scope)
            .field("scheme", &self.scheme)
            .field("digest", &"sealed")
            .finish()
    }
}
