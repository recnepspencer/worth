//! Generator ownership is independent of the opted-in publications its children own.
use super::{courts::*, producers::independent_parent, *};

pub(super) fn settle_independent(app: &world::App, request: &Request<'_, '_, '_>) {
    let mut operation = request
        .demand(independent_parent::Demand("root".into()))
        .retain_program_outputs::<Program, program::IndependentTree>(app)
        .unwrap_or_else(|e| panic!("{:?}", e.denial()));
    for _ in 0..128 {
        match operation.advance(request) {
            WorthQueryProgramOutputOperationAdvance::Pending(next) => operation = next,
            WorthQueryProgramOutputOperationAdvance::Settled(settled) => {
                settled.deliver_current(request).unwrap();
                settled
                    .close(request)
                    .unwrap_or_else(|e| panic!("{:?}", e.denial()));
                return;
            }
            WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => panic!("{denial:?}"),
            WorthQueryProgramOutputOperationAdvance::Terminal(terminal) => {
                panic!("{:?}", terminal.denial())
            }
        }
    }
    panic!("independent generator did not settle");
}

#[test]
fn a_non_opted_in_generator_shrinks_and_retires_its_childs_owned_closure() {
    assert_eq!(<independent_parent::preserve::Producer as WorthQueryApplicationProducerBinding<Schema>>::OWNERSHIP,
        ProducerPublicationOwnership::Independent);
    let app = world::install(2, Arc::default());
    let (scope, principal) = world::authenticate(&app);
    let request = app.runtime().request(&principal, &scope);
    settle_independent(&app, &request);
    let sibling = request
        .query(realization_read::ReadRealization("root.0".into()))
        .execute()
        .unwrap()
        .rows()[0]
        .native_id;
    resize(&app, &request, 1, 1);
    settle_independent(&app, &request);
    assert_absent(&request, &["root.1", "root.1.0"]);
    assert_live(&request, &["root", "root.0", "root.0.0"]);
    assert_eq!(
        request
            .query(realization_read::ReadRealization("root.0".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .native_id,
        sibling
    );

    let history = checkpoint::envelopes(&checkpoint::capture(&app));
    let event = history
        .iter()
        .find(|e| {
            e.get("producer_cascade_retirement")
                .is_some_and(|g| !g.is_null())
        })
        .unwrap();
    let generator = &event["required_output_origin"];
    assert_eq!(
        generator["producer"],
        "dispatch.independent.parent.preserve"
    );
    assert_eq!(generator["source_scope_owned"], false);
    let retired = event["producer_cascade_retirement"]["publications"]
        .as_array()
        .unwrap();
    assert_eq!(retired.len(), 2, "the actual child and leaf publications");
    assert!(retired.iter().all(|p| p["source_scope_owned"] == true));
}
