//! One real public producer-tree settlement path for both typed schema profiles.
use super::*;
use producers::{child, parent};
pub type Request<'a, 'p, 's> = WorthQueryApplicationRequest<'a, 'p, 's, Schema>;
pub type Settled = WorthQuerySettledProgramOutputOperation<Schema, Program, program::Tree>;
pub fn settle(app: &world::App, request: &Request<'_, '_, '_>) -> Settled {
    settle_key(app, request, "root")
}
pub fn settle_key(app: &world::App, request: &Request<'_, '_, '_>, key: &str) -> Settled {
    let settled = settle_key_unchecked(app, request, key);
    settled.deliver_current(request).unwrap();
    settled
}
pub fn settle_key_unchecked(app: &world::App, request: &Request<'_, '_, '_>, key: &str) -> Settled {
    let mut operation = request
        .demand(parent::Demand(key.into()))
        .retain_program_outputs::<Program, program::Tree>(app)
        .unwrap_or_else(|e| panic!("{:?}", e.denial()));
    for _ in 0..128 {
        match operation.advance(request) {
            WorthQueryProgramOutputOperationAdvance::Pending(next) => operation = next,
            WorthQueryProgramOutputOperationAdvance::Settled(settled) => {
                return settled;
            }
            WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => panic!("{denial:?}"),
            WorthQueryProgramOutputOperationAdvance::Terminal(terminal) => {
                panic!("{:?}", terminal.denial())
            }
        }
    }
    panic!("tree did not settle");
}
pub fn close(settled: Settled, request: &Request<'_, '_, '_>) {
    settled
        .close(request)
        .unwrap_or_else(|e| panic!("{:?}", e.denial()));
}
pub fn child_id(settled: &Settled, key: &str) -> impl Eq + std::fmt::Debug + use<> {
    let (_, output) = settled
        .settlement()
        .outputs_for::<Schema, program::SplitParentSplitChild>()
        .find(|(demand, _)| demand.0 == key)
        .expect("actual child settlement");
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
pub fn resize(
    app: &world::App,
    request: &Request<'_, '_, '_>,
    width: u64,
    nonce: u64,
) -> WorthQueryApplicationCommitReceipt {
    let result = request
        .advanced()
        .mutate(source_control::Control {
            key: "root".into(),
            width,
        })
        .idempotency(&nonce)
        .execute_in_program::<Program>(
            app,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = result else {
        panic!("{result:?}")
    };
    receipt
}
pub fn assert_absent(request: &Request<'_, '_, '_>, keys: &[&str]) {
    for key in keys {
        assert!(
            matches!(
                request.query(Read((*key).into())).execute(),
                Err(WorthQueryApplicationRequestQueryDenial::ScopeResolution(_))
            ),
            "{key} remains live"
        );
        assert!(
            matches!(
                request
                    .query(realization_read::ReadRealization((*key).into()))
                    .execute(),
                Err(WorthQueryApplicationRequestQueryDenial::ScopeResolution(_))
            ),
            "{key} realization remains live"
        );
    }
}
pub fn assert_live(request: &Request<'_, '_, '_>, keys: &[&str]) {
    for key in keys {
        assert_eq!(
            request.query(Read((*key).into())).execute().unwrap().rows()[0].key,
            *key
        );
        assert_eq!(
            request
                .query(realization_read::ReadRealization((*key).into()))
                .execute()
                .unwrap()
                .rows()[0]
                .key,
            *key
        );
    }
}
