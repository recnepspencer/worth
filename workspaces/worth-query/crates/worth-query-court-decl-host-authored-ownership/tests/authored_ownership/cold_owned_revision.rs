//! The ordinary required program whose owned body settles through worker requests.
use super::{lifecycle::Create, parameter_inverse::Edit, *};
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;

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

pub(super) struct Delivery;
impl<S: Scenario> ApplicationActionDelivery<S, parameter_inverse::Binding<S>> for Delivery {
    type Mode = ApplicationFixedDeliveryMode<Root<S>, ApplicationNoProjection>;
    const IDENTITY: &'static str = "ownership.ordinary-edit-delivery.v1";
}
impl<S: Scenario> ApplicationFixedActionDelivery<S, parameter_inverse::Binding<S>> for Delivery {
    type Root = Root<S>;
    type Projection = ApplicationNoProjection;
}
