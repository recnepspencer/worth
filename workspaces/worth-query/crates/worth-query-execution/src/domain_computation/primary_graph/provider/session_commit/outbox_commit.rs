//! Exact dispatch-outbox identity sealed from the authoritative commit artifact.

use crate::domain_computation::application_aftermath::{
    WorthQueryDispatchOutboxRecord, WorthQueryPendingDispatchOutbox,
};
use worth_relational::facade::transactions::{CommitResult, RecordRef};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryCommittedDispatchOutboxBinding {
    record: WorthQueryDispatchOutboxRecord,
    record_ref: RecordRef,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryCommittedDispatchOutboxBindingDenial {
    CreatedEntityMissing,
}

/// Owner-sealed outcome of binding one pending outbox create to its commit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryCommittedDispatchOutboxResolution {
    binding: Result<
        Option<WorthQueryCommittedDispatchOutboxBinding>,
        WorthQueryCommittedDispatchOutboxBindingDenial,
    >,
}

/// Unforgeable proof that committed-outbox resolution succeeded.
///
/// The wrapped binding still distinguishes exact `Some` from honest `None`.
/// No receipt path can manufacture this seal from a denied resolution.
pub struct WorthQueryCommittedDispatchOutboxReceiptSeal
{
    binding: Option<WorthQueryCommittedDispatchOutboxBinding>,
}

impl std::fmt::Display for WorthQueryCommittedDispatchOutboxBindingDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CreatedEntityMissing => {
                formatter.write_str("committed dispatch outbox record identity is absent")
            }
        }
    }
}

impl std::error::Error for WorthQueryCommittedDispatchOutboxBindingDenial {}

impl WorthQueryCommittedDispatchOutboxResolution {
    pub fn from_commit(
        pending: Option<&WorthQueryPendingDispatchOutbox>,
        committed: &CommitResult,
    ) -> Self {
        Self {
            binding: WorthQueryCommittedDispatchOutboxBinding::from_commit(pending, committed),
        }
    }

    /// Sole fallible decision consumed by receipt assembly.
    pub fn seal_for_receipt(
        &self,
    ) -> Result<
        WorthQueryCommittedDispatchOutboxReceiptSeal,
        WorthQueryCommittedDispatchOutboxBindingDenial,
    > {
        match &self.binding {
            Ok(binding) => Ok(WorthQueryCommittedDispatchOutboxReceiptSeal {
                binding: binding.clone(),
            }),
            Err(denial) => Err(*denial),
        }
    }
}

impl WorthQueryCommittedDispatchOutboxReceiptSeal {
    pub fn binding(&self) -> Option<&WorthQueryCommittedDispatchOutboxBinding> {
        self.binding.as_ref()
    }

    pub fn into_binding(self) -> Option<WorthQueryCommittedDispatchOutboxBinding> {
        self.binding
    }
}

impl WorthQueryCommittedDispatchOutboxBinding {
    pub fn from_commit(
        pending: Option<&WorthQueryPendingDispatchOutbox>,
        committed: &CommitResult,
    ) -> Result<Option<Self>, WorthQueryCommittedDispatchOutboxBindingDenial> {
        let Some(pending) = pending else {
            return Ok(None);
        };
        let Some(entity_id) = committed.created_entity(pending.created_entity()) else {
            return Err(WorthQueryCommittedDispatchOutboxBindingDenial::CreatedEntityMissing);
        };
        let record_ref = RecordRef::Entity(entity_id);
        Ok(Some(Self {
            record: pending.record().clone(),
            record_ref,
        }))
    }

    pub fn fixture_from_commit(
        layout: &crate::domain_computation::application_aftermath::WorthQueryDispatchOutboxLayout,
        record: Option<&WorthQueryDispatchOutboxRecord>,
        committed: &CommitResult,
    ) -> Result<Option<Self>, WorthQueryCommittedDispatchOutboxBindingDenial> {
        let pending =
            crate::domain_computation::application_aftermath::bind_dispatch_outbox_create_intent(
                Some(layout),
                record,
                worth_relational::facade::identity::PartitionId::main(),
            );
        Self::from_commit(pending.as_ref().map(|(_, pending)| pending), committed)
    }

    pub const fn record(&self) -> &WorthQueryDispatchOutboxRecord {
        &self.record
    }

    pub const fn record_ref(&self) -> &RecordRef {
        &self.record_ref
    }

    pub const fn fixture(record: WorthQueryDispatchOutboxRecord, record_ref: RecordRef) -> Self {
        Self { record, record_ref }
    }
}
