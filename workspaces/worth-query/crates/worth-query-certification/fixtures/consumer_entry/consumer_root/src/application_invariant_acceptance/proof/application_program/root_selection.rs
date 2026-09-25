use worth_query_host::facade::application_entry::WorthQueryApplicationRequestExt;
use worth_query_topology_entry::{PlanarRead, PlanarSourceAdjustment};

use super::super::super::{authentication, installation, seed::length};
use crate::ConsumerSchema;

pub(in crate::application_invariant_acceptance::proof) fn result_set_source_binds_through_publication(
    foreign: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    let world = installation::install(foreign);
    let scope = authentication::request_scope();
    let adapter = authentication::admit(world.application.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .expect("the application authenticates its principal");
    let request = world.application.request(&principal, &scope);
    let observed = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .expect("the result set is observed through the public query path");
    assert_eq!(observed.rows().len(), 1);
    let source = observed
        .into_result_set_observation()
        .expect("one-shot publication retains the result-set proof");
    let outcome = request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".to_owned(),
            replacement_y: length(2),
        })
        .expect_result_set(source)
        .idempotency(&10_009)
        .execute_performed::<crate::ConsumerProgram, crate::ConsumerProgramRoot>(&world.application)
        .expect("the result-set source is admissible for the declared query");
    assert!(matches!(
        outcome,
        worth_query_host::facade::application_entry::WorthQueryApplicationPerformedMutationOutcome::Performed(_)
    ));
    assert_eq!(
        request
            .query(PlanarRead {
                body_key: "anchor-a".to_owned(),
            })
            .execute()
            .unwrap()
            .rows()[0]
            .y,
        length(2)
    );
}

pub(in crate::application_invariant_acceptance::proof) fn foreign_program_is_denied_before_publication(
    foreign: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    let source_world = installation::install(foreign);
    let other_world = installation::install(foreign);
    let scope = authentication::request_scope();
    let adapter = authentication::admit(source_world.application.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .expect("the source application authenticates its principal");
    let request = source_world.application.request(&principal, &scope);
    let before = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .expect("the source occurrence is readable");
    let source = before.observed_sources()[0].clone();
    let outcome = request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".to_owned(),
            replacement_y: length(2),
        })
        .expect_source(source)
        .idempotency(&10_004)
        .execute_performed::<crate::ConsumerProgram, crate::ConsumerProgramRoot>(
            &other_world.application,
        );
    let Err(denial) = outcome else {
        panic!("foreign program meaning must be denied before source publication")
    };
    assert!(matches!(
        denial,
        worth_query_host::facade::application_entry::WorthQueryPerformedMutationExecutionDenial::ForeignProgram
    ));
    let after = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .expect("the source remains readable after denial");
    assert_eq!(after.rows()[0].y, length(1));
}

pub(in crate::application_invariant_acceptance::proof) fn undeclared_root_is_denied_before_publication(
    foreign: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    let world = installation::install(foreign);
    let scope = authentication::request_scope();
    let adapter = authentication::admit(world.application.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .expect("the application authenticates its principal");
    let request = world.application.request(&principal, &scope);
    let before = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .expect("the source occurrence is readable");
    let source = before.observed_sources()[0].clone();
    let denial = request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".to_owned(),
            replacement_y: length(2),
        })
        .expect_source(source)
        .idempotency(&10_007)
        .execute_performed::<crate::ConsumerProgram, crate::ConsumerUndeclaredProgramRoot>(
            &world.application,
        );
    let Err(denial) = denial else {
        panic!("a required connection absent from the installed root set must be denied")
    };
    assert!(matches!(
        denial,
        worth_query_host::facade::application_entry::WorthQueryPerformedMutationExecutionDenial::UndeclaredOutputRoot
    ));
    assert_eq!(
        request
            .query(PlanarRead {
                body_key: "anchor-a".to_owned(),
            })
            .execute()
            .expect("the denied root leaves its source readable")
            .rows()[0]
            .y,
        length(1)
    );
}

pub(in crate::application_invariant_acceptance::proof) fn truncated_root_is_denied_before_publication(
    foreign: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    let world = installation::install(foreign);
    let scope = authentication::request_scope();
    let adapter = authentication::admit(world.application.installed_schema());
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .expect("the application authenticates its principal");
    let request = world.application.request(&principal, &scope);
    let source = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .expect("the source occurrence is readable")
        .observed_sources()[0]
        .clone();
    let outcome = request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".to_owned(),
            replacement_y: length(2),
        })
        .expect_source(source)
        .idempotency(&10_008)
        .execute_performed::<crate::ConsumerProgram, crate::ConsumerTruncatedProgramRoot>(
            &world.application,
        );
    let Err(denial) = outcome else {
        panic!("a caller cannot truncate a declared root's dependent graph")
    };
    assert!(matches!(
        denial,
        worth_query_host::facade::application_entry::WorthQueryPerformedMutationExecutionDenial::UndeclaredOutputRoot
    ));
    assert_eq!(
        request
            .query(PlanarRead {
                body_key: "anchor-a".to_owned(),
            })
            .execute()
            .expect("the denied truncated root leaves its source readable")
            .rows()[0]
            .y,
        length(1)
    );
}
