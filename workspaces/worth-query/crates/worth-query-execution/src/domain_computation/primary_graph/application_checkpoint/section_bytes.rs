/// Bytes written by Query's checkpoint encoder for one captured artifact.
///
/// These are diagnostic sizes, not readmission authority. The native payload
/// remains opaque to Query; its owner may also report nested encoder sections.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationCheckpointSectionBytes {
    framing: usize,
    native: usize,
    accepted_outputs: usize,
    accepted_output_count: usize,
    native_sections: Option<WorthQueryNativeCheckpointSectionBytes>,
}

/// Relational-owned native encoder sections, copied as diagnostic values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryNativeCheckpointSectionBytes {
    pub total: usize,
    pub envelopes: usize,
    pub branch_roots: usize,
    pub branch_cells: usize,
    pub partition_mirror: usize,
    pub derived_indexes: usize,
    pub framing_and_metadata: usize,
}

impl From<worth_relational::facade::durability::NativeCheckpointSectionBytes>
    for WorthQueryNativeCheckpointSectionBytes
{
    fn from(value: worth_relational::facade::durability::NativeCheckpointSectionBytes) -> Self {
        Self {
            total: value.total,
            envelopes: value.envelopes,
            branch_roots: value.branch_roots,
            branch_cells: value.branch_cells,
            partition_mirror: value.partition_mirror,
            derived_indexes: value.derived_indexes,
            framing_and_metadata: value.framing_and_metadata,
        }
    }
}

impl WorthQueryApplicationCheckpointSectionBytes {
    pub(super) const fn new(
        framing: usize,
        native: usize,
        accepted_outputs: usize,
        accepted_output_count: usize,
        native_sections: Option<WorthQueryNativeCheckpointSectionBytes>,
    ) -> Self {
        Self {
            framing,
            native,
            accepted_outputs,
            accepted_output_count,
            native_sections,
        }
    }

    pub const fn framing_bytes(self) -> usize {
        self.framing
    }

    pub const fn native_bytes(self) -> usize {
        self.native
    }

    pub const fn accepted_output_bytes(self) -> usize {
        self.accepted_outputs
    }

    pub const fn accepted_output_count(self) -> usize {
        self.accepted_output_count
    }

    pub const fn native_sections(self) -> Option<WorthQueryNativeCheckpointSectionBytes> {
        self.native_sections
    }

    pub const fn total_bytes(self) -> usize {
        self.framing + self.native + self.accepted_outputs
    }
}
