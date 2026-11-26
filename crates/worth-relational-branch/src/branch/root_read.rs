//! Operation-scoped reads of one immutable root (Store M4 D1).
//!
//! A [`RelationalBranchRoot`] retains only durable member custody. Every lease
//! a read admits lives in this value's residency, so the views it lends are
//! bounded by the operation that owns it: dropping it releases each borrow and
//! lets bound cells be evicted, however long the root itself stays open.
use super::{PartitionId, RelationalBranchRoot};
use crate::branch::projection_view::snapshot_head::SelectedSnapshotHead;
use crate::branch::symbol_read::SymbolAdmittedRootRead;
use crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable;
use crate::storage::data::{
    RelationalStorageSubject, RelationalStorageUnavailable, RelationalStorageUnavailableCause,
};
use crate::storage::overlay::PartitionState;
use crate::storage::residency::{
    Full, Head, OperationResidency, PartitionHeadProbe, PartitionProbe, RelationalRegionHeadView,
    RelationalResidencyDemandFailure as Failure, RelationalResidencyLease,
};
use worth_execution::ExecutionRequest;

#[derive(Debug)]
pub struct RelationalRootRead<'root> {
    root: &'root RelationalBranchRoot,
    residency: Residency<'root>,
}

/// This read's own scope, or the scope of an enclosing operation such as a
/// transaction, whose demands and later accepted reads must agree.
#[derive(Debug)]
enum Residency<'op> {
    Own(OperationResidency),
    Within(&'op OperationResidency),
}

impl RelationalBranchRoot {
    /// Begin one read operation over this root with an empty residency.
    pub fn read(&self) -> RelationalRootRead<'_> {
        RelationalRootRead {
            root: self,
            residency: Residency::Own(OperationResidency::new()),
        }
    }

    /// Accepted Head custody of `residency`, lent for that residency's borrow.
    pub fn accepted_head_region_view_within<'s>(
        &'s self,
        id: PartitionId,
        residency: &'s OperationResidency,
    ) -> Option<RelationalRegionHeadView<'s>> {
        self.regions.accepted_head_region_view(id, residency)
    }

    pub fn probe_full_region_within<'s>(
        &'s self,
        id: PartitionId,
        residency: &'s OperationResidency,
    ) -> PartitionProbe<'s> {
        self.regions.probe_full_region(id, residency)
    }

    /// Read within an enclosing operation's residency.
    pub fn read_within<'op>(
        &'op self,
        residency: &'op OperationResidency,
    ) -> RelationalRootRead<'op> {
        RelationalRootRead {
            root: self,
            residency: Residency::Within(residency),
        }
    }
}

impl std::ops::Deref for RelationalRootRead<'_> {
    type Target = RelationalBranchRoot;
    fn deref(&self) -> &RelationalBranchRoot {
        self.root
    }
}

