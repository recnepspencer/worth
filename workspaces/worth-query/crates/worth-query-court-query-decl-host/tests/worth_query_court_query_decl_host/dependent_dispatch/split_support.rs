//! Drive only the declared sealed tree and observe its actual typed output roles.
use super::split_producer::{child, parent};
use super::*;
pub type Request<'a, 'p, 's> = WorthQueryApplicationRequest<'a, 'p, 's, Schema>;
pub type Operation = WorthQueryProgramOutputOperation<Schema, Program, split_program::Tree>;
pub type Settled = WorthQuerySettledProgramOutputOperation<Schema, Program, split_program::Tree>;
pub fn start(app: &world::App, request: &Request<'_, '_, '_>) -> Operation {
    request
        .demand(parent::Demand("root".into()))
        .retain_program_outputs::<Program, split_program::Tree>(app)
        .unwrap_or_else(|error| panic!("construction: {:?}", error.denial()))
}
pub fn settle(app: &world::App, request: &Request<'_, '_, '_>) -> Settled {
    finish(start(app, request), request)
}
pub fn finish(mut operation: Operation, request: &Request<'_, '_, '_>) -> Settled {
    for _ in 0..128 {
        match operation.advance(request) {
            WorthQueryProgramOutputOperationAdvance::Pending(next) => operation = next,
            WorthQueryProgramOutputOperationAdvance::Settled(settled) => {
                settled.deliver_current(request).unwrap();
                return settled;
            }
            WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => {
                panic!("tree refused: {denial:?}")
            }
            WorthQueryProgramOutputOperationAdvance::Terminal(terminal) => {
                panic!("tree terminal: {:?}", terminal.denial())
            }
        }
    }
    panic!("tree did not settle");
}
pub fn close(settled: Settled, request: &Request<'_, '_, '_>) {
    settled
        .close(request)
        .unwrap_or_else(|stop| panic!("close: {:?}", stop.denial()));
}
pub fn edit(
    app: &world::App,
    request: &Request<'_, '_, '_>,
    key: &str,
    value: u64,
    width: u64,
    nonce: u64,
) {
    control(
        app,
        request,
        source_control::Control {
            key: key.into(),
            change: source_control::Change::Edit { value, width },
        },
        nonce,
    );
}
pub fn control(
    app: &world::App,
    request: &Request<'_, '_, '_>,
    input: source_control::Control,
    nonce: u64,
) {
    let result = request
        .advanced()
        .mutate(input)
        .idempotency(&nonce)
        .execute_in_program::<Program>(
            app,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(
        matches!(
            result,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "source control: {result:?}"
    );
}
pub fn value(request: &Request<'_, '_, '_>, key: &str) -> u64 {
    let result = request
        .query(realization_read::ReadRealization(key.into()))
        .execute()
        .unwrap();
    assert_eq!(result.rows().len(), 1);
    assert_eq!(result.rows()[0].key, key);
    result.rows()[0].value
}
pub fn child_id(settled: &Settled, key: &str) -> impl Eq + std::fmt::Debug + use<> {
    let (_, output) = settled
        .settlement()
        .outputs_for::<Schema, split_program::SplitParentSplitChild>()
        .find(|(demand, _)| demand.0 == key)
        .expect("declared child settlement");
    if let Ok(roles) = output.outputs_of::<child::preserve::Outputs>() {
        roles
            .entity::<child::preserve::Output>()
            .unwrap()
            .entity_id()
    } else {
        output
            .outputs_of::<child::initial::Outputs>()
            .unwrap()
            .entity::<child::initial::Output>()
            .unwrap()
            .entity_id()
    }
}
pub fn sequence(selections: &Selections, key: &str, expected: &[(bool, bool)]) {
    let actual = selections
        .lock()
        .unwrap()
        .iter()
        .filter(|selection| selection.key == key)
        .map(|selection| (selection.preserve, selection.prior))
        .collect::<Vec<_>>();
    assert_eq!(actual, expected, "lifecycle for {key}");
}
