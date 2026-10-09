//! Native installation and authentication for the two physical reservation tests.

#[path = "candidate_retained_representation_fixture/bindings.rs"]
mod bindings;
#[path = "candidate_retained_representation_fixture/schema.rs"]
mod schema;
use crate::domain_computation::execution_runtime::WorthQueryExecutionRuntimeInstaller;
use crate::domain_computation::primary_graph::tests::fixture::{
    authenticate_external, external_identity,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationEntityIdentity, WorthQueryApplicationEntityKey,
    WorthQueryApplicationEntitySeed, WorthQueryApplicationInvariantProjectionAuthority,
    WorthQueryApplicationPrincipalKey, WorthQueryApplicationRelationSeed,
    WorthQueryAuthenticatedPrincipal, WorthQueryPrimaryGraphApplicationRuntime,
    WorthQueryPrincipalResolutionMode, WorthQuerySelectedProductOperation,
};
pub(super) use schema::*;
use std::time::Duration;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_declaration::facade::authentication::WorthQueryPrincipalMappingStatus;
use worth_query_installation::facade::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackage,
};

pub(super) struct ReservationWorld {
    pub(super) application: WorthQueryPrimaryGraphApplicationRuntime<ReservationSchema>,
    pub(super) invariant: WorthQueryApplicationInvariantProjectionAuthority<ReservationSchema>,
}

impl ReservationWorld {
    pub(super) fn selected_product(
        &self,
    ) -> WorthQuerySelectedProductOperation<'_, ReservationSchema> {
        self.application
            .select_product_branch(self.application.product_runtime().default_branch())
            .expect("the fixture Product branch remains admitted")
    }
}

pub(super) fn installed_world() -> ReservationWorld {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = active_phase.bootstrap_for_test();
        let declaration = ReservationSchema::declaration().unwrap();
        let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
            "candidate_retained_representation_test",
            1,
            0,
        ))
        .application_schema(declaration.clone())
        .validate()
        .unwrap();
        let admitted = WorthQueryInstallationAdmissionProfile::new("support", "configuration")
            .admit(package)
            .unwrap();
        let (runtime, authority) = WorthQueryExecutionRuntimeInstaller::new()
            .install(WorthQueryInstallationGeneration::initial(), [admitted])
            .unwrap()
            .into_parts();
        let schema = runtime
            .installed_packages()
            .bind_application_schema(declaration)
            .unwrap();
        let binding = schema
            .principal_binding(IdentityBinding::reference())
            .unwrap();
        let mut bootstrap = authority.prepare_primary_graph(
        &phase, &runtime, &schema,
        crate::domain_computation::execution_runtime::product_world::test_product_world_resources(),
    ).unwrap();
        bootstrap
            .bind_principal(
                &binding,
                WorthQueryApplicationPrincipalKey::new("principal-0").unwrap(),
                1_u64,
                external_identity("alice"),
                WorthQueryPrincipalMappingStatus::Enabled,
            )
            .unwrap();
        bootstrap
            .bind_entity(
                WorthQueryApplicationEntitySeed::new(
                    Account::reference(),
                    WorthQueryApplicationEntityKey::new("account-1").unwrap(),
                )
                .field(AccountStatus::reference(), "open".to_owned()),
            )
            .unwrap();
        bootstrap
            .bind_relation(WorthQueryApplicationRelationSeed::new(
                AccountOwner::reference(),
                "owner-1",
                WorthQueryApplicationEntityKey::new("principal-0").unwrap(),
                WorthQueryApplicationEntityKey::new("account-1").unwrap(),
            ))
            .unwrap();
        bindings::install_handlers(&schema, &mut bootstrap);
        let invariant = bootstrap.retain_invariant_projection_authority();
        let application = bootstrap
            .publish_application_runtime(
                &phase,
                runtime,
                authority,
                schema,
                worth_signal::facade::runtime::SignalConditionalEvaluationBudget::development(),
            )
            .unwrap();
        ReservationWorld {
            application,
            invariant,
        }
    })
}

pub(super) fn authenticated_principal(
    world: &ReservationWorld,
    request: &WorthQueryRequestScope,
) -> WorthQueryAuthenticatedPrincipal<ReservationSchema, Principal, u64> {
    let schema = world.application.installed_schema();
    let external = authenticate_external(schema, "alice", Duration::from_secs(60), request);
    let binding = schema
        .principal_binding(IdentityBinding::reference())
        .unwrap();
    world
        .selected_product()
        .resolve_authenticated_principal(
            &binding,
            &external,
            request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
}

pub(super) fn resolved_account(
    world: &ReservationWorld,
    status: &str,
    request: &WorthQueryRequestScope,
) -> WorthQueryApplicationEntityIdentity<ReservationSchema, Account> {
    world
        .selected_product()
        .resolve_entity(
            AccountStatus::reference(),
            status.to_owned(),
            request,
            WorthQueryPrincipalResolutionMode::Ordinary,
        )
        .unwrap()
}
