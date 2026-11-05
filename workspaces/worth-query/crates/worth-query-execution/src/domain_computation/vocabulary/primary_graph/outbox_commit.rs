//! Moved whole from `domain_computation::primary_graph::provider::session_commit::outbox_commit`: the items below and every impl they own.
use crate::domain_computation::vocabulary::application_aftermath::outbox::WorthQueryDispatchOutboxRecord;
use crate::domain_computation::vocabulary::application_aftermath::outbox::WorthQueryPendingDispatchOutbox;
use worth_relational::facade::transactions::CommitResult;
use worth_relational::facade::transactions::RecordRef;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryCommittedDispatchOutboxBinding {
    record: WorthQueryDispatchOutboxRecord,
    record_ref: RecordRef,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryCommittedDispatchOutboxBindingDenial {
    CreatedEntityMissing,
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

impl WorthQueryCommittedDispatchOutboxBinding {
    pub(in crate::domain_computation) fn from_commit(
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

    pub(in crate::domain_computation) fn fixture_from_commit(
        layout: &crate::domain_computation::vocabulary::application_aftermath::outbox::WorthQueryDispatchOutboxLayout,
        record: Option<&WorthQueryDispatchOutboxRecord>,
        committed: &CommitResult,
    ) -> Result<Option<Self>, WorthQueryCommittedDispatchOutboxBindingDenial> {
        let pending =
            crate::domain_computation::vocabulary::application_aftermath::outbox::bind_dispatch_outbox_create_intent(
                Some(layout),
                record,
                worth_relational::facade::identity::PartitionId::main(),
            );
        Self::from_commit(pending.as_ref().map(|(_, pending)| pending), committed)
    }

    pub(in crate::domain_computation) const fn record(&self) -> &WorthQueryDispatchOutboxRecord {
        &self.record
    }

    pub(in crate::domain_computation) const fn record_ref(&self) -> &RecordRef {
        &self.record_ref
    }

    pub const fn fixture(record: WorthQueryDispatchOutboxRecord, record_ref: RecordRef) -> Self {
        Self { record, record_ref }
    }
}

impl WorthQueryCommittedDispatchOutboxBinding {
    #[doc(hidden)]
    pub fn fixture_from_commit_for_test(
        layout: &crate::domain_computation::vocabulary::application_aftermath::outbox::WorthQueryDispatchOutboxLayout,
        record: Option<&WorthQueryDispatchOutboxRecord>,
        committed: &CommitResult,
    ) -> Result<Option<Self>, WorthQueryCommittedDispatchOutboxBindingDenial> {
        Self::fixture_from_commit(layout, record, committed)
    }
}
