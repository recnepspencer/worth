use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

/// Fixed-size, opaque, kind-typed persistent name minted by WORTH Relational.
///
/// A `WorthQueryName<K>` is never a `RecordId` and carries no allocation,
/// partition, slot, generation, or commit-position coordinates. Its layout is
/// strictly 32 bytes regardless of split or lineage depth.
///
/// # Compile-time authority enforcement
///
/// Callers and handlers cannot forge a `WorthQueryName<K>` from raw bytes or
/// arbitrary strings, cannot substitute a name of one kind for another, and
/// cannot mutate a held name in place.
///
/// Forging from raw bytes is refused at compile time:
///
/// ```compile_fail
/// use worth_query_declaration::facade::application_naming::WorthQueryName;
///
/// struct Face;
///
/// let _forged: WorthQueryName<Face> = WorthQueryName::from_bytes([0u8; 32]);
/// ```
///
/// Forging via struct literal is refused at compile time (private fields):
///
/// ```compile_fail
/// use worth_query_declaration::facade::application_naming::WorthQueryName;
///
/// struct Face;
///
/// let _forged = WorthQueryName::<Face> { bytes: [0u8; 32], _kind: std::marker::PhantomData };
/// ```
///
/// Forging from an arbitrary string is refused at compile time:
///
/// ```compile_fail
/// use worth_query_declaration::facade::application_naming::WorthQueryName;
///
/// struct Face;
///
/// let _forged: WorthQueryName<Face> = "worth.face.01".parse().unwrap();
/// ```
///
/// Cross-kind substitution (`WorthQueryName<Edge>` passed where
/// `WorthQueryName<Face>` is expected) is refused at compile time:
///
/// ```compile_fail
/// use worth_query_declaration::facade::application_naming::WorthQueryName;
///
/// struct Face;
/// struct Edge;
///
/// fn require_face(_face: WorthQueryName<Face>) {}
///
/// fn misuse(edge: WorthQueryName<Edge>) {
///     require_face(edge);
/// }
/// ```
///
/// A handler cannot overwrite or mutate a held name's bytes:
///
/// ```compile_fail
/// use worth_query_declaration::facade::application_naming::WorthQueryName;
///
/// struct Face;
///
/// fn handler_write_attempt(mut face: WorthQueryName<Face>) {
///     face.as_bytes_mut()[0] = 0xff;
/// }
/// ```
///
/// Valid positive counterpart — accepting an admitted name of the matching kind
/// and observing its fixed 32-byte canonical digest compiles cleanly:
///
/// ```
/// use worth_query_declaration::facade::application_naming::WorthQueryName;
///
/// struct Face;
///
/// fn observe_face(face: WorthQueryName<Face>) -> [u8; 32] {
///     assert_eq!(std::mem::size_of::<WorthQueryName<Face>>(), WorthQueryName::<Face>::BYTE_LEN);
///     *face.as_bytes()
/// }
///
/// let _check: fn(WorthQueryName<Face>) -> [u8; 32] = observe_face;
/// ```
pub struct WorthQueryName<K> {
    bytes: [u8; 32],
    _kind: PhantomData<fn() -> K>,
}

impl<K> WorthQueryName<K> {
    /// Canonical byte length of every persistent name identity.
    pub const BYTE_LEN: usize = 32;

    /// Internal-only admission helper for crate-local verification.
    ///
    /// External crates have no constructor from raw bytes; runtime admission
    /// flows through Relational's proof-bearing birth and readmission doors.
    #[cfg(test)]
    pub(crate) const fn from_admitted_digest(bytes: [u8; 32]) -> Self {
        Self {
            bytes,
            _kind: PhantomData,
        }
    }

    /// Read-only view of the fixed 32-byte canonical digest.
    #[inline]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.bytes
    }

    /// Lowercase hexadecimal representation for diagnostics and logs.
    pub fn render_hex(&self) -> String {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut out = String::with_capacity(Self::BYTE_LEN * 2);
        for byte in self.bytes {
            out.push(HEX[(byte >> 4) as usize] as char);
            out.push(HEX[(byte & 0x0f) as usize] as char);
        }
        out
    }
}

impl<K> Copy for WorthQueryName<K> {}

impl<K> Clone for WorthQueryName<K> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<K> PartialEq for WorthQueryName<K> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.bytes == other.bytes
    }
}

impl<K> Eq for WorthQueryName<K> {}

impl<K> PartialOrd for WorthQueryName<K> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<K> Ord for WorthQueryName<K> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.bytes.cmp(&other.bytes)
    }
}

impl<K> Hash for WorthQueryName<K> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.bytes.hash(state);
    }
}

impl<K> fmt::Debug for WorthQueryName<K> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WorthQueryName")
            .field("kind", &std::any::type_name::<K>())
            .field("digest", &self.render_hex())
            .finish()
    }
}
