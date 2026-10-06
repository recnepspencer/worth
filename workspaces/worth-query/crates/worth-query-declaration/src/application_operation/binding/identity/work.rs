//! Deterministic work of one canonical identity derivation.

/// What deriving canonical identities cost, counted while each encoding
/// streamed into its hasher and never measured by time.
///
/// The counts are exact: the same value under the same domain and scope always
/// reports the same numbers, and the work of several derivations adds up.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ApplicationCanonicalWork {
    derivations: u32,
    encoded_bytes: usize,
    sha256_input_bytes: usize,
    sha256_compression_blocks: usize,
    buffered_bytes: usize,
}

impl ApplicationCanonicalWork {
    /// No derivation happened.
    pub const fn none() -> Self {
        Self {
            derivations: 0,
            encoded_bytes: 0,
            sha256_input_bytes: 0,
            sha256_compression_blocks: 0,
            buffered_bytes: 0,
        }
    }

    pub(super) const fn one_derivation(
        encoded_bytes: usize,
        sha256_input_bytes: usize,
        buffered_bytes: usize,
    ) -> Self {
        Self {
            derivations: 1,
            encoded_bytes,
            sha256_input_bytes,
            sha256_compression_blocks: sha256_input_bytes.saturating_add(9 + 63) / 64,
            buffered_bytes,
        }
    }

    /// The work of both, as when one request derives its key and its input identity.
    #[must_use]
    pub const fn combine(self, other: Self) -> Self {
        Self {
            derivations: self.derivations.saturating_add(other.derivations),
            encoded_bytes: self.encoded_bytes.saturating_add(other.encoded_bytes),
            sha256_input_bytes: self
                .sha256_input_bytes
                .saturating_add(other.sha256_input_bytes),
            sha256_compression_blocks: self
                .sha256_compression_blocks
                .saturating_add(other.sha256_compression_blocks),
            buffered_bytes: self.buffered_bytes.saturating_add(other.buffered_bytes),
        }
    }

    /// How many identities were derived.
    pub const fn derivations(self) -> u32 {
        self.derivations
    }

    /// Bytes of canonical value encoding, without the domain and scope framing.
    pub const fn encoded_bytes(self) -> usize {
        self.encoded_bytes
    }

    /// Bytes fed to SHA-256: the framing plus the encoded value.
    pub const fn sha256_input_bytes(self) -> usize {
        self.sha256_input_bytes
    }

    /// SHA-256 compression blocks those bytes fill, padding included.
    pub const fn sha256_compression_blocks(self) -> usize {
        self.sha256_compression_blocks
    }

    /// Map entry bytes held in memory so the entries could be sorted.
    pub const fn buffered_bytes(self) -> usize {
        self.buffered_bytes
    }
}
