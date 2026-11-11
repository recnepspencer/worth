mod cell_custody;
use super::*;
use cell_custody::RegionLeaseCustody;
use std::marker::PhantomData;
use std::sync::Arc;

#[derive(Debug)]
pub(crate) enum Head {}
#[derive(Debug)]
pub(crate) enum Full {}
/// Canonical full contents for encoding, without a root's serving indexes.
#[derive(Debug)]
pub(super) enum CheckpointContents {}

/// Clones and releases are counted on the cell, including when an operation
/// retains a view after the root that supplied it has been released.
#[derive(Debug)]
pub(crate) struct RelationalResidencyLease<Level> {
    custody: RegionLeaseCustody,
    region: Arc<ResidentRegion>,
    level: PhantomData<Level>,
}

impl<Level> RelationalResidencyLease<Level> {
    pub(super) fn retire_native(&self) {
        self.custody.retire_native();
    }

    pub(super) fn into_owned_contents(self) -> crate::storage::overlay::PartitionState {
        let Self {
            custody, region, ..
        } = self;
        drop(custody);
        match Arc::try_unwrap(region) {
            Ok(region) => region.partition,
            Err(region) => region.partition.clone(),
        }
    }

    pub(super) fn resident_region(&self) -> &ResidentRegion {
        &self.region
    }
    pub(crate) fn has_complete_history(&self) -> bool {
        self.region.history_presence.is_complete()
    }
    /// A shared cell authenticates bytes once; each demanding runtime admits
    /// their dictionary closure into its own independently writable authority.
    fn admit_symbols_at(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        symbols: &crate::runtime::RuntimeSymbolTable,
        identity: super::RelationalRegionSymbolAdmissionIdentity,
        scope: SymbolScope,
    ) -> Result<(), RelationalResidencyDemandFailure> {
        let inheritance = self.cell().symbol_inheritance.get();
        let certificate = inheritance.and_then(|basis| basis.certificate.as_ref());
        if let Some(certificate) = certificate
            && (!matches!(scope, SymbolScope::Full) || certificate.full_at_birth)
            && symbols.admit_region_from_publication(
                execution,
                identity.clone(),
                &certificate.publication,
            )?
        {
            return Ok(());
        }
        // A mirror's projected delta is relative to the canonical root, not
        // its independently retained history. Only the accepted scope covers it.
        let predecessor = certificate
            .is_none()
            .then(|| {
                inheritance.and_then(|basis| match scope {
                    SymbolScope::Owned => Some(basis.owned_predecessor()),
                    SymbolScope::Head => basis.predecessor(false),
                    SymbolScope::Full => basis.predecessor(true),
                })
            })
            .flatten();
        symbols.demand_region_symbols_incremental(
            execution,
            identity,
            predecessor.map(|(id, delta)| (id, &delta.ids, delta.visits)),
            || {
                let required = self.region.symbol_requirements.get().and_then(|captured| match scope { SymbolScope::Owned => captured.owned(), SymbolScope::Head => captured.get(false), SymbolScope::Full => captured.get(true) }).ok_or_else(|| {
                    super::RelationalResidencyDemandFailure::StorageUnavailable(crate::storage::data::RelationalStorageUnavailable {
                        subject: crate::storage::data::RelationalStorageSubject::Region(self.region.partition.partition_id),
                        cause: crate::storage::data::RelationalStorageUnavailableCause::MissingSymbolMaintenancePredecessor,
                    })
                })?;
                Ok((required.ids.clone(), required.ids.len() as u64))
            },
        )
    }

    pub(crate) fn head_symbol_admission_identity(
        &self,
    ) -> super::RelationalRegionSymbolAdmissionIdentity {
        super::RelationalRegionSymbolAdmissionIdentity::head(self.cell())
    }

    pub(super) fn accepted(
        cell: Arc<RelationalRegionResidency>,
        region: Arc<ResidentRegion>,
    ) -> Self {
        Self {
            custody: RegionLeaseCustody::accepted(cell),
            region,
            level: PhantomData,
        }
    }

    pub(crate) fn cell(&self) -> &Arc<RelationalRegionResidency> {
        self.custody.cell()
    }

    pub(crate) fn head_view(&self) -> RelationalRegionHeadView<'_> {
        RelationalRegionHeadView::new(&self.region)
    }
}

#[derive(Clone, Copy)]
enum SymbolScope {
    Head,
    Owned,
    Full,
}
impl RelationalResidencyLease<Head> {
    pub(crate) fn admit_owned_symbols(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        symbols: &crate::runtime::RuntimeSymbolTable,
    ) -> Result<(), RelationalResidencyDemandFailure> {
        self.admit_symbols_at(
            execution,
            symbols,
            super::RelationalRegionSymbolAdmissionIdentity::owned(self.cell()),
            SymbolScope::Owned,
        )
    }
    pub(crate) fn symbol_admission_identity(
        &self,
    ) -> super::RelationalRegionSymbolAdmissionIdentity {
        self.head_symbol_admission_identity()
    }
    pub(crate) fn admit_symbols(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        symbols: &crate::runtime::RuntimeSymbolTable,
    ) -> Result<(), RelationalResidencyDemandFailure> {
        self.admit_symbols_at(
            execution,
            symbols,
            self.symbol_admission_identity(),
            SymbolScope::Head,
        )
    }
}

impl RelationalResidencyLease<Full> {
    pub(crate) fn as_head(&self) -> RelationalResidencyLease<Head> {
        RelationalResidencyLease {
            custody: self.custody.clone(),
            region: self.region.clone(),
            level: PhantomData,
        }
    }
    pub(crate) fn symbol_admission_identity(
        &self,
    ) -> super::RelationalRegionSymbolAdmissionIdentity {
        super::RelationalRegionSymbolAdmissionIdentity::full(self.cell())
    }
    pub(crate) fn admit_symbols(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        symbols: &crate::runtime::RuntimeSymbolTable,
    ) -> Result<(), RelationalResidencyDemandFailure> {
        self.admit_symbols_at(
            execution,
            symbols,
            self.symbol_admission_identity(),
            SymbolScope::Full,
        )
    }

    /// Accepted Full custody supplies all rows. This never admits or loads.
    pub(crate) fn into_owned_partition(self) -> crate::storage::overlay::PartitionState {
        self.into_owned_contents()
    }

    #[cfg(test)]
    pub(super) fn retained_region(&self) -> Arc<ResidentRegion> {
        self.region.clone()
    }
    pub(crate) fn full_view(&self) -> RelationalRegionFullView<'_> {
        RelationalRegionFullView::new(&self.region)
    }
}

impl RelationalResidencyLease<CheckpointContents> {
    pub(super) fn checkpoint_contents(&self) -> &crate::storage::overlay::PartitionState {
        &self.region.partition
    }
}

impl AsRef<crate::storage::overlay::PartitionState> for RelationalResidencyLease<Full> {
    fn as_ref(&self) -> &crate::storage::overlay::PartitionState {
        self.full_view().partition()
    }
}

impl std::ops::Deref for RelationalResidencyLease<Full> {
    type Target = crate::storage::overlay::PartitionState;
    fn deref(&self) -> &Self::Target {
        self.full_view().partition()
    }
}

impl<Level> Clone for RelationalResidencyLease<Level> {
    fn clone(&self) -> Self {
        Self {
            custody: self.custody.clone(),
            region: self.region.clone(),
            level: PhantomData,
        }
    }
}
