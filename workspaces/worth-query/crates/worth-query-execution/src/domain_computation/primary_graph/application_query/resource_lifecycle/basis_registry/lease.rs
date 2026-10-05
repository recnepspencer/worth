use super::super::lifecycle_count::release;
use super::{
    WorthQueryApplicationBasisIdentity, WorthQueryApplicationBasisRegistryState,
    WorthQueryApplicationBasisReleaseOutcome, WorthQueryApplicationBasisReleaseReceipt,
    WorthQueryApplicationBasisSelectionIdentity,
};
use crate::domain_computation::primary_graph::{
    output_lineage::invalidation::InvalidationEditAdmission,
    program_occurrence::WorthQueryProgramSupportInterpretation,
    WorthQueryPrimaryGraphIntegrationHandle,
};
use std::sync::Arc;
use worth_relational::facade::{
    branch::{AdmittedRelationalBranchBasis, RelationalBranchRetentionLease},
    mvcc::CompanionPreflightStop,
    snapshots::SnapshotHandle,
};

pub(crate) struct WorthQueryApplicationBasisLease {
    identity: WorthQueryApplicationBasisIdentity,
    custody: BasisCustody,
}

enum BasisCustody {
    Exclusive(BasisLeaseCore),
    Shared(Arc<BasisLeaseCore>),
    Transitioning,
}

struct BasisLeaseCore {
    basis: Option<AdmittedRelationalBranchBasis>,
    retention: Option<RelationalBranchRetentionLease>,
    snapshot: Option<SnapshotHandle>,
    graph: WorthQueryPrimaryGraphIntegrationHandle,
    state: Arc<WorthQueryApplicationBasisRegistryState>,
    program_interpretation: Option<WorthQueryProgramSupportInterpretation>,
    selected_program_inspected: bool,
}

impl WorthQueryApplicationBasisLease {
    pub(super) fn registered(
        identity: WorthQueryApplicationBasisIdentity,
        basis: AdmittedRelationalBranchBasis,
        retention: RelationalBranchRetentionLease,
        snapshot: SnapshotHandle,
        graph: WorthQueryPrimaryGraphIntegrationHandle,
        state: Arc<WorthQueryApplicationBasisRegistryState>,
    ) -> Self {
        Self {
            identity,
            custody: BasisCustody::Exclusive(BasisLeaseCore {
                basis: Some(basis),
                retention: Some(retention),
                snapshot: Some(snapshot),
                graph,
                state,
                program_interpretation: None,
                selected_program_inspected: false,
            }),
        }
    }

    fn core(&self) -> &BasisLeaseCore {
        match &self.custody {
            BasisCustody::Exclusive(core) => core,
            BasisCustody::Shared(core) => core,
            BasisCustody::Transitioning => {
                unreachable!("custody promotion cannot escape its owner")
            }
        }
    }

    fn core_mut(&mut self) -> &mut BasisLeaseCore {
        match &mut self.custody {
            BasisCustody::Exclusive(core) => core,
            BasisCustody::Shared(core) => {
                Arc::get_mut(core).expect("program binding precedes sharing")
            }
            BasisCustody::Transitioning => {
                unreachable!("custody promotion cannot escape its owner")
            }
        }
    }

    pub(in crate::domain_computation::primary_graph) fn prepare_shared(
        &mut self,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), CompanionPreflightStop> {
        if matches!(self.custody, BasisCustody::Shared(_)) {
            return Ok(());
        }
        admission.charge_external_work(1)?;
        let bytes =
            crate::domain_computation::primary_graph::output_lineage::invalidation::arc_bytes::<
                BasisLeaseCore,
            >()
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        admission.admit_read_scratch(bytes)?;
        let BasisCustody::Exclusive(core) =
            std::mem::replace(&mut self.custody, BasisCustody::Transitioning)
        else {
            unreachable!("only exclusive custody can enter the sharing phase")
        };
        self.custody = BasisCustody::Shared(Arc::new(core));
        Ok(())
    }

    pub(in crate::domain_computation::primary_graph) fn retain_shared(
        &self,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Self, CompanionPreflightStop> {
        let BasisCustody::Shared(core) = &self.custody else {
            return Err(CompanionPreflightStop::SelectedSourceMismatch);
        };
        let descriptor = &self.identity.descriptor;
        let parents = descriptor
            .reference()
            .target()
            .as_basis()
            .map_or(0, |target| target.parent_commit_ids().len());
        let strings = self
            .identity
            .branch_id
            .0
            .len()
            .checked_add(descriptor.branch_id().0.len())
            .and_then(|bytes| bytes.checked_add(descriptor.reference().branch_id().as_str().len()))
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        let bytes = parents
            .checked_mul(std::mem::size_of::<u64>())
            .and_then(|bytes| bytes.checked_add(strings))
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(CompanionPreflightStop::PreparationMemoryCounterOverflow)?;
        let work = u64::try_from(strings)
            .ok()
            .and_then(|work| work.checked_add(parents as u64))
            .and_then(|work| work.checked_add(7))
            .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
        admission.charge_external_work(work)?;
        admission.admit_read_scratch(bytes)?;
        Ok(Self {
            identity: self.identity.clone(),
            custody: BasisCustody::Shared(Arc::clone(core)),
        })
    }
}

impl WorthQueryApplicationBasisLease {
    /// The program inspection was performed for this lease's exact native
    /// basis, including the case where the installed program is unavailable.
    pub(in crate::domain_computation::primary_graph) fn mark_selected_program_inspected(
        &mut self,
        relational: &AdmittedRelationalBranchBasis,
    ) -> bool {
        if self.identity.runtime_instance_id != relational.identity().runtime_instance_id()
            || &self.identity.branch_id != relational.identity().branch_id()
            || &self.identity.descriptor != relational.descriptor()
            || self.version_id() != relational.observation().version_id()
        {
            return false;
        }
        self.core_mut().selected_program_inspected = true;
        true
    }

