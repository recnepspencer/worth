use super::{
    PhysicalPublicationCounterSnapshot, PhysicalPublicationDenial,
    PhysicalPublicationPlanCompletion, PhysicalPublicationReadiness, RootSwapOrderingContract,
    ValidatedPhysicalPublicationIntent,
};

#[derive(Debug, Clone)]
pub struct LoweredCopyOnWritePublicationPlan {
    intent: ValidatedPhysicalPublicationIntent,
    ordering: RootSwapOrderingContract,
    counters: PhysicalPublicationCounterSnapshot,
}

#[derive(Debug, Clone)]
pub struct CopyOnWritePublicationPlan {
    intent: ValidatedPhysicalPublicationIntent,
    readiness: PhysicalPublicationReadiness,
    ordering: RootSwapOrderingContract,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CopyOnWritePublicationBinding {
    store_authority_identity: worth_store_authority::StoreCurrentAuthorityIdentity,
    old_root: crate::CurrentPhysicalRoot,
    new_root: crate::CurrentPhysicalRoot,
    old_root_validation: worth_store_physical_format::RootPublicationValidationWitness,
    new_root_validation: worth_store_physical_format::RootPublicationValidationWitness,
}

impl CopyOnWritePublicationBinding {
    fn from_intent(intent: &ValidatedPhysicalPublicationIntent) -> Self {
        Self {
            store_authority_identity: intent.old_root().store_authority_identity(),
            old_root: intent.old_root(),
            new_root: intent.new_root(),
            old_root_validation: intent.old_root_validation(),
            new_root_validation: intent.new_root_validation(),
        }
    }

    pub const fn store_authority_identity(
        self,
    ) -> worth_store_authority::StoreCurrentAuthorityIdentity {
        self.store_authority_identity
    }

    pub const fn old_root(self) -> crate::CurrentPhysicalRoot {
        self.old_root
    }

    pub const fn new_root(self) -> crate::CurrentPhysicalRoot {
        self.new_root
    }

    pub const fn old_root_validation(
        self,
    ) -> worth_store_physical_format::RootPublicationValidationWitness {
        self.old_root_validation
    }

    pub const fn new_root_validation(
        self,
    ) -> worth_store_physical_format::RootPublicationValidationWitness {
        self.new_root_validation
    }
}

impl ValidatedPhysicalPublicationIntent {
    pub fn lower_with_ordering(
        self,
        ordering: RootSwapOrderingContract,
    ) -> Result<LoweredCopyOnWritePublicationPlan, PhysicalPublicationDenial> {
        Ok(LoweredCopyOnWritePublicationPlan {
            intent: self,
            ordering,
            counters: PhysicalPublicationCounterSnapshot::for_validated_lowering(),
        })
    }
}

impl LoweredCopyOnWritePublicationPlan {
    pub fn join_readiness(
        self,
        readiness: PhysicalPublicationReadiness,
    ) -> Result<CopyOnWritePublicationPlan, PhysicalPublicationDenial> {
        let readiness = readiness.validate_for_intent(&self.intent)?;
        Ok(CopyOnWritePublicationPlan {
            ordering: self.ordering,
            intent: self.intent,
            readiness,
        })
    }

    pub const fn counters(&self) -> PhysicalPublicationCounterSnapshot {
        self.counters
    }
}

impl CopyOnWritePublicationPlan {
    pub fn binding(&self) -> CopyOnWritePublicationBinding {
        CopyOnWritePublicationBinding::from_intent(&self.intent)
    }

    pub const fn readiness(&self) -> PhysicalPublicationReadiness {
        self.readiness
    }

    pub const fn ordering(&self) -> RootSwapOrderingContract {
        self.ordering
    }

    /// Complete local publication planning. No root swap, I/O, durable
    /// publication, live retention, or recovery is performed by this call.
    ///
    /// ```
    /// use worth_store_physical_isolation::{
    ///     CopyOnWritePublicationPlan, PhysicalPublicationPlanCompletion,
    /// };
    /// fn finish(plan: CopyOnWritePublicationPlan) -> PhysicalPublicationPlanCompletion {
    ///     plan.complete_plan()
    /// }
    /// ```
    pub fn complete_plan(self) -> PhysicalPublicationPlanCompletion {
        PhysicalPublicationPlanCompletion::from_completed_plan(
            self.binding(),
            self.readiness,
            self.ordering,
        )
    }
}
