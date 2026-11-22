use worth_query_execution_runtime::facade::ports::ProductBranchLeaseInternalPort as _;
use worth_relational::facade::snapshots::SnapshotHandle;

use super::super::WorthQueryPrimaryGraphIntegrationHandle;
use crate::ports::BasisLeasePort;
use worth_query_execution_runtime::facade::basis_identities::WorthQueryApplicationBasisReleaseOutcome;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationSnapshotLeaseDenial {
    ForeignRuntime,
    ActiveSnapshotCapacityExhausted { maximum_active_snapshots: usize },
    SnapshotIdentityExhausted,
}

pub use worth_query_execution_runtime::facade::basis_identities::WorthQueryApplicationSnapshotRelease;

enum SnapshotCustody {
    Owned {
        handle: WorthQueryPrimaryGraphIntegrationHandle,
        snapshot: SnapshotHandle,
    },
    Shared(Box<dyn BasisLeasePort>),
    Original(std::sync::Arc<SharedOriginalSnapshot>),
}

struct SharedOriginalSnapshot {
    custody: Option<SnapshotCustody>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl Drop for SharedOriginalSnapshot {
    fn drop(&mut self) {
        if let Some(custody) = self.custody.take() {
            release_custody(custody);
        }
    }
}
impl SnapshotCustody {
    fn snapshot(&self) -> &SnapshotHandle {
        match self {
            Self::Owned { snapshot, .. } => snapshot,
            Self::Shared(basis) => basis.snapshot_handle(),
            Self::Original(shared) => shared
                .custody
                .as_ref()
                .expect("live original pin")
                .snapshot(),
        }
    }
    fn handle(&self) -> &WorthQueryPrimaryGraphIntegrationHandle {
        match self {
            Self::Owned { handle, .. } => handle,
            Self::Shared(basis) => basis.graph_handle(),
            Self::Original(shared) => shared.custody.as_ref().expect("live original pin").handle(),
        }
    }
}

pub struct WorthQueryApplicationSnapshotLease {
    applied_source_qualification:
        Option<worth_relational::facade::runtime::RelationalAppliedCandidateReadQualification>,
    applied_projection: Option<
        worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
    >,
    custody: Option<SnapshotCustody>,
    product: crate::basis::WorthQueryProductBranchLease,
    selection_origin: super::super::product_operation::SelectedQueryOrigin,
    pub layout: std::sync::Arc<super::super::schema_layout::WorthQueryPrimaryGraphLayout>,
    /// A successor round's own-output view (§1.5) over the applied image.
    own_outputs: Option<crate::domain_computation::primary_graph::invariant_projection::read_basis::OwnOutputOverlay>,
}

impl WorthQueryApplicationSnapshotLease {
    /// The snapshot remains the genuine before authorization lease. This
    /// separately retained image supplies applied decision data and no Current receipt.
    pub fn with_applied_projection(
        mut self,
        pin: worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
    ) -> Self {
        self.retain_applied_projection(pin);
        self
    }
    /// Retain genuine applied decision data without changing the authorization lease.
    /// This does not supply an applied Source qualification or Current authority.
    pub fn retain_applied_projection(
        &mut self,
        pin: worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
    ) {
        self.applied_projection = Some(pin);
    }
    pub fn bind_applied_source_projection(
        &mut self,
        pin: worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
    ) {
        self.applied_source_qualification = Some(pin.candidate_identity().qualification());
        self.applied_projection = Some(pin);
    }
    pub fn applied_source_qualification(
        &self,
    ) -> Option<worth_relational::facade::runtime::RelationalAppliedCandidateReadQualification>
    {
        self.applied_source_qualification
    }
    /// Observe a successor's own chain records as before the chain (§1.5).
    pub fn retain_own_outputs(
        &mut self,
        own_outputs: Option<&crate::domain_computation::primary_graph::invariant_projection::read_basis::OwnOutputOverlay>,
    ) {
        self.own_outputs = own_outputs.cloned();
    }
    /// Observe `key` at the applied image, when the lease has one. Only a
    /// field read is served by a successor's own-output view; a covered
    /// record absent there was created by the chain, and the view is
    /// incoherent (#277).
    pub fn observe_applied(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        key: &super::WorthQueryApplicationFactKey,
    ) -> Option<Result<super::WorthQueryApplicationObservedFact, super::WorthQueryApplicationAttemptDenial>>
    {
        let entity = match key {
            super::WorthQueryApplicationFactKey::Field { entity_id, .. } => Some(*entity_id),
            _ => None,
        };
        let pin = self.applied_projection_of(entity)?;
        let observed =
            super::read_set::fact_observation::observe_applied_fact(execution, &self.layout, key, pin);
        // Incoherent only when the entity itself is absent at the head pre-pin,
        // never for an absent aspect or field of an entity that exists there.
        if let (Err(denial), Some(entity), Some(own)) = (&observed, entity, &self.own_outputs)
            && denial.kind() == super::WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact
            && own.covers(entity)
            && let super::WorthQueryApplicationFactKey::Field { entity: name, .. } = key
        {
            let presence = super::WorthQueryApplicationFactKey::Entity {
                entity: name.clone(),
                entity_id: entity,
            };
            if super::read_set::fact_observation::observe_applied_fact(execution, &self.layout, &presence, pin)
                .is_err_and(|denial| {
                    denial.kind() == super::WorthQueryApplicationAttemptDenialKind::MissingAuthoritativeFact
                        && denial.relational_execution_cause().is_none()
                })
            {
                own.mark_incoherent(entity);
            }
        }
        Some(observed)
    }
    /// The applied image a single-record fact of `entity` is observed at.
    pub fn applied_projection_of(
        &self,
        entity: Option<worth_relational::facade::identity::EntityId>,
    ) -> Option<
        &worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
    > {
        entity
            .and_then(|entity| self.own_outputs.as_ref()?.head_for(entity))
            .or(self.applied_projection.as_ref())
    }
    pub fn applied_projection(
        &self,
    ) -> Option<
        &worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
    > {
        self.applied_projection.as_ref()
    }
    pub fn projection_origin(
        &self,
    ) -> super::super::projection_read_origin::ProjectionReadOriginRef<'_> {
        match self.applied_qualification() {
            Some(candidate) => {
                super::super::projection_read_origin::ProjectionReadOriginRef::Applied(candidate)
            }
            None => self.snapshot().into(),
        }
    }

