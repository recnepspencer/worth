use std::sync::Arc;

use worth_query_host::facade::{
    admission, application_installation, declaration, domain, primary_graph, runtime,
};

use super::adapters::{ClockController, ClockSource, IdentityAdapter};
use super::conditional_contribution::{
    InstalledTemporalConditional, TemporalConditional, TemporalContributionConfiguration,
};
use super::program::{self, TemporalExampleProgram};
use super::schema::*;

pub struct ExampleApplication {
    pub runtime: application_installation::WorthQueryProgramApplicationRuntime<
        TemporalHostSchema,
        TemporalExampleProgram,
    >,
    pub conditional: Arc<InstalledTemporalConditional>,
    pub clock_control: ClockController,
}

impl ExampleApplication {
    pub fn publish(gate: &str) -> Self {
        let declaration = TemporalHostSchema::declaration().expect("the example schema is valid");
        let (clock_source, clock_control) = ClockSource::due();
        let runtime = application_installation::in_memory_program(
            program::validated_program(),
            declaration,
            (TemporalContributionConfiguration { clock_source },),
            example_limits(),
            |graph, installed| {
                let principal_binding = installed
                    .principal_binding(TemporalPrincipalBinding::reference())
                    .expect("the temporal principal binding must install");
                seed_graph(graph, &principal_binding, gate);
                Ok(())
            },
        )
        .expect("the validated temporal program must install");
        let conditional = runtime
            .conditional::<TemporalConditional>()
            .expect("the declared temporal conditional must install");
        Self {
            runtime,
            conditional,
            clock_control,
        }
    }
}

pub(crate) fn example_limits() -> application_installation::WorthQueryInMemoryApplicationLimits {
    application_installation::WorthQueryInMemoryApplicationLimits::new(
        product_world_resources(),
        runtime::WorthQueryApplicationCandidateResourceProfile::bounded(5_120, 2_048, 5_120)
            .expect("valid candidate limits"),
        runtime::WorthQueryApplicationQueryResourceProfile::bounded(5_120, 2_048, usize::MAX, 128)
            .expect("valid query limits"),
        primary_graph::SignalConditionalEvaluationBudget::development(),
    )
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

pub(crate) fn seed_graph(
    graph: &mut primary_graph::WorthQueryPrimaryGraphBootstrap<TemporalHostSchema>,
    principal_binding: &domain::WorthQueryInstalledPrincipalBinding<
        TemporalHostSchema,
        TemporalPrincipalBinding,
        ExternalMapping,
        Principal,
        u64,
        declaration::application_schema::U64ApplicationValueBinding,
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

pub(crate) fn product_world_resources() -> runtime::WorthQueryProductWorldResources {
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
