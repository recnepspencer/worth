//! Canonical physical-work obligation v6 byte mechanism.
//!
//! Store owns journal I/O, pending-name policy, operation lifecycle and
//! recovery disposition. This module owns only persisted field meaning.

mod checksum;
mod encode;
mod field_code;
mod identity;
mod target_shape;

pub use checksum::{
    physical_work_obligation_v6_incarnation_digest, physical_work_obligation_v6_scope_digest,
};
pub use encode::{
    decode_physical_work_obligation_v6, encode_physical_work_obligation_v6,
    PhysicalWorkObligationV6, PhysicalWorkObligationV6Denial,
};
pub use field_code::{
    PhysicalWorkArtifactCode, PhysicalWorkCheckpointActionCode,
    PhysicalWorkObligationOperationCode, PhysicalWorkObligationTargetCode,
};
pub use identity::PhysicalWorkObligationIdentity;

pub use crate::integrity_declarations::families::{
    PHYSICAL_WORK_OBLIGATION_V6_RECORD_BYTES, PHYSICAL_WORK_OBLIGATION_V6_VERSION,
};

#[cfg(test)]
mod tests;
