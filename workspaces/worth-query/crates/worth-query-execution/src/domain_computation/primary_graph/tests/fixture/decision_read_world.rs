//! A separate world whose one operation declares a managed-index decision
//! read, so the decision-read entry's own branch is reachable. Only the
//! decision-entry forgery court installs it.

#[path = "decision_read_world/operation.rs"]
mod operation;
#[path = "decision_read_world/provider.rs"]
mod provider;
#[path = "decision_read_world/queries.rs"]
mod queries;
#[path = "decision_read_world/schema.rs"]
mod schema;

pub(in crate::domain_computation) use operation::{DecisionReadBinding, DecisionReadOperation};
pub(in crate::domain_computation) use schema::{DecisionReadSchema, Principal};

use super::{authenticate_external, external_identity};
use crate::domain_computation::execution_runtime::WorthQueryExecutionRuntimeInstaller;
use crate::domain_computation::primary_graph::{
    TestHostRequest, WorthQueryAdmittedApplicationOperation, WorthQueryApplicationEntityIdentity,
    WorthQueryApplicationPrincipalKey, WorthQueryAuthenticatedPrincipal,
    WorthQueryPrimaryGraphApplicationRuntime, WorthQueryPrincipalResolutionMode,
    WorthQuerySelectedProductOperation,
};
use std::time::Duration;
use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryRequestScope,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationSchema, U64ApplicationValueBinding,
};
use worth_query_declaration::facade::authentication::WorthQueryPrincipalMappingStatus;
use worth_query_installation::facade::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryInstalledPrincipalBinding, WorthQueryPortableDomainIdentity,
    WorthQueryPortableDomainPackage,
};

pub(in crate::domain_computation) type DecisionReadAdmission =
    WorthQueryAdmittedApplicationOperation<
        DecisionReadSchema,
        DecisionReadOperation,
        (),
        Principal,
    >;
type Binding = WorthQueryInstalledPrincipalBinding<
    DecisionReadSchema,
    schema::DecisionPrincipalBinding,
    schema::Mapping,
    Principal,
    u64,
    U64ApplicationValueBinding,
>;

/// The world's one bound principal, which is also its scope.
pub(in crate::domain_computation) const PRINCIPAL: u64 = 1;

pub(in crate::domain_computation) struct DecisionReadWorld {
    pub(in crate::domain_computation) application:
        WorthQueryPrimaryGraphApplicationRuntime<DecisionReadSchema>,
    binding: Binding,
}

pub(in crate::domain_computation) fn installed_decision_read_world() -> DecisionReadWorld {
    crate::domain_computation::primary_graph::with_test_advancement(|phase| {
        let declaration = DecisionReadSchema::declaration().unwrap();
        let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
            <DecisionReadSchema as ApplicationSchema>::OWNER,
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
            .principal_binding(schema::DecisionPrincipalBinding::reference())
            .unwrap();
        let mut bootstrap = authority
            .prepare_primary_graph(
                &phase.bootstrap_for_test(),
                &runtime,
                &schema,
                crate::domain_computation::execution_runtime::product_world::test_product_world_resources(worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system()),
            )
            .unwrap();
        bootstrap
            .bind_principal(
                &binding,
                WorthQueryApplicationPrincipalKey::new("principal-0").unwrap(),
                PRINCIPAL,
                external_identity("alice"),
                WorthQueryPrincipalMappingStatus::Enabled,
            )
            .unwrap();
        bootstrap
            .managed_indexes
            .register::<queries::DecisionIndex, _>(
                &schema,
                schema::MEMBERS,
                provider::DecisionIndexProvider,
                None,
            )
            .unwrap();
        let decision = schema
            .installed_mutation_binding::<DecisionReadBinding>()
            .unwrap();
        bootstrap
            .install_handler(&decision, operation::DecisionReadHandler)
            .unwrap();
        let application = bootstrap
            .publish_application_runtime(
                &phase.bootstrap_for_test(),
                runtime,
                authority,
                schema,
                worth_signal::facade::runtime::SignalConditionalEvaluationBudget::development(),
            )
            .unwrap();
        DecisionReadWorld {
            application,
            binding,
        }
    })
}

impl DecisionReadWorld {
    pub(in crate::domain_computation) fn authenticate(
        &self,
        request: &WorthQueryRequestScope,
    ) -> WorthQueryAuthenticatedExternalPrincipal<DecisionReadSchema> {
        authenticate_external(
            self.application.installed_schema(),
            "alice",
            Duration::from_secs(60),
            request,
        )
    }

    /// Mints the decision-read operation for the bound principal on its own scope.
    pub(in crate::domain_computation) fn admit(
        &self,
        request: &WorthQueryRequestScope,
    ) -> DecisionReadAdmission {
        let operation = self
            .application
            .installed_schema()
            .installed_operation(DecisionReadOperation::reference())
            .unwrap();
        self.application
            .with_host_advancement(|phase| {
                let host = phase.test_host_request();
                let (principal, scope) = self.party(&host, request);
                self.selected(&host)
                    .authorize_operation(
                        &phase,
                        &principal,
                        &scope,
                        &operation,
                        Default::default(),
                        request,
                    )
                    .unwrap()
            })
            .expect("the fixture owns its installed host call")
    }

    fn party(
        &self,
        host: &TestHostRequest<'_>,
        request: &WorthQueryRequestScope,
    ) -> (
        WorthQueryAuthenticatedPrincipal<DecisionReadSchema, Principal, u64>,
        WorthQueryApplicationEntityIdentity<DecisionReadSchema, Principal>,
    ) {
        let phase = host.phase_for_application(&self.application);
        let principal = self
            .selected(host)
            .resolve_authenticated_principal(
                &phase,
                &self.binding,
                &self.authenticate(request),
                request,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap();
        let scope = self
            .selected(host)
            .resolve_entity(
                &phase,
                schema::PrincipalIdentity::reference(),
                PRINCIPAL,
                request,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap();
        (principal, scope)
    }

    fn selected(
        &self,
        host: &TestHostRequest<'_>,
    ) -> WorthQuerySelectedProductOperation<'_, DecisionReadSchema> {
        let execution = host
            .execution_request_for(self.application.product_runtime())
            .expect("the fixture lends its installed phase");
        self.application
            .select_product_branch(
                &execution,
                self.application.product_runtime().default_branch(),
            )
            .expect("the fixture product branch remains admitted")
    }
}
