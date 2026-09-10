use std::any::{Any, TypeId};
use std::sync::Arc;

/// Provider-owner declaration of the complete behavior that is relevant to
/// conditional continuity. Bridge compares the associated contract's concrete
/// type and typed `Eq` value; provider implementation identity is intentionally
/// reserved for exact execution affinity.
pub trait BridgeConditionalProviderSemantics: Send + Sync + 'static {
    type SemanticContract: Eq + Send + Sync + 'static;

    fn semantic_contract(&self) -> Self::SemanticContract;

    /// Heap allocations whose lifetime is extended by retaining this provider
    /// and its captured semantic contract. Each allocation is reported once,
    /// even when both values hold an `Arc` to the same backing allocation.
    fn retained_heap_bytes(
        &self,
        semantic_contract: &Self::SemanticContract,
    ) -> Result<BridgeConditionalProviderHeapRetention, BridgeConditionalProviderRetentionOverflow>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BridgeConditionalProviderRetentionOverflow;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BridgeConditionalProviderHeapRetention {
    provider_state_bytes: u64,
    semantic_contract_state_bytes: u64,
}

impl BridgeConditionalProviderHeapRetention {
    pub const fn none() -> Self {
        Self {
            provider_state_bytes: 0,
            semantic_contract_state_bytes: 0,
        }
    }

    pub const fn new(provider_state_bytes: u64, semantic_contract_state_bytes: u64) -> Self {
        Self {
            provider_state_bytes,
            semantic_contract_state_bytes,
        }
    }

    pub fn try_from_parts(
        provider_state_parts: impl IntoIterator<Item = u64>,
        semantic_contract_state_parts: impl IntoIterator<Item = u64>,
    ) -> Result<Self, BridgeConditionalProviderRetentionOverflow> {
        fn checked(
            parts: impl IntoIterator<Item = u64>,
        ) -> Result<u64, BridgeConditionalProviderRetentionOverflow> {
            parts.into_iter().try_fold(0u64, |total, part| {
                total
                    .checked_add(part)
                    .ok_or(BridgeConditionalProviderRetentionOverflow)
            })
        }
        Ok(Self::new(
            checked(provider_state_parts)?,
            checked(semantic_contract_state_parts)?,
        ))
    }

    pub fn arc_allocation_bytes<T: ?Sized>(value: &T) -> u64 {
        let (layout, _) = std::alloc::Layout::new::<[std::sync::atomic::AtomicUsize; 2]>()
            .extend(std::alloc::Layout::for_value(value))
            .expect("one live value has a representable Arc allocation layout");
        layout.pad_to_align().size() as u64
    }

    pub const fn provider_state_bytes(self) -> u64 {
        self.provider_state_bytes
    }

    pub const fn semantic_contract_state_bytes(self) -> u64 {
        self.semantic_contract_state_bytes
    }
}

#[derive(Clone)]
pub(super) struct BridgeErasedProviderSemanticContract {
    contract_type: TypeId,
    contract: Arc<dyn Any + Send + Sync>,
    equivalent: fn(&dyn Any, &dyn Any) -> bool,
    retained_heap:
        Result<BridgeConditionalProviderHeapRetention, BridgeConditionalProviderRetentionOverflow>,
}

impl BridgeErasedProviderSemanticContract {
    pub(super) fn capture<P>(provider: &P) -> Self
    where
        P: BridgeConditionalProviderSemantics,
    {
        fn equivalent<C: Eq + 'static>(left: &dyn Any, right: &dyn Any) -> bool {
            left.downcast_ref::<C>()
                .zip(right.downcast_ref::<C>())
                .is_some_and(|(left, right)| left == right)
        }

        let semantic_contract = provider.semantic_contract();
        let retained_heap = provider.retained_heap_bytes(&semantic_contract);
        Self {
            contract_type: TypeId::of::<P::SemanticContract>(),
            contract: Arc::new(semantic_contract),
            equivalent: equivalent::<P::SemanticContract>,
            retained_heap,
        }
    }

    pub(super) fn is_equivalent_to(&self, candidate: &Self) -> bool {
        self.contract_type == candidate.contract_type
            && (self.equivalent)(self.contract.as_ref(), candidate.contract.as_ref())
    }

    pub(super) fn retained_arc_bytes(
        &self,
    ) -> Result<u64, super::retention::BridgeRetentionDenial> {
        super::retention::sum(&[
            super::retention::arc_value_charge(self.contract.as_ref())?,
            self.retained_heap
                .map_err(|_| super::retention::BridgeRetentionDenial::BytesExhausted)?
                .provider_state_bytes(),
            self.retained_heap
                .map_err(|_| super::retention::BridgeRetentionDenial::BytesExhausted)?
                .semantic_contract_state_bytes(),
        ])
    }
}

#[derive(Clone, Default)]
pub(super) struct BridgeConditionalProviderSemanticContracts {
    pub(super) condition: Option<BridgeErasedProviderSemanticContract>,
    pub(super) dependency_comparator: Option<BridgeErasedProviderSemanticContract>,
    pub(super) output_comparator: Option<BridgeErasedProviderSemanticContract>,
    pub(super) reuse_comparator: Option<BridgeErasedProviderSemanticContract>,
    pub(super) trigger: Option<BridgeErasedProviderSemanticContract>,
    pub(super) wake: Option<BridgeErasedProviderSemanticContract>,
    pub(super) compute: Option<BridgeErasedProviderSemanticContract>,
}

impl BridgeConditionalProviderSemanticContracts {
    pub(super) fn retained_arc_bytes(
        &self,
    ) -> Result<u64, super::retention::BridgeRetentionDenial> {
        let contracts = [
            self.condition.as_ref(),
            self.dependency_comparator.as_ref(),
            self.output_comparator.as_ref(),
            self.reuse_comparator.as_ref(),
            self.trigger.as_ref(),
            self.wake.as_ref(),
            self.compute.as_ref(),
        ];
        contracts
            .into_iter()
            .flatten()
            .try_fold(0u64, |total, contract| {
                total
                    .checked_add(contract.retained_arc_bytes()?)
                    .ok_or(super::retention::BridgeRetentionDenial::BytesExhausted)
            })
    }
}
