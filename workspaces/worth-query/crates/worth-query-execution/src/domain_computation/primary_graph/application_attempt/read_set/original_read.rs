//! Live original physical-read custody, issued only after native completion.
use crate::domain_computation::primary_graph::vocabulary::application_attempt::retained_decision_facts::control::StorageControl;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::retained_decision_facts::StoreDenial;
use super::super::snapshot_lease::WorthQueryApplicationSnapshotLease;
use crate::domain_computation::primary_graph::vocabulary::application_attempt::application_projection::CompletedFactProjection;
use crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload;
use std::sync::Arc;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::snapshots::SnapshotHandle;

pub struct CompletedOriginalRead {
    body: FactPayload,
    projection: Arc<CompletedFactProjection>,
    lease: WorthQueryApplicationSnapshotLease,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
    admission: crate::domain_computation::vocabulary::authorization::operation_admission::WorthQueryOperationAdmissionIdentity,
    graph_work: crate::domain_computation::vocabulary::provider_session::session_identity::WorthQueryGraphWorkSessionIdentity,
}
impl CompletedOriginalRead {
    pub(super) fn from_native_completion(
        body: FactPayload,
        projection: Arc<CompletedFactProjection>,
        lease: WorthQueryApplicationSnapshotLease,
        admission: &crate::domain_computation::authorization::WorthQueryAdmittedOperation,
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
        admission: crate::domain_computation::vocabulary::authorization::operation_admission::WorthQueryOperationAdmissionIdentity,
    ) -> bool {
        self.admission == admission
    }
    pub(in crate::domain_computation::primary_graph) fn belongs_to_graph_work(
        &self,
        graph_work: crate::domain_computation::vocabulary::provider_session::session_identity::WorthQueryGraphWorkSessionIdentity,
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

        let truth = runtime.read_truth();
        let original = truth.positioned_snapshot(self.lease.snapshot());
        let current = truth.positioned_snapshot(snapshot);
        control.check_live()?;
        match (original, current) {
            (Ok(original), Ok(current)) => Ok(original == current),
            _ => Ok(false),
        }
    }
}