    pub fn applied_qualification(
        &self,
    ) -> Option<worth_relational::facade::runtime::RelationalAppliedCandidateReadQualification>
    {
        self.applied_projection
            .as_ref()
            .map(|pin| pin.candidate_identity().qualification())
    }

    pub fn share_original(
        mut self,
        control: super::retained_decision_facts::StorageControl<'_, '_>,
    ) -> Result<(Self, Self), super::retained_decision_facts::StoreDenial> {
        let memory = control
            .policy()
            .reserve_memory((size_of::<SharedOriginalSnapshot>() + 2 * size_of::<usize>()) as u64);
        control.check_live()?;
        let memory = memory?;
        let custody = self.custody.take().expect("live snapshot custody");
        let shared = match custody {
            SnapshotCustody::Original(shared) => shared,
            custody => std::sync::Arc::new(SharedOriginalSnapshot {
                custody: Some(custody),
                _memory: memory,
            }),
        };
        self.custody = Some(SnapshotCustody::Original(shared.clone()));
        let retained = Self {
            applied_source_qualification: self.applied_source_qualification,
            applied_projection: self.applied_projection.clone(),
            own_outputs: self.own_outputs.clone(),
            custody: Some(SnapshotCustody::Original(shared)),
            product: self.product.retained_clone(),
            selection_origin: self.selection_origin,
            layout: self.layout.clone(),
        };
        Ok((self, retained))
    }
    pub fn acquire(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        handle: WorthQueryPrimaryGraphIntegrationHandle,
        layout: std::sync::Arc<super::super::schema_layout::WorthQueryPrimaryGraphLayout>,
        product: crate::basis::WorthQueryProductBranchLease,
    ) -> Result<Self, WorthQueryApplicationSnapshotLeaseDenial> {
        let basis = product.relational_basis().clone();
        let snapshot = handle.with_runtime(execution, |runtime| {
            let snapshot = runtime
                .snapshots()
                .snapshot_for_observation(&basis.observation())
                .map_err(|denial| match denial {
                    worth_relational::facade::snapshots::RelationalSnapshotAdmissionDenial::ForeignRuntime { .. } => {
                        WorthQueryApplicationSnapshotLeaseDenial::ForeignRuntime
                    }
                    worth_relational::facade::snapshots::RelationalSnapshotAdmissionDenial::ActiveSnapshotCapacityExhausted {
                        maximum_active_snapshots,
                    } => WorthQueryApplicationSnapshotLeaseDenial::ActiveSnapshotCapacityExhausted {
                        maximum_active_snapshots,
                    },
                    worth_relational::facade::snapshots::RelationalSnapshotAdmissionDenial::SnapshotIdentityExhausted => {
                        WorthQueryApplicationSnapshotLeaseDenial::SnapshotIdentityExhausted
                    }
                })?;
            Ok(snapshot)
        })?;
        Ok(Self {
            applied_source_qualification: None,
            applied_projection: None,
            own_outputs: None,
            custody: Some(SnapshotCustody::Owned { handle, snapshot }),
            product,
            selection_origin: Default::default(),
            layout,
        })
    }

