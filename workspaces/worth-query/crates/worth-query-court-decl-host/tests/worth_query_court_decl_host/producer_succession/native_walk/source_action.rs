//! An authored source completion precedes its explicit committed parent demand.
use super::*;
use worth_query_decl::facade::application_program::*;

pub struct Change;
impl ApplicationChangeShape for Change {
    const IDENTITY: &'static str = "succession.walk.source-change";
    const POSTURE: ApplicationChangePosture = ApplicationChangePosture::Reconstruct;
}
pub struct Start;
impl ApplicationConnectionIdentity for Start {
    const IDENTITY: &'static str = "succession.walk.source-parent";
}
impl ApplicationOccurrenceConnectionBinding<Schema, program::SplitEntry, program::SplitParent>
    for Start
{
}
pub type Connection = ApplicationConnectionRef<
    Schema,
    program::SplitEntry,
    program::SplitEntryOutput,
    program::SplitParent,
    program::SplitParentInput,
    Start,
>;
impl WorthQueryApplicationRequiredOutputConnection<Schema> for Start {
    type Source = source_control::Binding;
    type Demand = producers::parent::Demand;
    const IDENTITY: &'static str = <Self as ApplicationConnectionIdentity>::IDENTITY;
    fn demand_from_source(
        input: &source_control::Control,
    ) -> Result<Self::Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(producers::parent::Demand(input.key.clone()))
    }
}
impl WorthQueryApplicationRequiredOutputSource<Schema, Start> for source_control::Binding {
    fn demand_from_source(
        input: &source_control::Control,
    ) -> Result<producers::parent::Demand, WorthQueryRequiredOutputConnectionDenial> {
        <Start as WorthQueryApplicationRequiredOutputConnection<Schema>>::demand_from_source(input)
    }
}
pub type Tree = ApplicationDemandOutputGraph<Connection, ApplicationOutputLeaf>;
pub struct Delivery;
impl ApplicationActionDelivery<Schema, source_control::Binding> for Delivery {
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "succession.walk.source-delivery";
}
type Action = ApplicationActionReference<
    Schema,
    ApplicationRootComposition,
    program::SplitEntry,
    source_control::Binding,
    Delivery,
>;

pub fn perform(
    app: &producer_world::App,
    request: &settlement::Request<'_, '_, '_>,
    width: u64,
    nonce: u64,
) {
    let before = request.query(Read("root".into())).execute().unwrap().rows()[0].width;
    assert_ne!(
        before, width,
        "each authored action performs a real source change"
    );
    let mut operation = request
        .mutate(source_control::Control {
            key: "root".into(),
            width,
        })
        .without_source()
        .idempotency_owned(nonce)
        .unwrap_or_else(|(_, _, cause)| panic!("source capture: {cause:?}"))
        .retain_operation::<producer_world::NativeProgram, Action>(app)
        .unwrap_or_else(|(_, cause)| panic!("installed source action: {cause:?}"));
    for _ in 0..32 {
        match operation.advance(request) {
            WorthQuerySourceOnlyAdvance::Progress(next) => operation = next,
            WorthQuerySourceOnlyAdvance::Completed(completed) => {
                assert!(
                    matches!(
                        completed.evidence(),
                        WorthQuerySourceCompletion::Fresh { .. }
                    ),
                    "the actual authored source committed freshly"
                );
                completed
                    .close(request)
                    .unwrap_or_else(|(_, cause)| panic!("source close: {cause:?}"));
                assert_eq!(
                    request.query(Read("root".into())).execute().unwrap().rows()[0].width,
                    width,
                    "source completion is committed before output demand"
                );
                demand(app, request);
                return;
            }
            WorthQuerySourceOnlyAdvance::Refused { denial, .. } => {
                panic!("source action: {denial:?}")
            }
            WorthQuerySourceOnlyAdvance::Terminal(terminal) => {
                panic!("source action: {:?}", terminal.native_outcome())
            }
        }
    }
    panic!("bounded authored source action did not finish");
}

fn demand(app: &producer_world::App, request: &settlement::Request<'_, '_, '_>) {
    let mut operation = request
        .demand(producers::parent::Demand("root".into()))
        .retain_program_outputs::<producer_world::NativeProgram, Tree>(app)
        .unwrap_or_else(|cause| panic!("installed demand: {:?}", cause.denial()));
    for _ in 0..32 {
        match operation.advance(request) {
            WorthQueryProgramOutputOperationAdvance::Pending(next) => operation = next,
            WorthQueryProgramOutputOperationAdvance::Settled(settled) => {
                settled.deliver_current(request).unwrap();
                settled
                    .close(request)
                    .unwrap_or_else(|cause| panic!("demand close: {:?}", cause.denial()));
                return;
            }
            WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => {
                panic!("demand: {denial:?}")
            }
            WorthQueryProgramOutputOperationAdvance::Terminal(terminal) => {
                panic!("demand: {:?}", terminal.denial())
            }
        }
    }
    panic!("bounded committed-source demand did not finish");
}
