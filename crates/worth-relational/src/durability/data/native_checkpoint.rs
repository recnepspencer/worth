use std::sync::Arc;
use worth_execution::ExecutionImmutableBytes;

#[cfg(test)]
mod tests;

/// Opaque native bytes for one complete Relational recovery checkpoint.
///
/// Construction accepts untrusted bytes deliberately: admission belongs to
/// the recovery authority, where codec integrity and runtime continuity are
/// checked together. Callers can retain or transport the bytes, but cannot
/// inspect or selectively rewrite Relational truth through this type.
/// Clones share immutable byte backing and retain the same selected region;
/// they do not clone or confer recovery authority.
pub struct RelationalNativeCheckpoint {
    bytes: ExecutionImmutableBytes,
    region: std::ops::Range<usize>,
    captured_sections: Option<NativeCheckpointSectionBytes>,
}

/// Actual MessagePack value bytes emitted during one native capture. The
/// remainder contains map keys, framing, and small checkpoint metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeCheckpointSectionBytes {
    pub total: usize,
    pub envelopes: usize,
    pub branch_roots: usize,
    pub branch_cells: usize,
    pub partition_mirror: usize,
    pub derived_indexes: usize,
    pub framing_and_metadata: usize,
}

impl Clone for RelationalNativeCheckpoint {
    fn clone(&self) -> Self {
        Self {
            bytes: self.bytes.clone(),
            region: self.region.clone(),
            captured_sections: self.captured_sections,
        }
    }
}

impl std::fmt::Debug for RelationalNativeCheckpoint {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RelationalNativeCheckpoint")
            .field("byte_len", &self.bytes().len())
            .finish_non_exhaustive()
    }
}

impl PartialEq for RelationalNativeCheckpoint {
    fn eq(&self, other: &Self) -> bool {
        self.bytes() == other.bytes()
    }
}

impl Eq for RelationalNativeCheckpoint {}

impl RelationalNativeCheckpoint {
    pub fn from_untrusted_bytes(bytes: impl Into<Box<[u8]>>) -> Self {
        let bytes = bytes.into();
        let region = 0..bytes.len();
        Self {
            bytes: ExecutionImmutableBytes::from_external_bytes(Arc::new(bytes)),
            region,
            captured_sections: None,
        }
    }

    /// Retain an embedded native payload without copying its enclosing buffer.
    /// The caller shares immutable backing; the selected range stays private.
    /// Recovery still authenticates and readmits the selected native bytes.
    pub fn from_untrusted_bytes_region(
        bytes: ExecutionImmutableBytes,
        region: std::ops::Range<usize>,
    ) -> Result<Self, &'static str> {
        if bytes.get(region.clone()).is_none() {
            return Err("native checkpoint byte region is out of bounds");
        }
        Ok(Self {
            bytes,
            region,
            captured_sections: None,
        })
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes[self.region.clone()]
    }

    /// Available only on a checkpoint captured by this runtime, not bytes
    /// received from a caller or an enclosing checkpoint container.
    pub fn captured_sections(&self) -> Option<NativeCheckpointSectionBytes> {
        self.captured_sections
    }

    pub(crate) fn from_captured_bytes(
        bytes: Vec<u8>,
        sections: NativeCheckpointSectionBytes,
    ) -> Self {
        let region = 0..bytes.len();
        Self {
            bytes: ExecutionImmutableBytes::from_external_bytes(Arc::new(bytes.into_boxed_slice())),
            region,
            captured_sections: Some(sections),
        }
    }
}
