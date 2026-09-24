/// Bytes written by Query's checkpoint encoder for one captured artifact.
///
/// These are diagnostic sizes, not readmission authority. The native payload
/// remains opaque to Query and is counted as one section here.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryApplicationCheckpointSectionBytes {
    framing: usize,
    native: usize,
    accepted_outputs: usize,
    accepted_output_count: usize,
}

impl WorthQueryApplicationCheckpointSectionBytes {
    pub(super) const fn new(
        framing: usize,
        native: usize,
        accepted_outputs: usize,
        accepted_output_count: usize,
    ) -> Self {
        Self {
            framing,
            native,
            accepted_outputs,
            accepted_output_count,
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

    pub const fn total_bytes(self) -> usize {
        self.framing + self.native + self.accepted_outputs
    }
}
