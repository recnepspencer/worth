use worth_query::facade::{domain, runtime};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PhaseDomain;
#[derive(Clone, Copy, Debug)]
pub(crate) struct PhaseFamily;
#[derive(Clone, Copy, Debug)]
pub(crate) struct PhaseOperation;

impl domain::WorthQueryDomainEntryMarker for PhaseDomain {
    fn domain_key(&self) -> &'static str {
        "WORTH.tests.geometry"
    }
    fn display_name(&self) -> &'static str {
        "Geometry"
    }
    fn required_capability_families(&self) -> &'static [domain::WorthQueryCapabilityFamily] {
        &[]
    }
}
impl domain::WorthQueryExecutableDomainOperation<PhaseDomain, PhaseFamily> for PhaseOperation {
    type Input = ();
    type Output = worth_query::facade::read::WorthQueryReadCompletion;
    type Publication = domain::WorthQueryPublishingOperation;
    type Execution = domain::WorthQueryWorkflowOperation;
}

#[path = "phased_workflow/definition.rs"]
mod definition;
#[path = "phased_workflow/fold_executor.rs"]
pub(crate) mod fold_executor;
#[path = "phased_workflow/reference_executor.rs"]
pub(crate) mod reference_executor;
#[path = "phased_workflow/world.rs"]
pub(crate) mod world;

pub(crate) const MEMBERS: [&str; 3] = ["left", "middle", "right"];

pub(crate) fn reference_workspace(name: &str) -> runtime::WorthQueryWorkspace {
    world::builder()
        .workflow_stage_executor(
            PhaseDomain,
            PhaseOperation,
            PhaseFamily,
            reference_executor::ReferenceExecutor,
        )
        .workflow_parallel_admission_provider(PhaseDomain, PhaseOperation, PhaseFamily, Admission)
        .workspace(name)
        .unwrap()
}

pub(crate) fn phased_workspace(name: &str) -> runtime::WorthQueryWorkspace {
    world::builder()
        .workflow_stage_executor(
            PhaseDomain,
            PhaseOperation,
            PhaseFamily,
            fold_executor::FoldExecutor,
        )
        .workflow_parallel_admission_provider(PhaseDomain, PhaseOperation, PhaseFamily, Admission)
        .workspace(name)
        .unwrap()
}

struct Admission;

impl domain::WorthQueryWorkflowParallelAdmissionProvider<PhaseDomain, PhaseOperation, PhaseFamily>
    for Admission
{
    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        world::execution_resource_support()
    }

    fn admit_parallel_frontier(
        &self,
        call: &domain::WorthQueryWorkflowParallelAdmissionCall,
    ) -> Result<
        worth_signal::facade::adapters::FrontierRouteEvidenceReceipt,
        domain::WorthQueryWorkflowParallelAdmissionFailure,
    > {
        assert_eq!(
            call.frontier()
                .iter()
                .map(|stage| stage.stage_identity())
                .collect::<Vec<_>>(),
            MEMBERS,
        );
        Ok(
            worth_signal::facade::adapters::FrontierRouteEvidenceReceipt::from_reason(
                worth_signal::facade::adapters::FrontierRouteEvidenceReason::AdmittedThroughput,
            ),
        )
    }
}
