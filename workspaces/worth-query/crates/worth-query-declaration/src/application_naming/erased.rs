use std::fmt;

/// Kind-erased, fixed 32-byte copy of an admitted [`super::WorthQueryName`].
///
/// It exists only so one [`super::NameSelection`] can name carriers of
/// several kinds. The only constructor is [`super::WorthQueryName::erased`],
/// so an erased name is always a copy of a name WORTH admitted; it grants
/// nothing and resolves nothing by itself.
///
/// Raw bytes cannot become an erased name:
///
/// ```compile_fail,E0599
/// use worth_query_declaration::facade::application_naming::ErasedWorthQueryName;
///
/// let _forged = ErasedWorthQueryName::from_bytes([0u8; 32]);
/// ```
///
/// A struct literal cannot forge one either (the field is private):
///
/// ```compile_fail,E0451
/// use worth_query_declaration::facade::application_naming::ErasedWorthQueryName;
///
/// let _forged = ErasedWorthQueryName { bytes: [0u8; 32] };
/// ```
///
/// Untrusted wire data cannot deserialize into one:
///
/// ```compile_fail,E0277
/// use worth_query_declaration::facade::application_naming::ErasedWorthQueryName;
///
/// fn require_deserialize<T: for<'de> serde::Deserialize<'de>>() {}
/// require_deserialize::<ErasedWorthQueryName>();
/// ```
///
/// Erasing an admitted name keeps its exact canonical digest:
///
/// ```
/// use worth_query_declaration::facade::application_naming::{
///     ErasedWorthQueryName, WorthQueryName,
/// };
///
/// struct Face;
///
/// fn erase(face: WorthQueryName<Face>) -> ErasedWorthQueryName {
///     let erased = face.erased();
///     assert_eq!(erased.as_bytes(), face.as_bytes());
///     erased
/// }
///
/// let _check: fn(WorthQueryName<Face>) -> ErasedWorthQueryName = erase;
/// assert_eq!(std::mem::size_of::<ErasedWorthQueryName>(), 32);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ErasedWorthQueryName {
    bytes: [u8; 32],
}

impl ErasedWorthQueryName {
    /// Canonical byte length, identical to [`super::WorthQueryName::BYTE_LEN`].
    pub const BYTE_LEN: usize = 32;

    pub(super) const fn from_admitted_name_bytes(bytes: [u8; 32]) -> Self {
        Self { bytes }
    }

    /// Read-only view of the fixed 32-byte canonical digest.
    #[inline]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.bytes
    }
}

impl fmt::Debug for ErasedWorthQueryName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut digest = String::with_capacity(Self::BYTE_LEN * 2);
        for byte in self.bytes {
            digest.push(HEX[(byte >> 4) as usize] as char);
            digest.push(HEX[(byte & 0x0f) as usize] as char);
        }
        formatter
            .debug_tuple("ErasedWorthQueryName")
            .field(&digest)
            .finish()
    }
}
