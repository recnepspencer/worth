//! The durable medium port: the one seam through which Relational's committed
//! history and branch lifecycle transitions reach durable storage.
//!
//! Relational never names a storage engine. A runtime with an attached medium
//! publishes each positioned commit's canonical artifact to it after the
//! commit is fully prepared and its position is reserved, and installs the
//! commit into live state only after the medium answers `Committed`. A fork,
//! archive or delete publishes its branch transition artifact the same way,
//! before any live effect. Live state therefore never selects a commit or a
//! branch the medium has not made durable.
//!
//! The medium's owner (Worth Store's Relational home) also owns recovery
//! sequencing: it hands the recovered checkpoint, journal tail and transition
//! tail back to Relational through
//! [`DurabilityAccess::recovery_plan_from_source`](crate::durability::access::DurabilityAccess::recovery_plan_from_source).

mod native_journal_source;
pub(crate) mod source_retirement;
pub(crate) use native_journal_source::NativeJournalSourceEdge;
pub use source_retirement::{RelationalMediumRetirementDenial, RelationalMediumRetirementProgress};
mod checkpoint_selection;
pub use checkpoint_selection::{
    CheckpointSelectionIdentity, CheckpointSelectionObservation, OpaqueCapturedCheckpointSubject,
    RelationalCheckpointAcknowledgementDenial, RelationalCheckpointAcknowledgementProgress,
};
mod cold_source;
pub use cold_source::{RelationalDurableColdSource, RelationalDurableMemberLocator};
mod member_custody;
pub(crate) use member_custody::{
    NativeMemberRetirementHandoff, NativeMemberSlotDenial, NativeSourceMemberState,
};
pub use member_custody::{
    RelationalDurableMemberCustody, RelationalDurableMemberRetirement,
    RelationalDurableMemberRetirementObservation, RelationalDurableMemberRetirementRefusal,
    RetiredRelationalDurableMemberCustody,
};
mod read_protection;
pub use read_protection::{
    RelationalDurableOwnedReadRefusal, RelationalDurableReadBounds, RelationalDurableReadPayload,
    RelationalDurableReadProtection,
};
mod transition_read;
pub(crate) mod unresolved;
pub use transition_read::RelationalDurableTransitionReadFailure;

pub use unresolved::{RelationalDurabilityUnresolved, RelationalDurabilityUnresolvedSubject};
pub use worth_foundational::facade::DurablePublicationFate;

pub(crate) mod publication_refusal;
pub use publication_refusal::RelationalDurablePublicationRefusal;

pub(crate) mod home_write_denial;

pub(crate) mod step_stop;
pub(crate) use step_stop::RelationalDurableStepStop;
