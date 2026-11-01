//! Actual acquired Native admissions survive every later readmission refusal.
use worth_proof::RelationalPublicationReadmitted;
use worth_relational::facade::durability::{
    RecoveredRelationalBranchBasis, RelationalRecoveredPublicationCustody,
};

pub struct WorldNativeReadmissionCustody {
    pub(crate) root: Option<RecoveredRelationalBranchBasis>,
    pub(crate) publications:
        Vec<Option<RelationalPublicationReadmitted<RelationalRecoveredPublicationCustody>>>,
    pub(crate) records: Vec<std::sync::Arc<crate::history::ReadmittedPublicationRecord>>,
    pub(crate) history: Option<crate::history::CompositeHistoryCatalog>,
    pub(crate) branches: Option<crate::branch::registry::ProductBranchRegistry>,
    pub(crate) recovery: Option<crate::recovery::RecoveryCatalog>,
    pub(crate) observations: Vec<crate::branch::ProductBranchObservation>,
    pub(crate) slots: Vec<Option<crate::recovery::ReservedProductUnpublishedSlot>>,
    pub(crate) capacities: Vec<crate::history::ReservedCompositeCommitCapacity>,
}

impl std::fmt::Debug for WorldNativeReadmissionCustody {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorldNativeReadmissionCustody")
            .field("root_retained", &self.root.is_some())
            .field("publication_slots", &self.publications.len())
            .field("readmitted_records", &self.records.len())
            .field("history_retained", &self.history.is_some())
            .field("branches_retained", &self.branches.is_some())
            .field("recovery_retained", &self.recovery.is_some())
            .field("observation_slots", &self.observations.len())
            .field("recovery_slots", &self.slots.len())
            .field("capacity_slots", &self.capacities.len())
            .finish()
    }
}

impl WorldNativeReadmissionCustody {
    pub(crate) fn empty() -> Self {
        Self {
            root: None,
            publications: Vec::new(),
            records: Vec::new(),
            history: None,
            branches: None,
            recovery: None,
            observations: Vec::new(),
            slots: Vec::new(),
            capacities: Vec::new(),
        }
    }
    pub(crate) fn refuse(
        self,
        cause: super::WorldReadmissionDenial,
    ) -> super::WorldReadmissionDenial {
        super::WorldReadmissionDenial::NativeCustody {
            cause: Box::new(cause),
            custody: Box::new(self),
        }
    }

    pub fn retains_root_admission(&self) -> bool {
        self.root.is_some()
            || self
                .branches
                .as_ref()
                .is_some_and(|branches| branches.root_cell().is_some())
    }
    /// Actual private materialization retained by this refusal. These are
    /// descriptive owner observations, never a Serving World or Native result.
    pub fn retained_history_snapshot(
        &self,
    ) -> Option<crate::inspection::RuntimeWorldHistorySnapshot> {
        self.history.as_ref().map(|history| history.snapshot())
    }
    pub fn retained_product_head(&self) -> Option<crate::branch::ProductBranchReferenceSnapshot> {
        self.branches
            .as_ref()?
            .root_cell()
            .map(|cell| cell.atomic_snapshot())
    }
    pub fn retained_publication_admissions(&self) -> usize {
        self.records.len()
            + self
                .publications
                .iter()
                .filter(|publication| publication.is_some())
                .count()
    }
}
