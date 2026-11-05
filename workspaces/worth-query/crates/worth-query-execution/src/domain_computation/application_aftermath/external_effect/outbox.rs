//! Query-owned dispatch outbox record co-committed with the mutation (R8.25).

use std::collections::BTreeMap;

use worth_foundational::facade::{
    AspectValue, BoundaryProtocolIdentity, BoundaryProtocolVersion, CanonicalDigestId,
    InternedString,
};
use worth_query_installation::facade::{
    InstalledExternalEffectContract, WorthQueryExternalEffectCorrelationFamily,
};
use worth_relational::facade::transactions::{
    CreateIntent, CreatedEntityRef, EntitySpec, MutationIntent,
};

use crate::domain_computation::vocabulary::application_aftermath::correlation::ExternalEffectCorrelationIdentity;
use crate::domain_computation::vocabulary::application_aftermath::outbox::WorthQueryDispatchOutboxLayout;
use crate::domain_computation::vocabulary::application_aftermath::outbox::WorthQueryDispatchOutboxRecord;
use crate::domain_computation::vocabulary::application_aftermath::outbox::bind_dispatch_outbox_create_intent;







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




mod court_doors_for_test;
