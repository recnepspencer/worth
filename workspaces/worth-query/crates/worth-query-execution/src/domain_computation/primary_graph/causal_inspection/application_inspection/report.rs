//! The same original ticket admits and retains the actual descriptive report.
use super::{
    identity::{
        identities, InspectionIdentityStop, InspectionOriginalReference,
        InspectionReferenceIdentities,
    },
    WorthQueryApplicationInspectionTarget,
};
use crate::domain_computation::{
    primary_graph::admitted_capture::{
        checkpoint, run_retained_stage, CaptureStop, RetainedStorageDenial,
    },
    provider_session::{
        WorthQueryGraphWorkManagedRunIdentity, WorthQueryGraphWorkSessionIdentity,
        WorthQueryManagedGraphWorkSession,
    },
};
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest, MapKernelFailure};
use worth_query_admission::facade::graph_obligation::WorthQueryGraphWorkPlanIdentity;
use worth_runtime_bridge::facade::{
    BridgeCausalExplanationEnvelope, BridgeCausalInspectionAdmissionSummaryKind,
    BridgeQueryObservationReportingInputDenial, BridgeQueryObservationReportingRequest,
    BridgeQueryObservationReportingStop,
};

mod connected;
pub(super) mod rich;
pub(super) use connected::{materialize_connected, FundedConnectedReport};
mod comparison;
pub(super) use comparison::{
    materialize_comparison, ComparisonReportPayload, FundedComparisonReport,
};

type ResourceStop = MapKernelFailure<RetainedStorageDenial>;
pub enum InspectionReportDenial {
    Identity(InspectionIdentityStop<ResourceStop>),
    Input(BridgeQueryObservationReportingInputDenial),
    Bridge(BridgeQueryObservationReportingStop<ResourceStop>),
    ForeignComparison,
    ForeignMaterialization,
    Storage(RetainedStorageDenial),
}

pub(super) struct InspectionReportPayload {
    envelope: BridgeCausalExplanationEnvelope,
    identities: InspectionReferenceIdentities,
    availability: Option<super::availability::WorthQueryInspectionEvidenceAvailability>,
}

pub struct FundedInspectionReport {
    payload: InspectionReportPayload,
    session: WorthQueryGraphWorkSessionIdentity,
    managed_run: WorthQueryGraphWorkManagedRunIdentity,
    plan: WorthQueryGraphWorkPlanIdentity,
    basis: crate::domain_computation::primary_graph::WorthQueryApplicationBasisIdentity,
    capacity_identity:
        worth_query_admission::integration::WorthQueryGraphCapacityReservationIdentity,
    target: Arc<WorthQueryApplicationInspectionTarget>,
    retained_bytes: u64,
    reference: InspectionOriginalReference,
}

