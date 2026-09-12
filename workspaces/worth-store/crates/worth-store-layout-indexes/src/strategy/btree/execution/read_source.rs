use super::{
    BaselineBTreeExecutionDenial, BaselineBTreeExecutionWitness, BaselineBTreeLookupAdmission,
    BaselineBTreeReadShape, StableBTreeLookupExecution, StableReadBindings,
};
use std::sync::atomic::{AtomicU64, Ordering};
use worth_store_physical_format::PhysicalRecordSlot;
use worth_store_physical_isolation::{
    CompactionProtectedReferenceSet, CurrentGenerationPhysicalReference,
    PhysicalReadPlanAdmissionDenial, StablePhysicalReadPlan,
};

static NEXT_BTREE_READ_SOURCE_IDENTITY: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaselineBTreeReadSourceReceipt {
    identity: u64,
    root: worth_store_physical_format::PhysicalReference,
    store_authority: worth_store_authority::StoreCurrentAuthorityIdentity,
}

impl BaselineBTreeReadSourceReceipt {
    fn issue(witness: &BaselineBTreeExecutionWitness) -> Self {
        let identity = NEXT_BTREE_READ_SOURCE_IDENTITY
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                current.checked_add(1)
            })
            .expect("B-tree read-source identity space exhausted");
        Self {
            identity,
            root: witness.root_reference(),
            store_authority: witness.store_authority_identity(),
        }
    }

    pub const fn root_reference(&self) -> worth_store_physical_format::PhysicalReference {
        self.root
    }

    pub const fn store_authority_identity(
        &self,
    ) -> worth_store_authority::StoreCurrentAuthorityIdentity {
        self.store_authority
    }
}

#[derive(Debug)]
pub struct BaselineBTreeReadPreflight {
    witness: BaselineBTreeExecutionWitness,
    references: [CurrentGenerationPhysicalReference; 3],
}

impl BaselineBTreeReadPreflight {
    pub(super) fn from_published_layout(
        witness: BaselineBTreeExecutionWitness,
    ) -> Result<Self, BaselineBTreeExecutionDenial> {
        let references = witness.stable_read_references()?;
        Ok(Self {
            witness,
            references,
        })
    }

    pub const fn protected_references(&self) -> [CurrentGenerationPhysicalReference; 3] {
        self.references
    }

    pub fn admit(
        self,
        plan: StablePhysicalReadPlan,
    ) -> Result<BaselineBTreeReadSource, BaselineBTreeExecutionDenial> {
        if plan.root().store_authority_identity() != self.witness.store_authority_identity() {
            return Err(BaselineBTreeExecutionDenial::StableReadPlan(
                PhysicalReadPlanAdmissionDenial::StoreAuthorityMismatch,
            ));
        }
        for reference in self.references {
            if !plan.footprint().admits_reference(reference) {
                return Err(BaselineBTreeExecutionDenial::StableReadPlan(
                    PhysicalReadPlanAdmissionDenial::ExecutionTimeReferenceDiscovery,
                ));
            }
        }
        Ok(BaselineBTreeReadSource {
            receipt: BaselineBTreeReadSourceReceipt::issue(&self.witness),
            witness: self.witness,
            protected: CompactionProtectedReferenceSet::from_read_plan(&plan),
            references: self.references,
            plan,
        })
    }
}

#[derive(Debug)]
pub struct BaselineBTreeReadSource {
    receipt: BaselineBTreeReadSourceReceipt,
    witness: BaselineBTreeExecutionWitness,
    plan: StablePhysicalReadPlan,
    protected: CompactionProtectedReferenceSet,
    references: [CurrentGenerationPhysicalReference; 3],
}

impl BaselineBTreeReadSource {
    pub const fn receipt(&self) -> &BaselineBTreeReadSourceReceipt {
        &self.receipt
    }

    pub const fn root_reference(&self) -> worth_store_physical_format::PhysicalReference {
        self.witness.root_reference()
    }

    pub fn store_authority_identity(&self) -> worth_store_authority::StoreCurrentAuthorityIdentity {
        self.receipt.store_authority_identity()
    }

    pub(crate) fn execute(
        self,
        admission: &BaselineBTreeLookupAdmission,
        probe_slot: PhysicalRecordSlot,
        shape: BaselineBTreeReadShape,
    ) -> Result<StableBTreeLookupExecution, BaselineBTreeExecutionDenial> {
        let handle = self.plan.into_execution_ready_handle();
        for reference in self.references {
            handle
                .read_protected_reference(reference)
                .map_err(BaselineBTreeExecutionDenial::StableReadPlan)?;
        }
        let observation = self
            .witness
            .execute_separator_directed_read(probe_slot, shape);
        let stable_read = handle.complete_plan();
        observation.and_then(|observation| {
            StableBTreeLookupExecution::issue(
                observation,
                admission.plan_binding(),
                StableReadBindings {
                    receipt: stable_read,
                    protected: self.protected,
                },
                admission.current_materialization().clone(),
            )
            .map_err(BaselineBTreeExecutionDenial::CounterEnvelope)
        })
    }
}
