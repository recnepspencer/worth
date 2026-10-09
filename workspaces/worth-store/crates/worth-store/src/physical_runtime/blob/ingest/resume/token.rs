use worth_store_physical_format::PersistedRecordIdentity;

const TOKEN_BYTES: usize = 116;
const MAGIC: &[u8; 8] = b"WBLRSM01";

/// Portable, untrusted reference to a durable declaration. Decoding does not
/// admit a resume: selected records, scope, expiry and custody must agree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlobResumeToken {
    pub(in crate::physical_runtime::blob) store: [u8; 16],
    pub(in crate::physical_runtime::blob) session: [u8; 16],
    pub(in crate::physical_runtime::blob) declaration_record: PersistedRecordIdentity,
    pub(in crate::physical_runtime::blob) declaration_digest: [u8; 32],
    pub(in crate::physical_runtime::blob) chunk_size: u32,
    pub(in crate::physical_runtime::blob) total_bytes: u64,
    pub(in crate::physical_runtime::blob) max_checkpoint_sequence: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlobResumeTokenDenial {
    Length,
    Version,
    InvalidReference,
}

impl BlobResumeToken {
    pub fn encode(self) -> [u8; TOKEN_BYTES] {
        let mut bytes = [0; TOKEN_BYTES];
        bytes[..8].copy_from_slice(MAGIC);
        bytes[8..24].copy_from_slice(&self.store);
        bytes[24..40].copy_from_slice(&self.session);
        bytes[40..56].copy_from_slice(&self.declaration_record.allocation_epoch());
        bytes[56..64].copy_from_slice(&self.declaration_record.ordinal().to_le_bytes());
        bytes[64..96].copy_from_slice(&self.declaration_digest);
        bytes[96..100].copy_from_slice(&self.chunk_size.to_le_bytes());
        bytes[100..108].copy_from_slice(&self.total_bytes.to_le_bytes());
        bytes[108..116].copy_from_slice(&self.max_checkpoint_sequence.to_le_bytes());
        bytes
    }

    /// Parses transport syntax only; even a well-formed token can be forged.
    pub fn decode(bytes: &[u8]) -> Result<Self, BlobResumeTokenDenial> {
        if bytes.len() != TOKEN_BYTES {
            return Err(BlobResumeTokenDenial::Length);
        }
        if &bytes[..8] != MAGIC {
            return Err(BlobResumeTokenDenial::Version);
        }
        let token = Self {
            store: bytes[8..24].try_into().expect("fixed field"),
            session: bytes[24..40].try_into().expect("fixed field"),
            declaration_record: PersistedRecordIdentity::new(
                bytes[40..56].try_into().expect("fixed field"),
                u64::from_le_bytes(bytes[56..64].try_into().expect("fixed field")),
            )
            .ok_or(BlobResumeTokenDenial::InvalidReference)?,
            declaration_digest: bytes[64..96].try_into().expect("fixed field"),
            chunk_size: u32::from_le_bytes(bytes[96..100].try_into().expect("fixed field")),
            total_bytes: u64::from_le_bytes(bytes[100..108].try_into().expect("fixed field")),
            max_checkpoint_sequence: u64::from_le_bytes(
                bytes[108..116].try_into().expect("fixed field"),
            ),
        };
        if token.store == [0; 16]
            || token.session == [0; 16]
            || token.declaration_digest == [0; 32]
            || token.total_bytes == 0
            || !(65536..=262144).contains(&token.chunk_size)
            || token.max_checkpoint_sequence == 0
        {
            return Err(BlobResumeTokenDenial::InvalidReference);
        }
        Ok(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transport_rejects_malformed_fields_without_claiming_authenticity() {
        let token = BlobResumeToken {
            store: [1; 16],
            session: [2; 16],
            declaration_record: PersistedRecordIdentity::new([3; 16], 1).unwrap(),
            declaration_digest: [4; 32],
            chunk_size: 65536,
            total_bytes: 131072,
            max_checkpoint_sequence: 17,
        };
        let wire = token.encode();
        assert_eq!(BlobResumeToken::decode(&wire), Ok(token));
        for length in 0..TOKEN_BYTES {
            assert_eq!(
                BlobResumeToken::decode(&wire[..length]),
                Err(BlobResumeTokenDenial::Length)
            );
        }
        let mut invalid = wire;
        invalid[7] = b'2';
        assert_eq!(
            BlobResumeToken::decode(&invalid),
            Err(BlobResumeTokenDenial::Version)
        );
        for range in [8..24, 24..40, 40..56, 64..96, 96..100, 100..108, 108..116] {
            invalid = wire;
            invalid[range].fill(0);
            assert_eq!(
                BlobResumeToken::decode(&invalid),
                Err(BlobResumeTokenDenial::InvalidReference)
            );
        }
        // A structurally valid altered reference is deliberately still just
        // syntax. Only selected-state readmission may reject its authenticity.
        invalid = wire;
        invalid[64] ^= 1;
        assert!(BlobResumeToken::decode(&invalid).is_ok());
    }
}
