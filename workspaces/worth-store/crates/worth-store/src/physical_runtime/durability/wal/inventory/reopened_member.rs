use sha2::{Digest, Sha256};
use worth_store_wal::{LogSequenceNumber, VerifiedWalFramePayload, WalLsnRange};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) enum PhysicalWalBindingReopenCutoff {
    GenerationZero,
    NamespaceDurableCheckpoint(LogSequenceNumber),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::physical_runtime) struct ReopenedPhysicalWalMember {
    lsn_range: WalLsnRange,
    persisted_binding: Box<[u8]>,
    redo_digest: [u8; 32],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime::durability::wal) enum PhysicalWalMemberPayloadDenial {
    CutoffStraddlesFrame,
    TruncatedFieldLength,
    EmptyPersistedBinding,
    EmptyCanonicalRedo,
    FieldLengthOverflow,
    TruncatedField,
    TrailingBytes,
}

enum ReopenedFrameDisposition {
    BeforeCheckpoint,
    RetainedTail,
}

impl PhysicalWalBindingReopenCutoff {
    pub(in crate::physical_runtime) const fn after_checkpoint(cutoff_lsn_exclusive: u64) -> Self {
        Self::NamespaceDurableCheckpoint(LogSequenceNumber::new(cutoff_lsn_exclusive))
    }

    pub(super) const fn lsn(self) -> Option<LogSequenceNumber> {
        match self {
            Self::GenerationZero => None,
            Self::NamespaceDurableCheckpoint(lsn) => Some(lsn),
        }
    }

    fn classify(
        self,
        range: WalLsnRange,
    ) -> Result<ReopenedFrameDisposition, PhysicalWalMemberPayloadDenial> {
        let Some(cutoff) = self.lsn() else {
            return Ok(ReopenedFrameDisposition::RetainedTail);
        };
        if range.end_exclusive() <= cutoff {
            return Ok(ReopenedFrameDisposition::BeforeCheckpoint);
        }
        if range.start() >= cutoff {
            return Ok(ReopenedFrameDisposition::RetainedTail);
        }
        Err(PhysicalWalMemberPayloadDenial::CutoffStraddlesFrame)
    }
}

impl ReopenedPhysicalWalMember {
    pub(super) fn decode_retained_frame(
        cutoff: PhysicalWalBindingReopenCutoff,
        frame: VerifiedWalFramePayload<'_>,
    ) -> Result<Option<Self>, PhysicalWalMemberPayloadDenial> {
        if matches!(
            cutoff.classify(frame.lsn_range())?,
            ReopenedFrameDisposition::BeforeCheckpoint
        ) {
            return Ok(None);
        }
        let mut payload = frame.payload();
        let persisted_binding = take_field(&mut payload)?;
        if persisted_binding.is_empty() {
            return Err(PhysicalWalMemberPayloadDenial::EmptyPersistedBinding);
        }
        let canonical_redo = take_field(&mut payload)?;
        if canonical_redo.is_empty() {
            return Err(PhysicalWalMemberPayloadDenial::EmptyCanonicalRedo);
        }
        if !payload.is_empty() {
            return Err(PhysicalWalMemberPayloadDenial::TrailingBytes);
        }
        Ok(Some(Self {
            lsn_range: frame.lsn_range(),
            persisted_binding: persisted_binding.into(),
            redo_digest: Sha256::digest(canonical_redo).into(),
        }))
    }

    /// Unit courts supply the verified frame facts WAL inventory would decode.
    #[cfg(test)]
    pub(in crate::physical_runtime::durability) fn from_verified_parts(
        lsn_range: WalLsnRange,
        persisted_binding: &[u8],
        redo_digest: [u8; 32],
    ) -> Self {
        Self {
            lsn_range,
            persisted_binding: persisted_binding.into(),
            redo_digest,
        }
    }

    pub(in crate::physical_runtime::durability) const fn lsn_range(&self) -> WalLsnRange {
        self.lsn_range
    }

    pub(in crate::physical_runtime::durability) fn persisted_binding(&self) -> &[u8] {
        &self.persisted_binding
    }

    pub(in crate::physical_runtime::durability) const fn redo_digest(&self) -> [u8; 32] {
        self.redo_digest
    }
}

fn take_field<'payload>(
    payload: &mut &'payload [u8],
) -> Result<&'payload [u8], PhysicalWalMemberPayloadDenial> {
    let length_bytes = payload
        .get(..8)
        .ok_or(PhysicalWalMemberPayloadDenial::TruncatedFieldLength)?;
    let length = usize::try_from(u64::from_le_bytes(length_bytes.try_into().unwrap()))
        .map_err(|_| PhysicalWalMemberPayloadDenial::FieldLengthOverflow)?;
    let end = 8_usize
        .checked_add(length)
        .ok_or(PhysicalWalMemberPayloadDenial::FieldLengthOverflow)?;
    let field = payload
        .get(8..end)
        .ok_or(PhysicalWalMemberPayloadDenial::TruncatedField)?;
    *payload = payload
        .get(end..)
        .ok_or(PhysicalWalMemberPayloadDenial::TruncatedField)?;
    Ok(field)
}
