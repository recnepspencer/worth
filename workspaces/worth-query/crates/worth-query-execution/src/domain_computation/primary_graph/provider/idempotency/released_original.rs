//! Read-only terminal original: an owner effect was released, never published.
use super::snapshot_resolution::RecordedOriginalBinding;
use super::WorthQueryProviderIdempotencyResolutionDenial as Denial;
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_relational::facade::{
    durability::ReadmittedRelationalCreatedEntity, transactions::RelationalCommitSemanticResult,
};
use worth_runtime_world::facade::WorldReleasedOriginal;

#[derive(Debug)]
pub(in crate::domain_computation) struct OriginalBinding {
    pub(in crate::domain_computation::primary_graph::provider) binding: RecordedOriginalBinding,
    _funding: Option<ExecutionMemoryReservation>,
}
impl PartialEq for OriginalBinding {
    fn eq(&self, other: &Self) -> bool {
        self.binding == other.binding
    }
}
impl Eq for OriginalBinding {}

/// Original keyed owner meaning and its actual terminal release. It grants no
/// occurrence, application receipt, delivery, adoption or custody authority.
///
/// ```compile_fail,E0451
/// use worth_query_execution::facade::primary_graph::WorthQueryReleasedApplicationEffect;
/// let _ = WorthQueryReleasedApplicationEffect { binding: panic!(), original: panic!() };
/// ```
///
/// ```compile_fail,E0599
/// use worth_query_execution::facade::primary_graph::WorthQueryReleasedApplicationEffect;
/// fn cannot_reacquire_custody(original: WorthQueryReleasedApplicationEffect) {
///     original.into_recovery();
/// }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryReleasedApplicationEffect {
    binding: Arc<OriginalBinding>,
    original: WorldReleasedOriginal,
}
impl WorthQueryReleasedApplicationEffect {
    pub(in crate::domain_computation::primary_graph::provider) fn new(
        binding: Arc<OriginalBinding>,
        original: WorldReleasedOriginal,
    ) -> Self {
        assert!(
            original.native_semantic_result().is_some(),
            "effect-bearing terminal original"
        );
        Self { binding, original }
    }
    pub fn original_effect(&self) -> &WorldReleasedOriginal {
        &self.original
    }
    pub fn idempotency_key_identity(&self) -> &[u8; 32] {
        &self.binding.binding.key
    }
    pub fn recorded_intent(&self) -> &str {
        &self.binding.binding.intent
    }
    pub fn recorded_affinity(&self) -> &str {
        &self.binding.binding.affinity
    }
    pub fn outcome_identity(&self) -> crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitOutcomeIdentity{
        self.binding.binding.outcome
    }
    pub fn emitted_effect_count(&self) -> usize {
        self.binding.binding.emitted
    }
    pub fn exact_output_settlement(
        &self,
    ) -> &worth_foundational::facade::DurableQueryOutputSettlement {
        &self.binding.binding.settlement
    }
    pub fn semantic_result(&self) -> &RelationalCommitSemanticResult {
        self.original
            .native_semantic_result()
            .expect("sealed effect-bearing original")
    }
}

pub(in crate::domain_computation::primary_graph::provider) fn read<'source>(
    execution: &ExecutionRequest<'_, '_>,
    layout: &crate::domain_computation::primary_graph::schema_layout::WorthQueryProviderIdempotencyLayout,
    semantic: &RelationalCommitSemanticResult,
    mut row: impl FnMut(
        worth_relational::facade::identity::EntityId,
    ) -> Result<Option<ReadmittedRelationalCreatedEntity<'source>>, Denial>,
) -> Result<Arc<OriginalBinding>, Denial> {
    let mut original = None;
    for created in semantic.created_entities() {
        if created.key.kind != layout.entity_kind {
            continue;
        }
        if original.is_some() {
            return Err(Denial::Unavailable);
        }
        let row = row(created.assigned)?.ok_or(Denial::Unavailable)?;
        let bytes = row
            .owned_allocation_capacity_bytes()
            .checked_mul(2)
            .and_then(|n| {
                n.checked_add(
                    (std::mem::size_of::<OriginalBinding>() + 2 * std::mem::size_of::<usize>())
                        as u64,
                )
            })
            .ok_or(Denial::Unavailable)?;
        let funding = execution
            .reserve_retained_memory(bytes)
            .map_err(|_| Denial::RetentionCapacityExhausted)?;
        let binding = super::snapshot_resolution::original_binding::read(&row, layout)?;
        original = Some(Arc::new(OriginalBinding {
            binding,
            _funding: funding,
        }));
    }
    original.ok_or(Denial::Unavailable)
}
