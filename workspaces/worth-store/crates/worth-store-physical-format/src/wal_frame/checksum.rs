use sha2::{Digest, Sha256};

use super::{WalFrameV1Denial, WalFrameV1Header, WAL_FRAME_V1_HEADER_BYTES};

pub fn wal_frame_v1_declared_identity_digest(identity: &[u8]) -> [u8; 32] {
    wal_frame_v1_validation_digest(identity)
}

/// SHA-256 mechanism used for WAL v1 validation evidence outside persisted
/// fields, including exact-scope records in the runtime integrity owner.
pub fn wal_frame_v1_validation_digest(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

/// Incremental WAL v1 checksum calculation for bounded streaming readers.
pub struct WalFrameV1ChecksumCalculator {
    payload: Sha256,
    frame: Sha256,
    observed_payload_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WalFrameV1CalculatedChecksums {
    payload: [u8; 32],
    frame: [u8; 32],
}

impl WalFrameV1ChecksumCalculator {
    pub fn new(header: &[u8; WAL_FRAME_V1_HEADER_BYTES]) -> Self {
        let mut frame = Sha256::new();
        frame.update(header);
        Self {
            payload: Sha256::new(),
            frame,
            observed_payload_bytes: 0,
        }
    }

    pub fn update_payload(&mut self, bytes: &[u8]) -> Result<(), WalFrameV1Denial> {
        self.observed_payload_bytes = self
            .observed_payload_bytes
            .checked_add(bytes.len() as u64)
            .ok_or(WalFrameV1Denial::PayloadLengthMismatch)?;
        self.payload.update(bytes);
        self.frame.update(bytes);
        Ok(())
    }

    pub fn finish(
        self,
        header: WalFrameV1Header,
        footer: &[u8; 32],
    ) -> Result<WalFrameV1CalculatedChecksums, WalFrameV1Denial> {
        let calculated = self.finish_calculation(header)?;
        if calculated.payload != header.payload_digest() || calculated.frame != *footer {
            return Err(WalFrameV1Denial::ChecksumMismatch);
        }
        Ok(calculated)
    }

    /// Finishes the canonical checksum mechanism without collapsing which
    /// owner-facing checksum comparison failed.
    pub fn finish_calculation(
        self,
        header: WalFrameV1Header,
    ) -> Result<WalFrameV1CalculatedChecksums, WalFrameV1Denial> {
        self.finish_for_payload_bytes(header.payload_bytes())
    }

    pub(super) fn finish_for_payload_bytes(
        self,
        expected_payload_bytes: u64,
    ) -> Result<WalFrameV1CalculatedChecksums, WalFrameV1Denial> {
        if self.observed_payload_bytes != expected_payload_bytes {
            return Err(WalFrameV1Denial::PayloadLengthMismatch);
        }
        Ok(WalFrameV1CalculatedChecksums {
            payload: self.payload.finalize().into(),
            frame: self.frame.finalize().into(),
        })
    }
}

impl WalFrameV1CalculatedChecksums {
    pub const fn payload(self) -> [u8; 32] {
        self.payload
    }

    pub const fn frame(self) -> [u8; 32] {
        self.frame
    }
}