pub fn materialize(
    execution: &ExecutionRequest<'_, '_>,
    target: &Arc<WorthQueryApplicationInspectionTarget>,
    session: &WorthQueryManagedGraphWorkSession,
    reference: InspectionOriginalReference,
    memory: &mut Option<ExecutionMemoryReservation>,
) -> Result<FundedInspectionReport, CaptureStop<InspectionReportDenial>> {
    run_retained_stage(execution, memory, |context, mut memory| {
        let mut admit = |work: usize, bytes: usize| {
            let work = u64::try_from(work).map_err(|_| {
                MapKernelFailure::Stop(worth_execution::MapKernelStop::WorkCounterOverflow)
            })?;
            checkpoint(context, work).map_err(MapKernelFailure::Stop)?;
            let bytes = u64::try_from(bytes)
                .map_err(|_| MapKernelFailure::Domain(RetainedStorageDenial::Overflow))?;
            if let Some(memory) = memory.as_deref_mut() {
                let total = memory
                    .bytes()
                    .checked_add(bytes)
                    .ok_or(MapKernelFailure::Domain(RetainedStorageDenial::Overflow))?;
                memory.resize(total).map_err(MapKernelFailure::Domain)?;
            }
            Ok(())
        };
        let original = match reference {
            InspectionOriginalReference::Before => target.before(),
            InspectionOriginalReference::After => target.original(),
            InspectionOriginalReference::Source => None,
        };
        let availability = original
            .map(super::availability::WorthQueryInspectionEvidenceAvailability::from_original);
        let identities = identities(
            target,
            session,
            reference,
            availability.as_ref(),
            &mut admit,
        )
        .map_err(|cause| MapKernelFailure::Domain(InspectionReportDenial::Identity(cause)))?;
        let request = if reference == InspectionOriginalReference::Source {
            BridgeQueryObservationReportingRequest::from_performed_source(
                BridgeCausalInspectionAdmissionSummaryKind::Admitted,
                &identities.admission,
                &identities.anchor,
                &identities.observation,
            )
        } else {
            BridgeQueryObservationReportingRequest::new(
                BridgeCausalInspectionAdmissionSummaryKind::Admitted,
                &identities.admission,
                &identities.anchor,
                &identities.observation,
            )
        }
        .map_err(|cause| MapKernelFailure::Domain(InspectionReportDenial::Input(cause)))?;
        let envelope = request
            .assemble_with_admission(&mut admit)
            .map_err(|cause| MapKernelFailure::Domain(InspectionReportDenial::Bridge(cause)))?;
        let (bound_target, basis) =
            session
                .inspection_binding()
                .ok_or(MapKernelFailure::Domain(
                    InspectionReportDenial::ForeignMaterialization,
                ))?;
        if !Arc::ptr_eq(target, bound_target) {
            return Err(MapKernelFailure::Domain(
                InspectionReportDenial::ForeignMaterialization,
            ));
        }
        let capacity = session
            .inspection_capacity_identity()
            .ok_or(MapKernelFailure::Domain(
                InspectionReportDenial::ForeignMaterialization,
            ))?;
        let bytes = basis
            .retained_clone_heap_bytes()
            .and_then(|bytes| usize::try_from(bytes).ok())
            .ok_or(MapKernelFailure::Domain(InspectionReportDenial::Storage(
                RetainedStorageDenial::Overflow,
            )))?;
        let selection_bytes = match basis.selection() {
            crate::domain_computation::primary_graph::WorthQueryApplicationBasisSelectionIdentity::Relational => 0,
            crate::domain_computation::primary_graph::WorthQueryApplicationBasisSelectionIdentity::Product(product) => product.branch_identity().name().as_str().len(),
        };
        // Fund the clone and both completion comparisons before retaining this basis.
        // Product selection custody is shared, but its name still has comparison cost.
        let work = bytes
            .checked_mul(3)
            .and_then(|work| {
                selection_bytes
                    .checked_mul(2)
                    .and_then(|selection| work.checked_add(selection))
            })
            .ok_or(MapKernelFailure::Domain(InspectionReportDenial::Storage(
                RetainedStorageDenial::Overflow,
            )))?;
        admit(work, bytes).map_err(|cause| {
            MapKernelFailure::Domain(InspectionReportDenial::Bridge(
                BridgeQueryObservationReportingStop::Resource(cause),
            ))
        })?;
        drop(admit);
        // This is this actual stage's admitted growth, not its prior live backing.
        // Canonical preparation's conservative transient allowance stays funded
        // with the report until its final owner closes.
        let retained_bytes = memory.as_deref().map_or(0, |memory| memory.bytes());
        Ok(FundedInspectionReport {
            payload: InspectionReportPayload {
                envelope,
                identities,
                availability,
            },
            session: session.identity(),
            managed_run: session.managed_run_identity(),
            plan: session.plan_identity(),
            basis: basis.clone(),
            capacity_identity: capacity,
            target: Arc::clone(target),
            retained_bytes,
            reference,
        })
    })
}

impl FundedInspectionReport {
    pub(super) fn managed_run(&self) -> WorthQueryGraphWorkManagedRunIdentity {
        self.managed_run
    }
    pub(super) fn basis(
        &self,
    ) -> &crate::domain_computation::primary_graph::WorthQueryApplicationBasisIdentity {
        &self.basis
    }
    pub(super) fn capacity_identity(
        &self,
    ) -> worth_query_admission::integration::WorthQueryGraphCapacityReservationIdentity {
        self.capacity_identity
    }

    pub(super) fn reference(&self) -> InspectionOriginalReference {
        self.reference
    }

    pub(super) fn session(&self) -> WorthQueryGraphWorkSessionIdentity {
        self.session
    }
    pub(super) fn plan(&self) -> WorthQueryGraphWorkPlanIdentity {
        self.plan
    }
    pub(super) fn target(&self) -> &Arc<WorthQueryApplicationInspectionTarget> {
        &self.target
    }
    pub(super) fn release_target(self) -> (InspectionReportPayload, u64) {
        (self.payload, self.retained_bytes)
    }
}

impl InspectionReportPayload {
    pub(super) fn availability(
        &self,
    ) -> super::availability::WorthQueryInspectionEvidenceAvailability {
        self.availability
            .expect("only a published original Query report exposes Query availability")
    }

    pub(super) fn envelope(&self) -> &BridgeCausalExplanationEnvelope {
        &self.envelope
    }
    pub(super) fn report_identity(&self) -> &str {
        self.envelope.envelope_for_reporting()
    }
    pub(super) fn observation_identity(&self) -> &str {
        self.identities
            .observation
            .terminal_projection_for_reporting()
    }
}
