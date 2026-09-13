use super::super::owner_artifact_verification::OwnerObservation;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct OwnerSemanticVerificationCounters {
    artifacts_attempted: u64,
    artifacts_verified: u64,
    bytes_verified: u64,
    bytes_read: u64,
    decoder_allocation_bytes: u64,
    peak_buffer_bytes: u64,
}

impl OwnerSemanticVerificationCounters {
    pub(crate) fn record_attempt(mut self) -> Option<Self> {
        self.artifacts_attempted = self.artifacts_attempted.checked_add(1)?;
        Some(self)
    }

    pub(crate) fn record(mut self, observation: OwnerObservation) -> Option<Self> {
        self.artifacts_verified = self.artifacts_verified.checked_add(1)?;
        self.bytes_verified = self.bytes_verified.checked_add(observation.bytes_read)?;
        self.decoder_allocation_bytes = self
            .decoder_allocation_bytes
            .max(observation.decoder_allocation_bytes);
        self.peak_buffer_bytes = self.peak_buffer_bytes.max(observation.peak_buffer_bytes);
        Some(self)
    }

    pub(crate) fn record_read(mut self, bytes: u64) -> Option<Self> {
        self.bytes_read = self.bytes_read.checked_add(bytes)?;
        Some(self)
    }

    pub(crate) const fn artifacts_verified(self) -> u64 {
        self.artifacts_verified
    }

    pub(crate) const fn artifacts_attempted(self) -> u64 {
        self.artifacts_attempted
    }

    pub(crate) const fn bytes_verified(self) -> u64 {
        self.bytes_verified
    }

    pub(crate) const fn bytes_read(self) -> u64 {
        self.bytes_read
    }

    pub(crate) const fn decoder_allocation_bytes(self) -> u64 {
        self.decoder_allocation_bytes
    }

    pub(crate) const fn peak_buffer_bytes(self) -> u64 {
        self.peak_buffer_bytes
    }
}
