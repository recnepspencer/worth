//! Query-owned dispatch outbox record co-committed with the mutation (R8.25).

use worth_relational::facade::transactions::MutationIntent;

use crate::domain_computation::vocabulary::application_aftermath::outbox::bind_dispatch_outbox_create_intent;
use crate::domain_computation::vocabulary::application_aftermath::outbox::WorthQueryDispatchOutboxLayout;
use crate::domain_computation::vocabulary::application_aftermath::outbox::WorthQueryDispatchOutboxRecord;

/// Build a create intent for a declared external effect. Returns `None` when
/// the operation declares no external effect (R8.4 — pay exactly zero).
pub fn dispatch_outbox_create_intent(
    layout: Option<&WorthQueryDispatchOutboxLayout>,
    record: Option<&WorthQueryDispatchOutboxRecord>,
) -> Option<MutationIntent> {
    bind_dispatch_outbox_create_intent(
        layout,
        record,
        worth_relational::facade::identity::PartitionId::main(),
    )
    .map(|(intent, _)| intent)
}
