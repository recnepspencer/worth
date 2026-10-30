//! Genuine direct plans must preserve the finite batch contract too.
pub use super::super::{product_query_support, schema::*, world::CourtroomWorld};
pub use worth_query_host::facade::{
    declaration::application_query::ApplicationQueryParameterSet,
    primary_graph::{
        self, WorthQueryApplicationBatchReadDenial as ReadDenial,
        WorthQueryApplicationQueryBatchAdmission as Batch,
        WorthQueryApplicationQueryBatchResourceDenial as Resource,
    },
};

pub fn batch(world: &CourtroomWorld) -> Batch {
    let query = world
        .application
        .installed_schema()
        .certification_query(TemporalIntentQuery::reference())
        .unwrap();
    let maximum_results = std::num::NonZeroUsize::new(query.read_graph().maximum_result_count())
        .expect("the installed TemporalIntent query has a nonzero result cardinality");
    world
        .application
        .application_query_batch_from_installed_reads(&[world
            .application
            .plan_installed_query_batch_reads(&query, 1, maximum_results)
            .unwrap()])
        .unwrap()
}

pub fn execute(
    world: &CourtroomWorld,
    batch: &Batch,
    retained: bool,
) -> Result<
    primary_graph::WorthQueryApplicationBatchResult<TemporalIntentQuery, IntentQueryResult>,
    ReadDenial,
> {
    let request = super::super::world::request_scope();
    let principal = product_query_support::principal(world, &request);
    let query = world
        .application
        .installed_schema()
        .certification_query(TemporalIntentQuery::reference())
        .unwrap();
    let maximum_results = std::num::NonZeroUsize::new(query.read_graph().maximum_result_count())
        .expect("the installed TemporalIntent query has a nonzero result cardinality");
    let selected = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .on_branch(world.application.current_world())
                .select(&phase)
                .unwrap()
        })
        .expect("installed host call opens");
    let scope = world
        .application
        .with_host_advancement(|phase| {
            selected
                .resolve_entity(
                    &phase,
                    IntentIdentityField::reference(),
                    "intent-1".to_owned(),
                    &request,
                    primary_graph::WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("installed host call opens");
    let access = primary_graph::WorthQueryApplicationQueryAccessContext::new(&principal, &scope);
    let mut basis = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .on_branch(world.application.current_world())
                .select(&phase)
                .unwrap()
        })
        .expect("installed host call opens");
    let plan = if retained {
        match world
            .application
            .with_host_advancement(|phase| {
                selected.admit_retained_application_query_from_selection(
                    &phase,
                    &mut basis,
                    &query,
                    &access,
                    ApplicationQueryParameterSet::new(),
                    primary_graph::WorthQueryProductQueryControls::new(maximum_results, &request),
                    batch,
                )
            })
            .expect("installed host call opens")
        {
            Ok(plan) => plan,
            Err(primary_graph::WorthQueryRetainedBatchQueryAdmissionDenial::Resource(cause)) => {
                return Err(ReadDenial::Resource(cause))
            }
            Err(other) => panic!("ordinary retained admission must remain valid: {other:?}"),
        }
    } else {
        world
            .application
            .with_host_advancement(|phase| {
                selected
                    .admit_application_query(
                        &phase,
                        &query,
                        &access,
                        ApplicationQueryParameterSet::new(),
                        primary_graph::WorthQueryProductQueryControls::new(
                            maximum_results,
                            &request,
                        ),
                    )
                    .unwrap()
            })
            .expect("installed host call opens")
    };
    world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .execute_application_query_one_shot_in_batch(&phase, plan, batch)
        })
        .expect("installed host call opens")
}
