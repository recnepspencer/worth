use std::sync::Arc;

use worth_query_host::facade::{admission, declaration, domain, primary_graph, runtime};

use super::adapters::{
    ClockController, ClockSource, ExampleClock, IdentityAdapter, IntentProjector, Invoker,
    Predicate, PrincipalSource,
};
use super::contract::{self, TemporalReadyNode};
use super::schema::*;

pub struct ExampleApplication {
    pub runtime: primary_graph::WorthQueryPrimaryGraphApplicationRuntime<TemporalHostSchema>,
    pub clock: primary_graph::WorthQueryConditionalClockHandle<
        TemporalHostSchema,
        TemporalReadyNode,
        ExampleClock,
    >,
    pub clock_control: ClockController,
    pub invariant:
        Arc<primary_graph::WorthQueryApplicationInvariantProjectionAuthority<TemporalHostSchema>>,
}

impl ExampleApplication {
    pub fn publish(gate: &str) -> Self {
        let declaration = TemporalHostSchema::declaration().expect("the example schema is valid");
        let conditional_binding = contract::conditional_binding();
        let package = domain::WorthQueryPortableDomainPackage::new(
            domain::WorthQueryPortableDomainIdentity::new("temporal_host_courtroom", 1, 0),
        )
        .application_schema(declaration.clone())
        .domain_operation(contract::operation_definition().into_portable())
        .conditional_application_operation(conditional_binding.clone())
        .validate()
        .expect("the example package is valid");
        let admitted = domain::WorthQueryInstallationAdmissionProfile::new("host", "example")
            .admit(package)
            .expect("the example package is admissible");
        let installation = runtime::WorthQueryExecutionRuntimeInstaller::new()
            .install(
                domain::WorthQueryInstallationGeneration::initial(),
                [admitted],
            )
            .expect("the example package must install");
        let (runtime, authority) = installation.into_parts();
        let schema = runtime
            .installed_packages()
            .bind_application_schema(declaration)
            .expect("the installed schema must bind");
        let principal_binding = schema
            .principal_binding(TemporalPrincipalBinding::reference())
            .expect("the principal binding must install");
        let authentication = admit_identity_adapter(&schema);
        let operation = schema
            .installed_operation(ExecuteTemporal::reference())
            .expect("the temporal operation must install");
        let query = schema
            .application_query(TemporalIntentQuery::reference())
            .expect("the temporal query must install");
        let (clock_source, clock_control) = ClockSource::due();
        let conditional = runtime
            .installed_packages()
            .bind_conditional_application_operation(operation, &conditional_binding)
            .expect("the conditional operation must bind")
            .bind_node(TemporalReadyNode::reference())
            .expect("the conditional node must bind")
            .bind_host_predicate_provider(Predicate)
            .expect("the predicate must bind")
            .bind_named_clock::<ExampleClock, _>(clock_source)
            .expect("the clock must bind")
            .bind_temporal_intent_projection(
                query,
                declaration::application_query::ApplicationQueryParameterSet::new(),
                IntentProjector,
                domain::WorthQueryTemporalIntentBounds::new(8, 8, 8)
                    .expect("the temporal bounds are valid"),
            )
            .expect("the temporal projection must bind");
        let mut graph = authority
            .prepare_primary_graph(&runtime, &schema, product_world_resources())
            .expect("the production graph and World owners must prepare");
        seed_graph(&mut graph, &principal_binding, gate);
        let invariant = Arc::new(graph.retain_invariant_projection_authority());
        let execution = primary_graph::WorthQueryTemporalOperationExecution::with_authorization(
            Arc::clone(&invariant),
            Invoker,
            IntentIdentityField::reference(),
            IntentRevisionField::reference(),
            IntentLifecycleField::reference(),
            "active".to_owned(),
            "completed".to_owned(),
            primary_graph::WorthQueryPublicTemporalOperationAuthorization,
        )
        .expect("the temporal invocation must bind");
        let reconstruction = primary_graph::WorthQueryTemporalReconstructionAccess::new(
            principal_binding,
            PrincipalSource::new(authentication),
            IntentIdentityField::reference(),
            "intent-1".to_owned(),
        )
        .expect("the temporal reconstruction source must bind");
        let mut conditional_installation = graph
            .conditional_application_runtime_installation(
                runtime,
                authority,
                schema,
                primary_graph::SignalConditionalEvaluationBudget::development(),
            )
            .expect("the conditional runtime must prepare");
        let clock = conditional_installation
            .bind_temporal_operation(conditional, execution, reconstruction)
            .expect("the temporal operation must bind to its clock");
        let runtime = conditional_installation
            .publish()
            .expect("the application runtime must publish");
        Self {
            runtime,
            clock,
            clock_control,
            invariant,
        }
    }
}

