use super::{nonzero, u64_at, BlobFact};

/// Independent parsing of abandonment v1; no runtime or format decoder.
pub(super) fn decode(payload: &[u8], frame_digest: [u8; 32]) -> Option<BlobFact> {
    let expiry_checkpoint = match (payload.len(), payload.get(88)) {
        (89, Some(1)) => None,
        (97, Some(2)) if u64_at(payload, 89) != 0 => Some(u64_at(payload, 89)),
        _ => return None,
    };
    let record: [u8; 24] = payload[32..56].try_into().ok()?;
    let epoch: [u8; 16] = record[..16].try_into().ok()?;
    let digest = payload[56..88].try_into().ok()?;
    if !nonzero(&epoch) || u64_at(&record, 16) == 0 || !nonzero(&digest) {
        return None;
    }
    Some(BlobFact::Abandoned {
        store: payload[..16].try_into().ok()?,
        frame_digest,
        session: payload[16..32].try_into().ok()?,
        declaration_record: record,
        declaration_digest: digest,
        expiry_checkpoint,
    })
}

#[cfg(test)]
mod tests {
    use super::super::{decode, BlobFact, OfflineIntegrityObservationCounters, Outcome, Sha256};

    fn frame() -> Vec<u8> {
        let mut bytes = vec![0; 137];
        bytes[..8].copy_from_slice(b"WRC11BLB");
        bytes[8..10].copy_from_slice(&[6, 1]);
        bytes[12..16].copy_from_slice(&89_u32.to_le_bytes());
        bytes[48..64].fill(1);
        bytes[64..80].fill(2);
        bytes[80..96].fill(3);
        bytes[96..104].copy_from_slice(&7_u64.to_le_bytes());
        bytes[104..136].fill(4);
        bytes[136] = 1;
        reseal(&mut bytes);
        bytes
    }

    fn reseal(bytes: &mut [u8]) {
        let mut sha = Sha256::new();
        sha.update(&bytes[..16]);
        sha.update(&bytes[48..]);
        bytes[16..48].copy_from_slice(&sha.finish());
    }

    #[test]
    fn independent_abort_decoder_requires_exact_binding_and_known_cause() {
        let bytes = frame();
        let mut counters = OfflineIntegrityObservationCounters::default();
        assert!(matches!(decode(&bytes, Some([1; 16]), &mut counters),
            Ok(BlobFact::Abandoned { declaration_digest, .. })
                if declaration_digest == [4; 32]));
        for range in [80..96, 96..104, 104..136, 136..137] {
            let mut invalid = bytes.clone();
            invalid[range].fill(0);
            reseal(&mut invalid);
            assert!(matches!(
                decode(&invalid, Some([1; 16]), &mut counters),
                Err(Outcome::Damaged(_))
            ));
        }
        let mut unsupported_cause = bytes;
        unsupported_cause[136] = 2;
        reseal(&mut unsupported_cause);
        assert!(matches!(
            decode(&unsupported_cause, Some([1; 16]), &mut counters),
            Err(Outcome::Damaged(_))
        ));
    }

    #[test]
    fn independent_expiry_decoder_requires_reason_specific_nonzero_witness() {
        let mut bytes = frame();
        bytes.resize(145, 0);
        bytes[12..16].copy_from_slice(&97_u32.to_le_bytes());
        bytes[136] = 2;
        bytes[137..145].copy_from_slice(&6_u64.to_le_bytes());
        reseal(&mut bytes);
        let mut counters = OfflineIntegrityObservationCounters::default();
        assert!(matches!(
            decode(&bytes, Some([1; 16]), &mut counters),
            Ok(BlobFact::Abandoned {
                expiry_checkpoint: Some(6),
                ..
            })
        ));
        for invalid_reason in [0, 1, 3] {
            let mut invalid = bytes.clone();
            invalid[136] = invalid_reason;
            reseal(&mut invalid);
            assert!(matches!(
                decode(&invalid, Some([1; 16]), &mut counters),
                Err(Outcome::Damaged(_))
            ));
        }
        bytes[137..145].fill(0);
        reseal(&mut bytes);
        assert!(matches!(
            decode(&bytes, Some([1; 16]), &mut counters),
            Err(Outcome::Damaged(_))
        ));
    }
}
