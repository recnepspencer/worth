//! Exact dispatch-outbox identity sealed from the authoritative commit artifact.

#[cfg(test)]
mod tests;

use crate::domain_computation::application_aftermath::{
    WorthQueryDispatchOutboxRecord, WorthQueryPendingDispatchOutbox,
};
use worth_relational::facade::transactions::{CommitResult, RecordRef};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation) struct WorthQueryCommittedDispatchOutboxBinding {
    record: WorthQueryDispatchOutboxRecord,
    record_ref: RecordRef,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum WorthQueryCommittedDispatchOutboxBindingDenial
{
    CreatedEntityMissing,
}

/// Owner-sealed outcome of binding one pending outbox create to its commit.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryCommittedDispatchOutboxResolution
{
    binding: Result<
        Option<WorthQueryCommittedDispatchOutboxBinding>,
        WorthQueryCommittedDispatchOutboxBindingDenial,
    >,
}

/// Unforgeable proof that committed-outbox resolution succeeded.
///
/// The wrapped binding still distinguishes exact `Some` from honest `None`.
/// No receipt path can manufacture this seal from a denied resolution.
pub(in crate::domain_computation::primary_graph) struct WorthQueryCommittedDispatchOutboxReceiptSeal
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
    pub(in crate::domain_computation::primary_graph::provider) fn from_commit(
        pending: Option<&WorthQueryPendingDispatchOutbox>,
        committed: &CommitResult,
    ) -> Self {
        Self {
            binding: WorthQueryCommittedDispatchOutboxBinding::from_commit(pending, committed),
        }
    }

    /// Sole fallible decision consumed by receipt assembly.
    pub(in crate::domain_computation::primary_graph) fn seal_for_receipt(
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
    pub(in crate::domain_computation::primary_graph) fn binding(
        &self,
    ) -> Option<&WorthQueryCommittedDispatchOutboxBinding> {
        self.binding.as_ref()
    }

    pub(in crate::domain_computation::primary_graph) fn into_binding(
        self,
    ) -> Option<WorthQueryCommittedDispatchOutboxBinding> {
        self.binding
    }
}

impl WorthQueryCommittedDispatchOutboxBinding {
    pub(super) fn from_commit(
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

    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn fixture_from_commit(
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

    pub(in crate::domain_computation) const fn record(&self) -> &WorthQueryDispatchOutboxRecord {
        &self.record
    }

    pub(in crate::domain_computation) const fn record_ref(&self) -> &RecordRef {
        &self.record_ref
    }

    #[cfg(test)]
    pub(in crate::domain_computation) const fn fixture(
        record: WorthQueryDispatchOutboxRecord,
        record_ref: RecordRef,
    ) -> Self {
        Self { record, record_ref }
    }
}