pub fn admit_identity_adapter(
    schema: &domain::WorthQueryInstalledApplicationSchema<TemporalHostSchema>,
) -> admission::authenticated_principal::WorthQueryAdmittedAuthenticationAdapter<
    TemporalHostSchema,
    IdentityAdapter,
> {
    admission::authenticated_principal::admit_authentication_adapter(
        schema,
        admission::authenticated_principal::WorthQueryAuthenticationAdapterAdmission::new(
            admission::authenticated_principal::WorthQueryAuthenticationAudience::new("host")
                .expect("the example audience is valid"),
            admission::authenticated_principal::WorthQueryAuthenticationMethod::new("example")
                .expect("the example method is valid"),
        ),
        IdentityAdapter,
    )
    .expect("the example identity adapter must be admitted")
}

fn seed_graph(
    graph: &mut primary_graph::WorthQueryPrimaryGraphBootstrap<TemporalHostSchema>,
    principal_binding: &domain::WorthQueryInstalledPrincipalBinding<
        TemporalHostSchema,
        TemporalPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
    >,
    gate: &str,
) {
    graph
        .bind_principal(
            principal_binding,
            primary_graph::WorthQueryApplicationPrincipalKey::new("product-example")
                .expect("the principal key is valid"),
            1_u64,
            declaration::authentication::WorthQueryExternalPrincipalIdentity::new(
                "https://issuer.example",
                "product-example",
            )
            .expect("the external identity is valid"),
            declaration::authentication::WorthQueryPrincipalMappingStatus::Enabled,
        )
        .expect("the principal must seed");
    graph
        .bind_entity(
            primary_graph::WorthQueryApplicationEntitySeed::new(
                TemporalIntent::reference(),
                primary_graph::WorthQueryApplicationEntityKey::new("intent-row-1")
                    .expect("the row key is valid"),
            )
            .field(IntentIdentityField::reference(), "intent-1".to_owned())
            .field(IntentRevisionField::reference(), 1_u64)
            .field(IntentDueField::reference(), 5_u64)
            .field(IntentLifecycleField::reference(), "active".to_owned())
            .field(IntentInputField::reference(), "payload".to_owned())
            .field(IntentGateField::reference(), gate.to_owned())
            .field(IntentEffectField::reference(), "pending".to_owned()),
        )
        .expect("the intent must seed");
}

fn product_world_resources() -> runtime::WorthQueryProductWorldResources {
    runtime::WorthQueryProductWorldResources::install(
        runtime::RuntimeWorldBudgetInstallation {
            branches: runtime::RuntimeWorldBranchBudgetInstallation {
                live_product_branches: 128,
            },
            history: runtime::RuntimeWorldHistoryBudgetInstallation {
                retained_composite_commits: 1_024,
                history_metadata_bytes: 16 * 1024 * 1024,
            },
            observations: runtime::RuntimeWorldObservationBudgetInstallation {
                active_observations: 512,
            },
            publication: runtime::RuntimeWorldPublicationBudgetInstallation {
                active_publication_attempts: 128,
            },
            recovery: runtime::RuntimeWorldRecoveryBudgetInstallation {
                retained_product_unpublished_records: 128,
                retained_partial_metadata_bytes: 16 * 1024 * 1024,
            },
            retention: runtime::RuntimeWorldRetentionBudgetInstallation {
                unique_exact_component_pins: 1_024,
                in_flight_pin_acquisition_reservations: 256,
            },
            custody: runtime::RuntimeWorldCustodyBudgetInstallation {
                owner_created_component_custody_records: 256,
            },
        },
        runtime::WorthQueryProductWorldClock::start(),
    )
    .expect("the example World resources are valid")
}
