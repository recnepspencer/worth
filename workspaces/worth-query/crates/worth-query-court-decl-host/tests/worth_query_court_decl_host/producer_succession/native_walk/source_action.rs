//! An authored source edit owns its real Width effect and required parent output.
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
pub type Tree = ApplicationOutputGraph<Connection, ApplicationOutputLeaf>;
pub struct Delivery;
impl ApplicationActionDelivery<Schema, source_control::Binding> for Delivery {
    type Mode = ApplicationFixedDeliveryMode<Tree, ApplicationNoProjection>;
    const IDENTITY: &'static str = "succession.walk.source-delivery";
}
impl ApplicationFixedActionDelivery<Schema, source_control::Binding> for Delivery {
    type Root = Tree;
    type Projection = ApplicationNoProjection;
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
            WorthQueryOperationAdvance::Progress {
                operation: next, ..
            } => operation = next,
            WorthQueryOperationAdvance::Settled(settled) => {
                assert_eq!(
                    settled.declared_action().binding(),
                    <source_control::Binding as ApplicationMutationBinding<Schema>>::IDENTITY
                );
                settled.deliver_current(request).unwrap();
                settled
                    .close(request)
                    .unwrap_or_else(|(_, cause)| panic!("source close: {cause:?}"));
                assert_eq!(
                    request.query(Read("root".into())).execute().unwrap().rows()[0].width,
                    width,
                    "the actual source handler committed the requested Width"
                );
                return;
            }
            WorthQueryOperationAdvance::Refused { denial, .. }
            | WorthQueryOperationAdvance::Terminal { denial, .. } => {
                panic!("source action: {denial:?}")
            }
        }
    }
    panic!("bounded authored source action did not finish");
}
