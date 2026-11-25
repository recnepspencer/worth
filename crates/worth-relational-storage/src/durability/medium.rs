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
//! [`DurabilityAccess::recovery_plan_from_source`](crate::runtime::durability_access::DurabilityAccess::recovery_plan_from_source).

pub mod native_journal_source;
pub mod source_retirement;
pub use native_journal_source::NativeJournalSourceEdge;
pub use source_retirement::{RelationalMediumRetirementDenial, RelationalMediumRetirementProgress};
pub mod checkpoint_acknowledgement_denial;
pub use checkpoint_acknowledgement_denial::RelationalCheckpointAcknowledgementDenial;
pub mod cold_source;
pub use cold_source::{RelationalDurableColdSource, RelationalDurableMemberLocator};
pub mod member_custody;
pub use member_custody::{
    NativeMemberRetirementHandoff, NativeMemberSlotDenial, NativeSourceMemberState,
};
pub use member_custody::{
    RelationalDurableMemberCustody, RelationalDurableMemberRetirement,
    RelationalDurableMemberRetirementObservation, RelationalDurableMemberRetirementRefusal,
    RetiredRelationalDurableMemberCustody,
};
pub mod read_protection;
pub use read_protection::{
    RelationalDurableOwnedReadRefusal, RelationalDurableReadBounds, RelationalDurableReadPayload,
    RelationalDurableReadProtection,
};
pub mod transition_read;
pub use transition_read::RelationalDurableTransitionReadFailure;
pub use worth_relational_contracts::internal_port::durability::medium::unresolved;

pub use unresolved::{RelationalDurabilityUnresolved, RelationalDurabilityUnresolvedSubject};
pub use worth_foundational::facade::DurablePublicationFate;

pub use publication_refusal::RelationalDurablePublicationRefusal;
pub use worth_relational_contracts::internal_port::durability::medium::publication_refusal;

pub use worth_relational_contracts::internal_port::durability::medium::home_write_denial;

pub mod step_stop;
pub use step_stop::RelationalDurableStepStop;

pub mod owned_read_stop;
