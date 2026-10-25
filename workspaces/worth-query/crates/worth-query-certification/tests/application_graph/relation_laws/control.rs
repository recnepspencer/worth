//! The original public request stops before any candidate becomes committed truth.
use super::{installation::*, read, schema::*};
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_contribution::{
        WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
    },
    application_entry::{
        WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestMutationDenial,
    },
};

#[test]
fn cancelled_request_preserves_truth_and_a_fresh_request_can_complete() {
    let world = install::<Acyclic>(3, &[(0, 1)]);
    let before = read::committed(&world);
    let cancellation = WorthQueryCancellationSource::new();
    let stopped = WorthQueryRequestScope::new(None, cancellation.token());
    cancellation.cancel();
    assert!(matches!(
        world.edit_in_scope(1, &[(1, 2)], &stopped),
        Err(
            WorthQueryApplicationRequestMutationDenial::ExecutionRequest(
                WorthQueryAdvancementDenial::Interrupted(
                    WorthQueryManagedComputationInterruption::Cancelled
                )
            )
        )
    ));
    assert_eq!(read::committed(&world).edges, before.edges);
    assert!(matches!(
        world.edit(1, &[(1, 2)]),
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
    let mut expected = before.edges;
    expected.insert((1, 2));
    assert_eq!(read::committed(&world).edges, expected);
    world.close();
}
