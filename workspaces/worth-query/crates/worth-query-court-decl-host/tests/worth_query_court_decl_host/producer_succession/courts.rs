//! Literal branch rosters and generation observations through real producer settlement.
use super::*;
use producers::{child, parent};
pub type Request<'a, 'p, 's> = WorthQueryApplicationRequest<'a, 'p, 's, Schema>;
pub type Settled = WorthQuerySettledProgramOutputOperation<Schema, Program, program::Tree>;
pub fn settle(app: &world::App, request: &Request<'_, '_, '_>) -> Settled {
    let mut operation = request
        .demand(parent::Demand("root".into()))
        .retain_program_outputs::<Program, program::Tree>(app)
        .unwrap_or_else(|e| panic!("{:?}", e.denial()));
    for _ in 0..128 {
        match operation.advance(request) {
            WorthQueryProgramOutputOperationAdvance::Pending(next) => operation = next,
            WorthQueryProgramOutputOperationAdvance::Settled(settled) => {
                settled.deliver_current(request).unwrap();
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
#[test]
fn generator_shrink_retires_the_deleted_childs_owned_publications() {
    let app = world::install_two_levels();
    let (scope, principal) = world::authenticate(&app);
    let request = app.runtime().request(&principal, &scope);
    let first = settle(&app, &request);
    let sibling = child_id(&first, "root.0");
    close(first, &request);
    resize(&app, &request, 1, 1);
    let shrunk = settle(&app, &request);
    assert_eq!(child_id(&shrunk, "root.0"), sibling);
    close(shrunk, &request);
    assert_absent(&request, &["root.1"]);
    assert_live(&request, &["root", "root.0"]);
}
#[test]
fn three_level_shrink_retires_exactly_the_removed_branch_and_regrows_fresh() {
    let selections = Selections::default();
    let app = world::install(2, selections.clone());
    let (scope, principal) = world::authenticate(&app);
    let request = app.runtime().request(&principal, &scope);
    let first = settle(&app, &request);
    let old = child_id(&first, "root.1");
    let sibling = child_id(&first, "root.0");
    close(first, &request);
    resize(&app, &request, 1, 2);
    let shrunk = settle(&app, &request);
    assert_eq!(child_id(&shrunk, "root.0"), sibling);
    close(shrunk, &request);
    assert_absent(&request, &["root.1", "root.1.0"]);
    assert_live(&request, &["root", "root.0", "root.0.0"]);
    resize(&app, &request, 2, 3);
    let regrown = settle(&app, &request);
    assert_ne!(child_id(&regrown, "root.1"), old);
    assert_eq!(child_id(&regrown, "root.0"), sibling);
    close(regrown, &request);
    assert_live(
        &request,
        &["root", "root.0", "root.0.0", "root.1", "root.1.0"],
    );
    let selected = selections.lock().unwrap();
    let regrown = selected
        .iter()
        .filter(|s| s.key == "root.1")
        .collect::<Vec<_>>();
    assert_eq!(regrown.len(), 2);
    assert!(regrown
        .iter()
        .all(|s| s.tier == 1 && !s.preserve && !s.prior));
}
