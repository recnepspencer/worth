//! Co-committed Query registration for one original keyed application effect.
use super::{recorded_origin, WorthQueryProductIdempotencyAffinity};
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationCommitOutcomeIdentity, WorthQueryApplicationIdempotencyBinding,
};
use crate::domain_computation::primary_graph::schema_layout::WorthQueryProviderIdempotencyLayout;
use std::collections::BTreeMap;
use worth_foundational::facade::{AspectValue, InternedString};
use worth_relational::facade::transactions::{
    AspectFieldPatch, CreateIntent, EntitySpec, MutationIntent,
};

// Native scopes appended participants' local keys; this issued root key remains
// the creation correspondence for the original application operation.
pub const ORIGINAL_CLIENT_KEY_PREFIX: &str = "worth-query-idempotency:";

/// The local create coordinate of `binding`'s original idempotency record.
pub(in crate::domain_computation::primary_graph) fn original_client_key(
    binding: WorthQueryApplicationIdempotencyBinding,
) -> String {
    format!("{ORIGINAL_CLIENT_KEY_PREFIX}{}", binding.key_text())
}

pub fn idempotency_create_intent(
    layout: &WorthQueryProviderIdempotencyLayout,
    binding: WorthQueryApplicationIdempotencyBinding,
    outcome_identity: WorthQueryApplicationCommitOutcomeIdentity,
    emitted_effect_count: u64,
    mutation_partition: worth_relational::facade::identity::PartitionId,
    product: Option<&worth_runtime_world::facade::ProductBranchObservation>,
    has_output_settlement: bool,
) -> MutationIntent {
    let key = binding.key_text();
    let fields = BTreeMap::from([
        (
            layout.key_locator.clone(),
            AspectValue::String(InternedString::from(key.clone())),
        ),
        (
            layout.intent_locator.clone(),
            AspectValue::String(InternedString::from(binding.intent_text())),
        ),
        (
            layout.outcome_identity_locator.clone(),
            AspectValue::UInt64(outcome_identity.get()),
        ),
        (
            layout.emitted_effect_count_locator.clone(),
            AspectValue::UInt64(emitted_effect_count),
        ),
        (layout.affinity_locator.clone(), AspectValue::String(InternedString::from(
            product.map(|product| recorded_origin::affinity_text(
                &WorthQueryProductIdempotencyAffinity::from_observation(product),
            )).unwrap_or_else(|| "memory-only".to_owned()),
        ))),
        (layout.association_locator.clone(), AspectValue::UInt64(recorded_origin::ENCLOSING_NATIVE_ASSOCIATION)),
        (layout.output_settlement_locator.clone(), AspectValue::UInt64(
            match has_output_settlement {
                false => worth_foundational::facade::DurableQueryOutputSettlement::Absent,
                true => worth_foundational::facade::DurableQueryOutputSettlement::OriginalOccurrenceSlot { slot: 0 },
            }.recorded_code().expect("the original application's output slot is representable"),
        )),
    ]);
    MutationIntent::Create(CreateIntent::Entity(EntitySpec {
        partition_id: mutation_partition,
        kind_id: layout.entity_kind,
        client_key: worth_relational::facade::symbols::ClientKey::raw(original_client_key(binding)),
        fields: AspectFieldPatch::from(fields),
    }))
}
