//! Live original physical-read custody, issued only after native completion.
use super::super::retained_decision_facts::{StorageControl, StoreDenial};
use super::{super::snapshot_lease::WorthQueryApplicationSnapshotLease, CompletedFactProjection};
use crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload;
use std::sync::Arc;
use worth_relational::facade::{runtime::RelationalRuntime, snapshots::SnapshotHandle};

pub(in crate::domain_computation) struct CompletedOriginalRead {
    body: FactPayload,
    projection: Arc<CompletedFactProjection>,
    lease: WorthQueryApplicationSnapshotLease,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
    admission: crate::domain_computation::authorization::WorthQueryOperationAdmissionIdentity,
    graph_work: crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
}
impl CompletedOriginalRead {
    pub(super) fn from_native_completion<S, O, I, R>(
        body: FactPayload,
        projection: Arc<CompletedFactProjection>,
        lease: WorthQueryApplicationSnapshotLease,
        admission: &crate::domain_computation::authorization::WorthQueryAdmittedApplicationOperation<S, O, I, R>,
        control: StorageControl<'_, '_>,
    ) -> Result<(WorthQueryApplicationSnapshotLease, Option<Arc<Self>>), StoreDenial> {
        if !projection.physical() {
            return Ok((lease, None));
        }
        let memory = control.policy().reserve_memory(
            (std::mem::size_of::<Self>() + 2 * std::mem::size_of::<usize>()) as u64,
        );
        control.check_live()?;
        let memory = memory?;
        let (lease, retained) = lease.share_original(control)?;
        Ok((
            lease,
            Some(Arc::new(Self {
                body,
                projection,
                lease: retained,
                _memory: memory,
                admission: admission.admission_identity(),
                graph_work: admission.graph_work_session_identity(),
            })),
        ))
    }

    pub(in crate::domain_computation::primary_graph) fn matches_body(
        &self,
        body: &FactPayload,
        projection: &Arc<CompletedFactProjection>,
    ) -> bool {
        self.body.same_backing(body) && Arc::ptr_eq(&self.projection, projection)
    }
    pub(in crate::domain_computation::primary_graph) fn belongs_to_admission(
        &self,
        admission: crate::domain_computation::authorization::WorthQueryOperationAdmissionIdentity,
    ) -> bool {
        self.admission == admission
    }
    pub(in crate::domain_computation::primary_graph) fn belongs_to_graph_work(
        &self,
        graph_work: crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity,
    ) -> bool {
        self.graph_work == graph_work
    }

    pub(in crate::domain_computation::primary_graph) fn matches_source(
        &self,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        control: StorageControl<'_, '_>,
    ) -> Result<bool, StoreDenial> {
        // Both are actual live native observations. No digest or snapshot ID
        // substitutes for native source-position correspondence.
        use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;
        use worth_relational::facade::mvcc::CompanionPreflightBudget;
        use worth_relational::facade::runtime::RelationalSnapshotPositionAdmissionStop as Stop;
        let width = (self.lease.snapshot().branch_id().0.len() as u64)
            .checked_add(snapshot.branch_id().0.len() as u64)
            .ok_or(StoreDenial::Representability)?;
        let bytes = width.checked_mul(4).ok_or(StoreDenial::Representability)?;
        let mut admission = InvalidationEditAdmission::new(CompanionPreflightBudget {
            maximum_work_visits: Some(bytes.checked_add(20).ok_or(StoreDenial::Representability)?),
            maximum_preparation_bytes: Some(bytes),
        });
        // Native resolves each pin with two admissions. Hold each admitted
        // copy's grant until BOTH positioned observations have been consumed.
        let mut grants = [None, None, None, None];
        let mut next = 0;
        let mut admit = |work, bytes| -> Result<(), StoreDenial> {
            control.check_live()?;
            admission
                .charge_external_work(work)
                .map_err(|_| StoreDenial::Representability)?;
            admission
                .admit_read_scratch(bytes)
                .map_err(|_| StoreDenial::Representability)?;
            let memory = control.policy().reserve_memory(bytes);
            control.check_live()?;
            *grants.get_mut(next).ok_or(StoreDenial::Representability)? = memory?;
            next += 1;
            Ok(())
        };
        let truth = runtime.read_truth();
        let original = truth.positioned_snapshot_admitted(self.lease.snapshot(), &mut admit);
        let current = truth.positioned_snapshot_admitted(snapshot, &mut admit);
        control.check_live()?;
        match (original, current) {
            (Err(Stop::Admission(stop)), _) | (_, Err(Stop::Admission(stop))) => Err(stop),
            (Err(Stop::AccountingOverflow), _) | (_, Err(Stop::AccountingOverflow)) => {
                Err(StoreDenial::Representability)
            }
            (Ok(original), Ok(current)) => Ok(original == current),
            _ => Ok(false),
        }
    }
}
