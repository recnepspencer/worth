//! The branch-selected performed lane refuses what the selected program does
//! not own before any effect, and no refusal claims its idempotency key.
//!
//! A foreign program runtime and a root the installed shape never declared
//! are refused before publication. After each refusal the same key performs on
//! a legal lane. The commit-time check that the branch's program still owns
//! the root is defense in depth here: adopting a successor that drops an
//! output needs a migration, and every consumer binding produces output, which
//! a migration may not.

use worth_query_host::facade::application_entry::{
    WorthQueryApplicationPerformedMutationOutcome, WorthQueryApplicationRequest,
    WorthQueryApplicationRequestExt, WorthQueryPerformedMutationExecutionDenial,
};
use worth_query_topology_entry::{PlanarRead, PlanarSourceAdjustment};

use super::super::super::{authentication, installation, seed::length};
use crate::{ConsumerProgram, ConsumerProgramRoot, ConsumerSchema, ConsumerUndeclaredProgramRoot};

type Request<'a> = WorthQueryApplicationRequest<'a, 'a, 'a, ConsumerSchema>;

/// Runs the selected performed lane for one adjustment of `anchor-a`.
macro_rules! selected {
    ($request:expr, $application:expr, $root:ty, $key:expr, $y:expr) => {{
        let source = $request
            .query(PlanarRead {
                body_key: "anchor-a".to_owned(),
            })
            .execute()
            .expect("the source occurrence is readable")
            .observed_sources()[0]
            .clone();
        $request
            .mutate(PlanarSourceAdjustment {
                scope_key: "anchor-a".to_owned(),
                replacement_y: length($y),
            })
            .expect_source(source)
            .idempotency(&$key)
            .execute_performed_in_selected_program::<ConsumerProgram, $root>($application)
    }};
}

pub(in crate::application_invariant_acceptance::proof) fn selected_lane_refuses_what_the_selected_program_does_not_own(
    foreign: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    let world = installation::install(foreign);
    let other = installation::install(foreign);
    let scope = authentication::request_scope();
    let adapter = authentication::admit(world.application.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .expect("the application authenticates its principal");
    let request = world.application.request(&principal, &scope);

    let foreign_runtime = selected!(
        request,
        &other.application,
        ConsumerProgramRoot,
        10_101_u64,
        2
    );
    assert!(matches!(
        foreign_runtime,
        Err(WorthQueryPerformedMutationExecutionDenial::ForeignProgram)
    ));
    let undeclared = selected!(
        request,
        &world.application,
        ConsumerUndeclaredProgramRoot,
        10_102_u64,
        2
    );
    assert!(matches!(
        undeclared,
        Err(WorthQueryPerformedMutationExecutionDenial::UndeclaredOutputRoot)
    ));
    assert_eq!(
        read_y(&request),
        length(1),
        "both refusals precede any effect"
    );
    for (key, y) in [(10_101_u64, 2), (10_102_u64, 3)] {
        let retried = selected!(request, &world.application, ConsumerProgramRoot, key, y);
        assert!(
            matches!(
                retried,
                Ok(WorthQueryApplicationPerformedMutationOutcome::Performed(_))
            ),
            "a refused key {key} must remain unclaimed"
        );
        assert_eq!(read_y(&request), length(y));
    }
}

fn read_y(request: &Request<'_>) -> worth_query_consumer_values::PositiveLength {
    request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .expect("the source occurrence is readable")
        .rows()[0]
        .y
}
