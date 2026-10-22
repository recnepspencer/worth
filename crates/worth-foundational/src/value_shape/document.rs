//! Domain-owned pairing of an admitted document with its original canonical text.
use super::{CanonicalValue, ValueCodecDenial, ValueDecodeAdmission, ValueDecodeBounds};

/// Implement on the actual envelope grammar. Only its domain owner constructs
/// `Document`; adapters borrow its original encoding and delegate both readers.
pub trait CanonicalDocumentCodec: CanonicalValue {
    type Document: 'static;
    const BOUNDS: ValueDecodeBounds;
    fn encoded(document: &Self::Document) -> &str;
    fn read(raw: &str) -> Result<Self::Document, ValueCodecDenial>;
    fn read_admitted(
        raw: &str,
        admission: &mut ValueDecodeAdmission<'_>,
    ) -> Result<Self::Document, ValueCodecDenial>;
}