impl<'root> RelationalRootRead<'root> {
    pub fn root(&self) -> &'root RelationalBranchRoot {
        self.root
    }

    pub fn residency(&self) -> &OperationResidency {
        match &self.residency {
            Residency::Own(scope) => scope,
            Residency::Within(scope) => scope,
        }
    }

    pub fn symbol_admitted_projection<'a>(
        &'a self,
        symbols: &'a RuntimeSymbolTable,
    ) -> SymbolAdmittedRootRead<'a> {
        SymbolAdmittedRootRead::new(&self.root.regions, symbols, self.residency())
    }

    pub fn project_snapshot(&self) -> super::projection::SnapshotProjectionRoot<'_> {
        self.root.project_snapshot_within(self.residency())
    }

    pub fn project_selected_head(&self, partition: PartitionId) -> SelectedSnapshotHead<'_> {
        self.root
            .regions
            .project_selected_head(partition, self.residency())
    }

    pub fn accepted_head_region_witness(
        &self,
        id: PartitionId,
    ) -> Result<Option<RelationalResidencyLease<Head>>, PartitionId> {
        self.root
            .regions
            .accepted_head_region_witness(id, self.residency())
    }

    pub fn resident_full_region_witness(
        &self,
        id: PartitionId,
    ) -> Result<Option<RelationalResidencyLease<Full>>, RelationalStorageUnavailable> {
        self.root
            .regions
            .get(id)
            .map(|region| region.partition.pin_lease(self.residency()))
            .transpose()
    }

    pub fn accepted_full_region(&self, id: PartitionId) -> Option<&PartitionState> {
        self.root.regions.accepted_full_region(id, self.residency())
    }

    pub fn accepted_head_region_view(
        &self,
        id: PartitionId,
    ) -> Option<RelationalRegionHeadView<'_>> {
        self.root
            .regions
            .accepted_head_region_view(id, self.residency())
    }

    pub fn probe_head_region(&self, id: PartitionId) -> PartitionHeadProbe<'_> {
        self.root.regions.probe_head_region(id, self.residency())
    }

    pub fn probe_full_region(&self, id: PartitionId) -> PartitionProbe<'_> {
        self.root.regions.probe_full_region(id, self.residency())
    }

    pub fn peek_head_region_with_symbols(
        &self,
        id: PartitionId,
        symbols: &RuntimeSymbolTable,
    ) -> PartitionHeadProbe<'_> {
        match self.root.regions.get(id) {
            None => PartitionHeadProbe::Absent,
            Some(region) => region
                .partition
                .peek_head_view_with_symbols(self.residency(), symbols)
                .map_or(PartitionHeadProbe::Cold, PartitionHeadProbe::Resident),
        }
    }

    pub fn peek_full_region_with_symbols(
        &self,
        id: PartitionId,
        symbols: &RuntimeSymbolTable,
    ) -> PartitionProbe<'_> {
        match self.root.regions.get(id) {
            None => PartitionProbe::Absent,
            Some(region) => region
                .partition
                .peek_full_partition_with_symbols(self.residency(), symbols)
                .map_or(PartitionProbe::Cold, PartitionProbe::Resident),
        }
    }

    pub fn demand_region_full(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        id: PartitionId,
    ) -> Result<Option<RelationalResidencyLease<Full>>, Failure> {
        self.root
            .regions
            .demand_region_full(execution, id, self.residency())
    }

    pub fn demand_region_head(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        id: PartitionId,
    ) -> Result<Option<RelationalResidencyLease<Head>>, Failure> {
        self.root
            .regions
            .demand_region_head(execution, id, self.residency())
    }

    pub fn demand_region_head_cached(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        id: PartitionId,
    ) -> Result<(), Failure> {
        self.root
            .regions
            .demand_region_head_cached(execution, id, self.residency())
    }

    pub fn demand_regions_head(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        symbols: &RuntimeSymbolTable,
        partitions: impl IntoIterator<Item = PartitionId>,
    ) -> Result<(), Failure> {
        self.root
            .regions
            .demand_regions_head(execution, symbols, partitions, self.residency())
    }

    pub fn demand_regions_full(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        symbols: &RuntimeSymbolTable,
        partitions: impl IntoIterator<Item = PartitionId>,
    ) -> Result<(), Failure> {
        self.root
            .regions
            .demand_regions_full(execution, symbols, partitions, self.residency())
    }

    /// Whole-root Full admission for operations that genuinely consume every
    /// region at once; the residency releases it when this read drops.
    pub fn demand_all_full(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        symbols: &RuntimeSymbolTable,
    ) -> Result<(), Failure> {
        self.demand_regions_full(execution, symbols, self.root.regions.partition_ids_iter())
    }

    /// A cold region refuses rather than reading as absent.
    pub fn partition_state(
        &self,
        id: PartitionId,
    ) -> Result<Option<&PartitionState>, RelationalStorageUnavailable> {
        let Some(region) = self.root.regions.get(id) else {
            return Ok(None);
        };
        region
            .partition
            .try_admitted_partition(self.residency())?
            .map(Some)
            .ok_or(RelationalStorageUnavailable {
                subject: RelationalStorageSubject::Region(id),
                cause: RelationalStorageUnavailableCause::SourceUnavailable,
            })
    }
}

/// One operation's selection of a root: reads through it admit into the
/// operation's residency, shared by its clones and never by the root.
#[derive(Debug, Clone)]
pub struct RelationalBranchRootState {
    pub(in crate::internal_port::branch) root: std::sync::Arc<RelationalBranchRoot>,
    pub(in crate::internal_port::branch) residency: std::sync::Arc<OperationResidency>,
}

impl RelationalBranchRootState {
    pub fn new(root: std::sync::Arc<RelationalBranchRoot>) -> Self {
        Self::within(root, Default::default())
    }

    pub fn within(
        root: std::sync::Arc<RelationalBranchRoot>,
        residency: std::sync::Arc<OperationResidency>,
    ) -> Self {
        Self { root, residency }
    }

    pub fn root(&self) -> &std::sync::Arc<RelationalBranchRoot> {
        &self.root
    }

    pub fn read(&self) -> RelationalRootRead<'_> {
        self.root.read_within(&self.residency)
    }
}

impl RelationalBranchRoot {
    /// Lend a restricted projection whose admissions live in `residency`.
    pub fn project_snapshot_within<'s>(
        &'s self,
        residency: &'s OperationResidency,
    ) -> super::projection::SnapshotProjectionRoot<'s> {
        use super::projection::{SnapshotProjectionRoot, SnapshotProjectionSchema};

        let (version, semantics, evidence) = self.query_schema_context();
        SnapshotProjectionRoot {
            regions: &self.regions,
            residency,
            id: self.id,
            storage_version: self
                .axes()
                .map(|axes| crate::identity::data::VersionId(axes.storage_version)),
            schema: SnapshotProjectionSchema {
                registry: self.schema_authority.registry(),
                plans: self.schema_authority.aspect_plans(),
                digest: self.schema_authority.authority_digest(),
                version,
                semantics,
                evidence,
            },
        }
    }

    pub fn project_selected_head_within<'s>(
        &'s self,
        partition: PartitionId,
        residency: &'s OperationResidency,
    ) -> SelectedSnapshotHead<'s> {
        self.regions.project_selected_head(partition, residency)
    }
}
