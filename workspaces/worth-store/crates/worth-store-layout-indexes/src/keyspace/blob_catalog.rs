/// Canonical key for the Store's derived blob-publication catalog.
///
/// The Store must separately establish that the object identity belongs to
/// its current incarnation. This mechanism only fixes comparison bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BlobCatalogPointKey {
    bytes: [u8; 24],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlobCatalogPointKeyDenial {
    EmptyObject,
    ZeroGeneration,
}

impl BlobCatalogPointKey {
    pub fn admit(object: [u8; 16], generation: u64) -> Result<Self, BlobCatalogPointKeyDenial> {
        if object == [0; 16] {
            return Err(BlobCatalogPointKeyDenial::EmptyObject);
        }
        if generation == 0 {
            return Err(BlobCatalogPointKeyDenial::ZeroGeneration);
        }
        let mut bytes = [0; 24];
        bytes[..16].copy_from_slice(&object);
        bytes[16..].copy_from_slice(&generation.to_be_bytes());
        Ok(Self { bytes })
    }

    pub const fn bytes(self) -> [u8; 24] {
        self.bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_order_is_object_then_unsigned_generation() {
        let object = [1; 16];
        let first = BlobCatalogPointKey::admit(object, 1).unwrap();
        let second = BlobCatalogPointKey::admit(object, 256).unwrap();
        let next_object = BlobCatalogPointKey::admit([2; 16], 1).unwrap();
        assert!(first < second && second < next_object);
        assert_eq!(first.bytes()[16..], 1_u64.to_be_bytes());
        assert_eq!(
            BlobCatalogPointKey::admit([0; 16], 1),
            Err(BlobCatalogPointKeyDenial::EmptyObject)
        );
        assert_eq!(
            BlobCatalogPointKey::admit(object, 0),
            Err(BlobCatalogPointKeyDenial::ZeroGeneration)
        );
    }
}
