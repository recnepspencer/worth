use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use worth_query_host::facade::{
    admission::authenticated_principal::{
        WorthQueryAuthenticatedExternalPrincipal, WorthQueryCancellationSource,
        WorthQueryRequestScope,
    },
    declaration::application_query::{ApplicationQueryIntent, ApplicationQueryScopeResolution},
    primary_graph::{
        WorthQueryAdmittedOutputDemand, WorthQueryApplicationOutputDemandSource,
        WorthQueryApplicationQueryAccessContext, WorthQueryOutputDemandAdvance,
        WorthQueryOutputDemandDenialKind, WorthQueryPrimaryGraphApplicationRuntime,
        WorthQueryPrincipalResolutionMode, WorthQueryProductQueryControls,
    },
};
use worth_query_topology_entry::{
    PlanarOutputFamily, PlanarQuery, PlanarRead, PlanarReadBinding, PlanarReadResult,
};

use super::super::{authentication, installation};
use crate::ConsumerSchema;

type Application = WorthQueryPrimaryGraphApplicationRuntime<ConsumerSchema>;
type Admitted = WorthQueryAdmittedOutputDemand<ConsumerSchema, PlanarOutputFamily>;

mod producer_lifecycle;

pub(super) fn producer_lifecycle_probe(
    foreign_schema: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    producer_lifecycle::live_output_and_stale_head_selection(foreign_schema);
}

pub(super) fn raw_owner_guards(
    foreign_schema: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    sibling_target_disclosure_is_denied(foreign_schema);
    principal_and_scope_are_revalidated(foreign_schema);
    close_is_idempotent_and_terminal(foreign_schema);
    denied_executor_releases_shared_claim(foreign_schema);
}

fn sibling_target_disclosure_is_denied(
    foreign_schema: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    let world = installation::install(foreign_schema);
    let scope = authentication::request_scope();
    let principal = authenticate(&world.application, &scope);
    let mut owner = admit(
        &world.application,
        source(&world.application, &principal, &scope, "anchor-a"),
    );
    let mut peer = admit(
        &world.application,
        source(&world.application, &principal, &scope, "anchor-a"),
    );
    settle(
        &world.application,
        &principal,
        &scope,
        &mut owner,
        "anchor-a",
    );

    let denial = match world.application.advance_output_demand(
        &mut peer,
        &principal,
        &scope,
        world.application.current_world(),
        source(&world.application, &principal, &scope, "anchor-b"),
    ) {
        Ok(_) => panic!("a sibling target consumed the settled target's owner record"),
        Err(denial) => denial,
    };
    assert_eq!(denial.kind(), WorthQueryOutputDemandDenialKind::Superseded);
    owner.close();
    peer.close();
}

fn principal_and_scope_are_revalidated(
    foreign_schema: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    let world = installation::install(foreign_schema);
    let scope = authentication::request_scope();
    let principal = authenticate(&world.application, &scope);
    let mut owner = admit(
        &world.application,
        source(&world.application, &principal, &scope, "anchor-a"),
    );
    let mut foreign_peer = admit(
        &world.application,
        source(&world.application, &principal, &scope, "anchor-a"),
    );
    settle(
        &world.application,
        &principal,
        &scope,
        &mut owner,
        "anchor-a",
    );

    let other = installation::install(foreign_schema);
    let foreign_principal = authenticate(&other.application, &scope);
    let local_disclosure = source(&world.application, &principal, &scope, "anchor-a");
    let denial = expect_denial(
        world.application.advance_output_demand(
            &mut foreign_peer,
            &foreign_principal,
            &scope,
            world.application.current_world(),
            local_disclosure,
        ),
        "a foreign principal consumed the admitted source",
    );
    assert_eq!(denial.kind(), WorthQueryOutputDemandDenialKind::Superseded);

    let other_scope = authentication::request_scope();
    let wrong_scope_disclosure = source(&world.application, &principal, &scope, "anchor-a");
    let denial = expect_denial(
        world.application.advance_output_demand(
            &mut foreign_peer,
            &principal,
            &other_scope,
            world.application.current_world(),
            wrong_scope_disclosure,
        ),
        "a different live request scope consumed the disclosure",
    );
    assert_eq!(denial.kind(), WorthQueryOutputDemandDenialKind::Superseded);

    let cancellation = WorthQueryCancellationSource::new();
    let cancelled_scope = WorthQueryRequestScope::new(
        Instant::now() + Duration::from_secs(30),
        cancellation.token(),
    );
    let mut cancelled_peer = admit(
        &world.application,
        source(&world.application, &principal, &cancelled_scope, "anchor-a"),
    );
    let cancelled_disclosure = source(&world.application, &principal, &cancelled_scope, "anchor-a");
    cancellation.cancel();
    let denial = expect_denial(
        world.application.advance_output_demand(
            &mut cancelled_peer,
            &principal,
            &cancelled_scope,
            world.application.current_world(),
            cancelled_disclosure,
        ),
        "a cancelled exact request scope advanced demand",
    );
    assert_eq!(denial.kind(), WorthQueryOutputDemandDenialKind::Superseded);
    owner.close();
    foreign_peer.close();
    cancelled_peer.close();
}

