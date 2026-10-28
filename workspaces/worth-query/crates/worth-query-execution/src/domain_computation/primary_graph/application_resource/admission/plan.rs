use super::*;
use crate::domain_computation::primary_graph::admitted_capture::{checkpoint, run_stage};
use worth_execution::{ChargedBytes, LeaseDenial, MapKernelFailure, WorkCeilingDenial};
use worth_query_admission::facade::graph_obligation::{
    admit_prepared_resource_read_graph_work, WorthQueryAdmittedGraphWorkPlan,
    WorthQueryGraphWorkAdmissionDenial, WorthQueryGraphWorkCapacityAdmissionStop,
    WorthQueryPreparedResourceReadGraphWork,
};
use worth_query_declaration::facade::{
    application_resource::ApplicationResourceBinding, application_schema::ApplicationSchema,
};
use worth_query_installation::facade::{
    WorthQueryInstalledApplicationResource, WorthQueryResourceAccessPurpose,
};

#[expect(
    dead_code,
    reason = "original typed refusal is retained opaquely; disclosure uses approved projections"
)]
pub(in crate::domain_computation::primary_graph::application_resource) enum ResourcePlanDenial {
    Graph(WorthQueryGraphWorkAdmissionDenial),
    Request(WorkCeilingDenial),
    Overflow,
}

impl ChargedBytes for ResourcePlanDenial {
    fn additional_charged_bytes(&self) -> u64 {
        match self {
            Self::Graph(cause) => cause.additional_charged_bytes(),
            Self::Request(_) | Self::Overflow => 0,
        }
    }
}

struct CapturedPlan {
    plan: WorthQueryAdmittedGraphWorkPlan,
    bytes: u64,
}
impl ChargedBytes for CapturedPlan {
    fn additional_charged_bytes(&self) -> u64 {
        self.bytes
    }
}

pub(super) fn admit<Schema: ApplicationSchema, Binding: ApplicationResourceBinding<Schema>>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    resource: &WorthQueryInstalledApplicationResource<Schema, Binding>,
    purpose: WorthQueryResourceAccessPurpose,
    prepared: &WorthQueryPreparedResourceReadGraphWork,
) -> Result<
    (
        WorthQueryAdmittedGraphWorkPlan,
        Option<worth_execution::ExecutionMemoryReservation>,
    ),
    ResourceAdmissionDenial,
> {
    let support = runtime.primary_provider.application_resource_support_ref();
    let (capture, memory) =
        run_stage(execution, 0, |context, mut memory| {
            let mut admit =
                |work, bytes| {
                    checkpoint(context, work).map_err(WorkCeilingDenial::Stopped)?;
                    if let Some(memory) = memory.as_deref_mut() {
                        let total = memory.bytes().checked_add(bytes).ok_or(
                            WorkCeilingDenial::Admission(LeaseDenial::ChargedBytesOverflow),
                        )?;
                        memory.resize(total).map_err(|cause| {
                            WorkCeilingDenial::Admission(LeaseDenial::MemoryExhausted(cause))
                        })?;
                    }
                    Ok(())
                };
            let plan = admit_prepared_resource_read_graph_work(
                resource, purpose, prepared, support, &mut admit,
            )
            .map_err(|cause| {
                MapKernelFailure::Domain(match cause {
                    WorthQueryGraphWorkCapacityAdmissionStop::Admission(cause) => {
                        ResourcePlanDenial::Request(cause)
                    }
                    WorthQueryGraphWorkCapacityAdmissionStop::Denial(cause) => {
                        ResourcePlanDenial::Graph(cause)
                    }
                    WorthQueryGraphWorkCapacityAdmissionStop::AccountingOverflow => {
                        ResourcePlanDenial::Overflow
                    }
                })
            })?;

            Ok(CapturedPlan {
                plan,
                bytes: memory.as_deref().map_or(0, |memory| memory.bytes()),
            })
        })
        .map_err(ResourceAdmissionDenial::Plan)?;
    Ok((capture.plan, memory))
}
