use crate::physical_runtime::durability::{
    join_dispatched_data, PhysicalDataEffectSettlement, PhysicalDataSettlementOutcome,
};
use crate::physical_runtime::{PhysicalMutationIdentity, WalDurablePhysicalMutation};

pub struct DataDispatchedPhysicalMutation {
    durable: WalDurablePhysicalMutation,
    effects: Vec<PhysicalDataEffectSettlement>,
}

impl DataDispatchedPhysicalMutation {
    pub(in crate::physical_runtime) fn from_source_copy(
        durable: WalDurablePhysicalMutation,
    ) -> Self {
        assert!(
            durable.source_copy().is_some(),
            "copy dispatch requires the sealed copy capability"
        );
        Self {
            durable,
            effects: Vec::new(),
        }
    }
    pub(in crate::physical_runtime) fn new(
        durable: WalDurablePhysicalMutation,
        effects: Vec<PhysicalDataEffectSettlement>,
    ) -> Self {
        Self { durable, effects }
    }

    pub const fn mutation_identity(&self) -> PhysicalMutationIdentity {
        self.durable.mutation_identity()
    }

    pub const fn durable(&self) -> &WalDurablePhysicalMutation {
        &self.durable
    }

    /// Individual WAL-publication frame writes. A source-copy adoption retains
    /// its separate, bounded copy evidence in the typed data plan instead.
    pub fn effects(&self) -> &[PhysicalDataEffectSettlement] {
        &self.effects
    }

    pub fn source_copy_evidence(
        &self,
    ) -> Option<crate::physical_runtime::PhysicalExtentCopySettlementObservation> {
        self.durable.source_copy().map(|(copy, range)| {
            crate::physical_runtime::PhysicalExtentCopySettlementObservation::from_capability(
                copy, range,
            )
        })
    }

    pub fn settle_exact_effects(self) -> PhysicalDataSettlementOutcome {
        join_dispatched_data(self)
    }

    pub(in crate::physical_runtime) fn into_parts(
        self,
    ) -> (
        WalDurablePhysicalMutation,
        Vec<PhysicalDataEffectSettlement>,
    ) {
        (self.durable, self.effects)
    }
}