fn close_is_idempotent_and_terminal(
    foreign_schema: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    let world = installation::install(foreign_schema);
    let scope = authentication::request_scope();
    let principal = authenticate(&world.application, &scope);
    let mut admitted = admit(
        &world.application,
        source(&world.application, &principal, &scope, "anchor-a"),
    );
    admitted.close();
    admitted.close();
    let notification = match admitted.notifications() {
        Ok(_) => panic!("a closed owner exposed notifications"),
        Err(denial) => denial,
    };
    assert_eq!(
        notification.kind(),
        WorthQueryOutputDemandDenialKind::Closed
    );
    let denial = match world.application.advance_output_demand(
        &mut admitted,
        &principal,
        &scope,
        world.application.current_world(),
        source(&world.application, &principal, &scope, "anchor-a"),
    ) {
        Ok(_) => panic!("a closed owner advanced"),
        Err(denial) => denial,
    };
    assert_eq!(denial.kind(), WorthQueryOutputDemandDenialKind::Closed);
}

fn denied_executor_releases_shared_claim(
    foreign_schema: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ConsumerSchema,
    >,
) {
    let world = installation::install(foreign_schema);
    let scope = authentication::request_scope();
    let principal = authenticate(&world.application, &scope);
    let mut denied = admit(
        &world.application,
        source(&world.application, &principal, &scope, "anchor-a"),
    );
    let mut peer = admit(
        &world.application,
        source(&world.application, &principal, &scope, "anchor-a"),
    );
    world
        .producer_authorization_denials
        .store(1, Ordering::SeqCst);
    let mut observed_denial = None;
    for _ in 0..4 {
        match world.application.advance_output_demand(
            &mut denied,
            &principal,
            &scope,
            world.application.current_world(),
            source(&world.application, &principal, &scope, "anchor-a"),
        ) {
            Ok(WorthQueryOutputDemandAdvance::Pending) => {}
            Ok(WorthQueryOutputDemandAdvance::Settled(_)) => {
                panic!("the authorization-denied interest unexpectedly settled")
            }
            Err(denial) => {
                observed_denial = Some(denial);
                break;
            }
        }
    }
    let denial = observed_denial.expect("the injected authorization denial was observed");
    assert_eq!(
        denial.kind(),
        WorthQueryOutputDemandDenialKind::ProducerUnavailable
    );
    settle(
        &world.application,
        &principal,
        &scope,
        &mut peer,
        "anchor-a",
    );
    denied.close();
    peer.close();
}

fn authenticate<'a>(
    application: &Application,
    scope: &'a WorthQueryRequestScope,
) -> WorthQueryAuthenticatedExternalPrincipal<ConsumerSchema> {
    let adapter = authentication::admit(application.installed_schema());
    authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        scope,
    ))
    .expect("the owner-boundary proof authenticates its principal")
}

fn source(
    application: &Application,
    principal: &WorthQueryAuthenticatedExternalPrincipal<ConsumerSchema>,
    request_scope: &WorthQueryRequestScope,
    body_key: &str,
) -> WorthQueryApplicationOutputDemandSource<PlanarQuery, PlanarReadResult> {
    let intent = PlanarRead {
        body_key: body_key.to_owned(),
    };
    let binding = application
        .installed_schema()
        .installed_query_binding::<PlanarReadBinding<ConsumerSchema>>()
        .expect("the source query binding is installed");
    let selected = application
        .on_branch(application.current_world())
        .select()
        .expect("the current program occurrence is selectable");
    let resolved_principal = selected
        .resolve_authenticated_principal(
            binding.principal_binding(),
            principal,
            request_scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .expect("the principal resolves for the source query");
    let (scope_field, scope_value) = intent
        .clone()
        .into_scope()
        .into_field_parts(resolved_principal.principal_identity());
    let scope = selected
        .resolve_entity(
            scope_field,
            scope_value,
            request_scope,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .expect("the source occurrence resolves");
    let access = WorthQueryApplicationQueryAccessContext::new(&resolved_principal, &scope);
    let plan = selected
        .admit_application_query(
            binding.query(),
            &access,
            <PlanarRead as ApplicationQueryIntent<ConsumerSchema>>::parameters(&intent),
            WorthQueryProductQueryControls::new(
                binding.limits().maximum_results(),
                binding.limits().maximum_work(),
                request_scope,
            ),
        )
        .expect("the source query admits");
    application
        .execute_application_query_one_shot(plan)
        .expect("the source query executes")
        .into_admitted_disclosed()
        .into_output_demand_source()
}

fn admit(
    application: &Application,
    source: WorthQueryApplicationOutputDemandSource<PlanarQuery, PlanarReadResult>,
) -> Admitted {
    application
        .admit_output_demand::<PlanarOutputFamily>(source, 4_096, 8_192)
        .expect("the raw owner demand admits")
}

fn settle(
    application: &Application,
    principal: &WorthQueryAuthenticatedExternalPrincipal<ConsumerSchema>,
    scope: &WorthQueryRequestScope,
    admitted: &mut Admitted,
    body_key: &str,
) {
    for _ in 0..8 {
        let progress = application
            .advance_output_demand(
                admitted,
                principal,
                scope,
                application.current_world(),
                source(application, principal, scope, body_key),
            )
            .expect("the owner output advances");
        if matches!(progress, WorthQueryOutputDemandAdvance::Settled(_)) {
            return;
        }
    }
    panic!("the owner output did not settle within its synchronous bound");
}

fn expect_denial(
    result: Result<
        WorthQueryOutputDemandAdvance,
        worth_query_host::facade::primary_graph::WorthQueryOutputDemandDenial,
    >,
    success: &str,
) -> worth_query_host::facade::primary_graph::WorthQueryOutputDemandDenial {
    match result {
        Ok(_) => panic!("{success}"),
        Err(denial) => denial,
    }
}