    pub fn from_existing(
        handle: WorthQueryPrimaryGraphIntegrationHandle,
        layout: std::sync::Arc<super::super::schema_layout::WorthQueryPrimaryGraphLayout>,
        basis: worth_relational::facade::branch::AdmittedRelationalBranchBasis,
        snapshot: SnapshotHandle,
        product: crate::basis::WorthQueryProductBranchLease,
    ) -> Self {
        assert_eq!(
            basis.observation().version_id(),
            snapshot.version_id(),
            "existing application snapshot must select its carried owner basis"
        );
        assert_eq!(
            basis.identity().branch_id(),
            snapshot.branch_id(),
            "existing application snapshot and carried basis must share a branch"
        );
        assert_eq!(
            basis.descriptor(),
            product.relational_basis().descriptor(),
            "existing application snapshot must carry the admitted composite occurrence"
        );
        Self {
            applied_source_qualification: None,
            applied_projection: None,
            own_outputs: None,
            custody: Some(SnapshotCustody::Owned { handle, snapshot }),
            product,
            selection_origin: Default::default(),
            layout,
        }
    }

    /// Shares the wave's final Native snapshot owner without registering or
    /// releasing another Native snapshot. The session owns this local share.
    pub fn from_shared_basis(
        basis: Box<dyn BasisLeasePort>,
        product: crate::basis::WorthQueryProductBranchLease,
        selection_origin: super::super::product_operation::SelectedQueryOrigin,
        layout: std::sync::Arc<super::super::schema_layout::WorthQueryPrimaryGraphLayout>,
    ) -> Self {
        Self {
            applied_source_qualification: None,
            applied_projection: None,
            own_outputs: None,
            custody: Some(SnapshotCustody::Shared(basis)),
            product,
            selection_origin,
            layout,
        }
    }

    pub fn set_selection_origin(
        &mut self,
        selected_product: &crate::basis::WorthQueryProductBranchLease,
        selection_origin: super::super::product_operation::SelectedQueryOrigin,
    ) -> bool {
        if !self
            .product
            .shares_retained_mutation_source(selected_product)
        {
            return false;
        }
        self.selection_origin = selection_origin;
        true
    }

    pub fn selection_origin(&self) -> super::super::product_operation::SelectedQueryOrigin {
        self.selection_origin
    }

    pub fn snapshot(&self) -> &SnapshotHandle {
        self.custody
            .as_ref()
            .expect("snapshot lease retains custody")
            .snapshot()
    }

    pub fn product(&self) -> &crate::basis::WorthQueryProductBranchLease {
        &self.product
    }

    pub fn handle(&self) -> &WorthQueryPrimaryGraphIntegrationHandle {
        self.custody
            .as_ref()
            .expect("snapshot lease retains custody")
            .handle()
    }

    pub fn release(self) -> bool {
        self.release_custody().snapshot_released()
    }

    pub fn release_custody(mut self) -> WorthQueryApplicationSnapshotRelease {
        release_custody(self.custody.take().expect("snapshot lease releases once"))
    }
}

impl Drop for WorthQueryApplicationSnapshotLease {
    fn drop(&mut self) {
        if let Some(custody) = self.custody.take() {
            release_custody(custody);
        }
    }
}

fn release_custody(custody: SnapshotCustody) -> WorthQueryApplicationSnapshotRelease {
    match custody {
        SnapshotCustody::Owned { handle, snapshot } => {
            handle.release_query_snapshot(&snapshot);
            WorthQueryApplicationSnapshotRelease::NativeSnapshotReleased
        }
        SnapshotCustody::Shared(basis) => {
            WorthQueryApplicationSnapshotRelease::SharedBasisRelease(basis.release_basis())
        }
        SnapshotCustody::Original(shared) => match std::sync::Arc::try_unwrap(shared) {
            Ok(mut owner) => release_custody(owner.custody.take().expect("live original pin")),
            Err(shared) => {
                drop(shared);
                WorthQueryApplicationSnapshotRelease::SharedBasisRelease(
                    WorthQueryApplicationBasisReleaseOutcome::SharedCustodyRetained,
                )
            }
        },
    }
}

mod court_doors_for_test;
