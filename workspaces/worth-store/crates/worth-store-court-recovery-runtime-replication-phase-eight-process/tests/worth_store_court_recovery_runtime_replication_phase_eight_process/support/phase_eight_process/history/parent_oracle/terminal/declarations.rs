pub(crate) use {sha2::Digest, sha2::Sha256};
pub(crate) use {worth_store_physical_format::PHYSICAL_MUTATION_BINDING_COMPACTION_RECORD_DOMAIN};

pub(crate) const CHECKPOINT_MAGIC: &[u8] = b"WCP7REC\0";
pub(crate) const TERMINAL_STATE: u8 = 3;
pub(crate) const PROVEN_NO_EFFECT_CLASS: u8 = 1;
pub(crate) const KEY_DOMAIN: &[u8] = b"store.physical.mutation.idempotency-key.v1";

/// Returns a no-effect conclusion only when a verified checkpoint contains a
/// canonical-shaped terminal binding with an explicit persisted no-effect
/// fate. Identity bytes in a WAL, receipt, or arbitrary checkpoint payload are
/// not terminal evidence.
pub(crate) fn contains_persisted_no_effect_terminal(
    files: &[(String, Vec<u8>)],
    identity: &[u8],
) -> Result<bool, String> {
    for (path, bytes) in files {
        if !bytes.starts_with(CHECKPOINT_MAGIC) {
            continue;
        }
        let records = crate::phase_eight_process::history::parent_oracle::wire::validated_checkpoint_binding_payloads(bytes)
            .ok_or_else(|| format!("independent terminal oracle rejected checkpoint: {path}"))?;
        for record in records {
            if is_proven_no_effect_terminal(record, identity) {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

pub(crate) fn is_proven_no_effect_terminal(bytes: &[u8], identity: &[u8]) -> bool {
    let mut cursor = crate::phase_eight_process::history::parent_oracle::terminal::FieldCursor::new(bytes);
    if cursor.field() != Some(PHYSICAL_MUTATION_BINDING_COMPACTION_RECORD_DOMAIN)
        || cursor.byte() != Some(TERMINAL_STATE)
    {
        return false;
    }
    if cursor.field() != Some(identity) {
        return false;
    }
    let Some(store) = cursor.array_field(16) else {
        return false;
    };
    let Some(policy) = cursor.array_field(32) else {
        return false;
    };
    let Some(issuance) = cursor.u64() else {
        return false;
    };
    let Some(expiry) = cursor.u64() else {
        return false;
    };
    let Some(material) = cursor.array_field(32) else {
        return false;
    };
    let Some(fingerprint) = cursor.array_field(32) else {
        return false;
    };
    let Some(mutation_store) = cursor.array_field(16) else {
        return false;
    };
    let Some(runtime) = cursor.u64() else {
        return false;
    };
    let Some(lifecycle) = cursor.u64() else {
        return false;
    };
    let Some(operation) = cursor.u64() else {
        return false;
    };
    let mut key = sha2::Sha256::new();
    key.update((KEY_DOMAIN.len() as u64).to_le_bytes());
    key.update(KEY_DOMAIN);
    key.update(store);
    key.update(policy);
    key.update(issuance.to_le_bytes());
    key.update(expiry.to_le_bytes());
    key.update(material);
    let key_identity: [u8; 32] = key.finalize().into();
    if key_identity.as_slice() != identity
        || store == [0; 16]
        || policy == [0; 32]
        || material == [0; 32]
        || fingerprint == [0; 32]
        || mutation_store != store
        || issuance >= expiry
        || runtime == 0
        || lifecycle == 0
        || operation == 0
    {
        return false;
    }
    cursor.byte() == Some(PROVEN_NO_EFFECT_CLASS)
        && matches!(cursor.byte(), Some(1..=4))
        && cursor.is_empty()
}

pub(crate) struct FieldCursor<'bytes> {
    pub(crate) remaining: &'bytes [u8],
}

impl<'bytes> FieldCursor<'bytes> {
    pub(crate) const fn new(bytes: &'bytes [u8]) -> Self {
        Self { remaining: bytes }
    }

    pub(crate) fn field(&mut self) -> Option<&'bytes [u8]> {
        let length = usize::try_from(self.u64()?).ok()?;
        let (field, remaining) = self.remaining.split_at_checked(length)?;
        self.remaining = remaining;
        Some(field)
    }

    pub(crate) fn array_field(&mut self, expected: usize) -> Option<&'bytes [u8]> {
        let field = self.field()?;
        (field.len() == expected).then_some(field)
    }

    pub(crate) fn u64(&mut self) -> Option<u64> {
        let (value, remaining) = self.remaining.split_at_checked(8)?;
        self.remaining = remaining;
        Some(u64::from_le_bytes(value.try_into().ok()?))
    }

    pub(crate) fn byte(&mut self) -> Option<u8> {
        let (value, remaining) = self.remaining.split_at_checked(1)?;
        self.remaining = remaining;
        value.first().copied()
    }

    pub(crate) const fn is_empty(&self) -> bool {
        self.remaining.is_empty()
    }
}