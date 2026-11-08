//! Actual acquired Native admissions survive every later readmission refusal.
use worth_proof::RelationalPublicationReadmitted;
use worth_relational::facade::durability::{
    RecoveredRelationalBranchBasis, RelationalRecoveredPublicationCustody,
};

pub struct WorldNativeReadmissionCustody {
    pub(crate) root: Option<RecoveredRelationalBranchBasis>,
    pub(crate) publications:
        Vec<Option<RelationalPublicationReadmitted<RelationalRecoveredPublicationCustody>>>,
    pub(crate) occurrence_owners: Vec<Option<crate::history::RecoveredWorldOwnerEvidence>>,
    pub(crate) records: Vec<std::sync::Arc<crate::history::ReadmittedPublicationRecord>>,
    pub(crate) history: Option<crate::history::CompositeHistoryCatalog>,
    pub(crate) branches: Option<crate::branch::registry::ProductBranchRegistry>,
    pub(crate) recovery: Option<crate::recovery::RecoveryCatalog>,
    pub(crate) observations: Vec<crate::branch::ProductBranchObservation>,
    pub(crate) slots: Vec<Option<crate::recovery::ReservedProductUnpublishedSlot>>,
    pub(crate) capacities: Vec<crate::history::ReservedCompositeCommitCapacity>,
    pub(crate) checkpoint_origin: Option<super::WorldCheckpointOrigin>,
    pub(crate) checkpoint_occurrences: Vec<super::PreparedCheckpointOccurrence>,
    _metadata_funding: Option<worth_execution::ExecutionMemoryReservation>,
}

impl std::fmt::Debug for WorldNativeReadmissionCustody {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorldNativeReadmissionCustody")
            .field("root_retained", &self.root.is_some())
            .field("publication_slots", &self.publications.len())
            .field("occurrence_owner_slots", &self.occurrence_owners.len())
            .field("readmitted_records", &self.records.len())
            .field("history_retained", &self.history.is_some())
            .field("branches_retained", &self.branches.is_some())
            .field("recovery_retained", &self.recovery.is_some())
            .field("observation_slots", &self.observations.len())
            .field("recovery_slots", &self.slots.len())
            .field("capacity_slots", &self.capacities.len())
            .field(
                "checkpoint_origin_retained",
                &self.checkpoint_origin.is_some(),
            )
            .field("checkpoint_occurrences", &self.checkpoint_occurrences.len())
            .finish()
    }
}

impl WorldNativeReadmissionCustody {
    pub(crate) fn allocate(
        request: &worth_execution::ExecutionRequest<'_, '_>,
        occurrences: usize,
        opens: usize,
    ) -> Result<Self, super::WorldReadmissionDenial> {
        use super::{WorldDurableMediumDenial as Medium, WorldReadmissionDenial as Denial};
        let failed = || Denial::RecoveryCapacity;
        let publications = opens;
        let observations = occurrences.checked_add(1).ok_or_else(failed)?;
        let quote = [
            std::mem::size_of::<super::PreparedCheckpointOccurrence>().checked_mul(occurrences),
            std::mem::size_of::<Option<crate::history::RecoveredWorldOwnerEvidence>>()
                .checked_mul(occurrences),
            std::mem::size_of::<
                Option<RelationalPublicationReadmitted<RelationalRecoveredPublicationCustody>>,
            >()
            .checked_mul(publications),
            std::mem::size_of::<std::sync::Arc<crate::history::ReadmittedPublicationRecord>>()
                .checked_mul(occurrences),
            std::mem::size_of::<crate::branch::ProductBranchObservation>()
                .checked_mul(observations),
            std::mem::size_of::<Option<crate::recovery::ReservedProductUnpublishedSlot>>()
                .checked_mul(opens),
            std::mem::size_of::<crate::history::ReservedCompositeCommitCapacity>()
                .checked_mul(occurrences),
        ]
        .into_iter()
        .try_fold(0usize, |total, bytes| total.checked_add(bytes?))
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or_else(failed)?;
        let funding = request
            .reserve_retained_memory(quote)
            .map_err(|cause| Denial::Medium(Medium::Request(cause)))?;
        Ok(Self {
            root: None,
            publications: Vec::with_capacity(publications),
            occurrence_owners: Vec::with_capacity(occurrences),
            records: Vec::with_capacity(occurrences),
            history: None,
            branches: None,
            recovery: None,
            observations: Vec::with_capacity(observations),
            slots: Vec::with_capacity(opens),
            capacities: Vec::with_capacity(occurrences),
            checkpoint_origin: None,
            checkpoint_occurrences: Vec::with_capacity(occurrences),
            _metadata_funding: funding,
        })
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
                .occurrence_owners
                .iter()
                .filter(|owners| owners.is_some())
                .count()
            + self
                .publications
                .iter()
                .filter(|publication| publication.is_some())
                .count()
    }
}
