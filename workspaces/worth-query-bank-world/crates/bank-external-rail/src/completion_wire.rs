//! Version 1 Bank rail completion wire contract.
//!
//! The HTTP body is exactly `signed_bytes || signature[64]`. Integers are
//! unsigned big-endian. Text is UTF-8 with a `u16` byte length; token and
//! payload are bytes with `u16` and `u32` lengths respectively. The signed
//! bytes, in order, are: magic, audience, source, key epoch (`u64`), message
//! identity (`32` bytes), issued/expiry Unix seconds (`u64` each), protocol
//! identity, protocol version (`u16`), correlation family, correlation token,
//! and exact effect payload. No field may be omitted or trailing bytes added.
//! The decoder at the Bank HTTP boundary is independent of the rail encoder.

/// Domain-separated prefix of the signed completion bytes.
pub const COMPLETION_V1_MAGIC: &[u8; 16] = b"BANK-COMPLETION1";
/// Domain-separated prefix of a signed Bank custody response.
pub const CUSTODY_ACK_V1_MAGIC: &[u8; 16] = b"BANK-CUSTODY-ACK";
/// The signature size of Ed25519.
pub const SIGNATURE_BYTES: usize = 64;
/// First implementation's absolute envelope ceiling, including signature.
pub const MAXIMUM_ENVELOPE_BYTES: usize = 4096;
/// One signed body produced only from the rail's completed-effect owner.
#[derive(Clone)]
pub struct SignedRailCompletion {
    bytes: Vec<u8>,
    message_id: [u8; 32],
    digest: [u8; 32],
}

impl SignedRailCompletion {
    pub(crate) fn new(bytes: Vec<u8>, message_id: [u8; 32], digest: [u8; 32]) -> Self {
        Self {
            bytes,
            message_id,
            digest,
        }
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn message_id(&self) -> &[u8; 32] {
        &self.message_id
    }

    /// SHA-256 of the complete signed HTTP body, for exact ACK binding.
    pub fn digest(&self) -> &[u8; 32] {
        &self.digest
    }
}

/// Authenticated Bank custody result. These values do not claim World success.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum RailCustodyAckPosture {
    AcceptedPending = 1,
    Performed = 2,
    AlreadyAccepted = 3,
    PermanentDenied = 4,
    AlreadyCompleted = 5,
}

impl RailCustodyAckPosture {
    pub(crate) const fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            1 => Some(Self::AcceptedPending),
            2 => Some(Self::Performed),
            3 => Some(Self::AlreadyAccepted),
            4 => Some(Self::PermanentDenied),
            5 => Some(Self::AlreadyCompleted),
            _ => None,
        }
    }
}

/// The ACK body is magic, `u8` posture, message ID, completion-body SHA-256,
/// then a 64-byte Bank Ed25519 signature over all preceding bytes.
pub const CUSTODY_ACK_V1_BYTES: usize = 16 + 1 + 32 + 32 + SIGNATURE_BYTES;
