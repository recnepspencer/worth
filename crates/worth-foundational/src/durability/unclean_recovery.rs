//! Descriptive refusal of an unsupported unclean Home tail.
use super::DurableOwnerFamily;
use crate::CanonicalDigestId;

/// The exact owner effect observed without an occurrence or disposition.
/// This names durable data, never a recovered fate or a serving capability.
/// Cleanup track C5 deletes this refusal after C1 supplies crash recovery.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UncleanRecoveryUnsupported {
    pub owner: DurableOwnerFamily,
    pub effect: CanonicalDigestId,
}
