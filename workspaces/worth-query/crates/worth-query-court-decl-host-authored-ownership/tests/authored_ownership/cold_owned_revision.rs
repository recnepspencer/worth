//! An installed owned body settles through ordinary worker requests after cold reopen.
use super::{
    correction_courts::try_install_definition_with_limits, lifecycle::Create,
    parameter_inverse::Edit, *,
};
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::*;

#[test]
fn an_ordinary_body_revision_settles_after_checkpoint_and_cold_reopen() {
    type S = Model<68>;
    let host = try_install_definition_with_limits::<S, RequiredProgram>(limits()).unwrap();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    create(&host, &request, 10, vec![20]);
    settle(&host, &request, 10);
    edit(&host, &request, 3);
    let checkpoint = host
        .with_host_advancement(|phase| {
            host.capture_application_checkpoint(
                &phase,
                WorthQueryCheckpointCapturePolicy::SystemAllocation,
            )
            .unwrap()
        })
        .unwrap();
    drop(host);
    let host = reopen::<S>(checkpoint);
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    edit(&host, &request, 4);
    assert!(request.query(query::Input { id: 103 }).execute().is_err());
    assert!(request.query(query::Input { id: 104 }).execute().is_ok());
}

pub(super) type Root<S> = ApplicationOutputGraph<program::Connection<S>, ApplicationOutputLeaf>;
pub struct RequiredProgram;
impl<S: Scenario> ApplicationProgramDefinition<S> for RequiredProgram {
    type Contributions = (Contribution,);
    type Outputs = ApplicationProgramOutputs<Root<S>>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("ownership.ordinary-required.v1");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        <program::Program as ApplicationProgramDefinition<S>>::feature_specs()
    }
}
impl<S: Scenario> WorthQueryApplicationRequiredOutputSource<S, program::Start>
    for parameter_inverse::Binding<S>
{
    fn demand_from_source(
        input: &Edit,
    ) -> Result<program::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(program::Demand(input.id))
    }
}
pub(super) fn create<S: Scenario>(
    host: &WorthQueryProgramApplicationRuntime<S, RequiredProgram>,
    request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    id: u64,
    co_anchors: Vec<u64>,
) {
    let outcome = request
        .advanced()
        .mutate(Create {
            create_co: true,
            co_anchors,
            scope: 1,
            id,
            depth: 1,
        })
        .without_source()
        .idempotency(&"create-body".to_owned())
        .execute_in_program(host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{outcome:?}"
    );
}
pub(super) fn settle<S: Scenario>(
    host: &WorthQueryProgramApplicationRuntime<S, RequiredProgram>,
    request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    id: u64,
) {
    let mut pending = request
        .demand(program::Demand(id))
        .retain_program_outputs::<RequiredProgram, Root<S>>(host)
        .unwrap_or_else(|failure| panic!("{:?}", failure.denial()));
    for _ in 0..32 {
        match pending.advance(request) {
            WorthQueryProgramOutputOperationAdvance::Pending(next) => pending = next,
            WorthQueryProgramOutputOperationAdvance::Settled(settled) => {
                settled
                    .close(request)
                    .unwrap_or_else(|failure| panic!("{:?}", failure.denial()));
                return;
            }
            WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => panic!("{denial:?}"),
            WorthQueryProgramOutputOperationAdvance::Terminal(terminal) => {
                panic!("{:?}", terminal.denial())
            }
        }
    }
    panic!("initial body output did not settle");
}
fn reopen<S: Scenario>(
    checkpoint: WorthQueryApplicationCheckpoint,
) -> WorthQueryProgramApplicationRuntime<S, RequiredProgram> {
    let schema = S::declaration().unwrap();
    in_memory_program_from_checkpoint(
        ApplicationProgramAuthoring::<S, RequiredProgram>::begin()
            .with_schema(&schema)
            .validated_program()
            .unwrap(),
        schema,
        ((),),
        ApplicationBuildIdentity::parse("919b0039-0000-0000-0000-000000000002").unwrap(),
        limits(),
        checkpoint,
    )
    .unwrap()
}
fn edit<S: Scenario>(
    host: &WorthQueryProgramApplicationRuntime<S, RequiredProgram>,
    request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    value: u64,
) {
    let outcome = request
        .advanced()
        .mutate(Edit { id: 10, value })
        .without_source()
        .idempotency(&format!("ordinary-edit-{value}"))
        .execute_performed::<RequiredProgram, Root<S>>(
            host,
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let performed = match outcome {
        WorthQueryApplicationPerformedMutationOutcome::Performed(performed) => performed,
        WorthQueryApplicationPerformedMutationOutcome::NotPerformed(outcome) => {
            panic!("source refused: {outcome:?}")
        }
        WorthQueryApplicationPerformedMutationOutcome::RequiredOutputDenied(failure) => {
            panic!("outputs refused: {:?}", failure.denial())
        }
        _ => panic!("source must commit"),
    };
    let mut started = performed
        .start_required_outputs(host, request, WorthQueryOutputDemandControls::default())
        .unwrap_or_else(|failure| panic!("start outputs: {:?}", failure.denial()));
    let progress = started.required_output_mut().settle(host, request).unwrap();
    assert!(
        matches!(
            progress,
            WorthQueryApplicationProgramOutputProgress::Settled(_)
        ),
        "body must settle at its performed basis"
    );
}

pub(super) struct Delivery;
impl<S: Scenario> ApplicationActionDelivery<S, parameter_inverse::Binding<S>> for Delivery {
    type Mode = ApplicationFixedDeliveryMode<Root<S>, ApplicationNoProjection>;
    const IDENTITY: &'static str = "ownership.ordinary-edit-delivery.v1";
}
impl<S: Scenario> ApplicationFixedActionDelivery<S, parameter_inverse::Binding<S>> for Delivery {
    type Root = Root<S>;
    type Projection = ApplicationNoProjection;
}

fn limits() -> WorthQueryInMemoryApplicationLimits {
    host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
}