    pub(in crate::domain_computation::primary_graph) fn selected_program_inspected(&self) -> bool {
        self.core().selected_program_inspected
    }

    pub(in crate::domain_computation::primary_graph) fn carries_selected_program_interpretation(
        &self,
        selected: Option<&crate::domain_computation::primary_graph::program_occurrence::WorthQueryProgramSupportInterpretation>,
    ) -> bool {
        match (self.core().program_interpretation.as_ref(), selected) {
            (Some(retained), Some(current)) => retained.same_support_as(current),
            (None, None) => true,
            _ => false,
        }
    }

    pub(crate) fn bind_product_observation(
        &mut self,
        product: &worth_runtime_world::facade::ProductBranchObservation,
    ) {
        assert_eq!(
            self.identity.descriptor(),
            product.basis().relational_basis().descriptor(),
            "product custody retains the exact query snapshot basis"
        );
        self.identity.selection = WorthQueryApplicationBasisSelectionIdentity::Product(
            crate::basis::WorthQueryProductBranchReadIdentity::from_observation(product),
        );
    }

    pub(in crate::domain_computation::primary_graph) fn bind_program_interpretation(
        &mut self,
        interpretation: crate::domain_computation::primary_graph::program_occurrence::WorthQueryProgramSupportInterpretation,
    ) {
        assert!(
            self.core_mut()
                .program_interpretation
                .replace(interpretation)
                .is_none(),
            "one retained basis binds exactly one program interpretation"
        );
    }

    pub fn identity(&self) -> &WorthQueryApplicationBasisIdentity {
        &self.identity
    }

    pub fn version_id(&self) -> worth_relational::facade::identity::VersionId {
        self.basis().observation().version_id()
    }

    pub fn snapshot_handle(&self) -> &SnapshotHandle {
        self.core()
            .snapshot
            .as_ref()
            .expect("an active application-query basis retains its snapshot")
    }

    pub(in crate::domain_computation) fn graph_handle(
        &self,
    ) -> &WorthQueryPrimaryGraphIntegrationHandle {
        &self.core().graph
    }

    pub fn is_live(&self) -> bool {
        self.core().basis.is_some()
            && self.core().snapshot.as_ref().is_some_and(|snapshot| {
                self.core().graph.with_runtime(|runtime| {
                    runtime.read_truth().project_snapshot(snapshot).is_some()
                })
            })
    }

    pub fn release(self) -> WorthQueryApplicationBasisReleaseReceipt {
        let Self { identity, custody } = self;
        let outcome = match custody {
            BasisCustody::Exclusive(core) => core.release(&identity),
            BasisCustody::Shared(core) => match Arc::try_unwrap(core) {
                Ok(core) => core.release(&identity),
                Err(core) => {
                    drop(core);
                    WorthQueryApplicationBasisReleaseOutcome::SharedCustodyRetained
                }
            },
            BasisCustody::Transitioning => {
                unreachable!("custody promotion cannot escape its owner")
            }
        };
        WorthQueryApplicationBasisReleaseReceipt { identity, outcome }
    }

    fn basis(&self) -> &AdmittedRelationalBranchBasis {
        self.core()
            .basis
            .as_ref()
            .expect("an active Query basis retains its observation")
    }
}

impl BasisLeaseCore {
    fn release(
        mut self,
        identity: &WorthQueryApplicationBasisIdentity,
    ) -> WorthQueryApplicationBasisReleaseOutcome {
        let released = self.release_snapshot();
        let retention = self
            .retention
            .take()
            .expect("active Query basis owns Native retention");
        let receipt = self
            .graph
            .with_runtime(|runtime| runtime.release_component_basis(retention))
            .unwrap_or_else(|denial| {
                panic!(
                    "Query retained a lease its Native owner rejected: {:?}",
                    denial.denial()
                )
            });
        assert_eq!(
            receipt.descriptor(),
            identity.descriptor(),
            "Query releases its exact admitted Native basis"
        );
        self.basis.take();
        self.release_observation();
        WorthQueryApplicationBasisReleaseOutcome::NativeResourcesReleased {
            snapshot_released: released,
            relational_retention: receipt.outcome(),
        }
    }

    fn release_snapshot(&mut self) -> bool {
        let Some(snapshot) = self.snapshot.take() else {
            return false;
        };
        self.graph.with_runtime_mut(|runtime| {
            crate::relational_snapshot_release::release_query_snapshot(runtime, &snapshot);
        });
        true
    }

    fn release_observation(&self) {
        release(&self.state.active, 1).expect("live Query basis count cannot underflow");
    }
}

impl Drop for BasisLeaseCore {
    fn drop(&mut self) {
        if self.basis.take().is_some() {
            self.release_snapshot();
            self.retention.take();
            self.release_observation();
        }
    }
}
