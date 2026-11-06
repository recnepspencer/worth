#[path = "application_runtime_surface/records.rs"]
mod storage_model;
pub use storage_model::*;
use worth_query_decl::facade::application_schema::{
    StringApplicationValueBinding, U64ApplicationValueBinding,
};
use worth_query_decl::facade::authentication::{
    WorthQueryExternalPrincipalIdentity, WorthQueryExternalPrincipalIdentityBinding,
    WorthQueryPrincipalMappingStatus, WorthQueryPrincipalMappingStatusBinding,
};
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_host::facade::primary_graph::WorthQueryApplicationPrincipalKey;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationEntityKey, WorthQueryApplicationEntitySeed,
    WorthQueryPrincipalResolutionMode,
};
use worth_query_host::facade::{application_installation::*, runtime::*};

#[path = "application_runtime_surface/declaration.rs"]
mod declaration;
pub use declaration::*;

impl
    worth_query_host::facade::application_contribution::WorthQueryApplicationContribution<
        HostIdentitySchema,
    > for HostIdentitySurface
{
    type Configuration = ();
    fn configure(
        (): (),
        _setup: &mut worth_query_host::facade::application_contribution::WorthQueryApplicationContributionSetup<'_, HostIdentitySchema>,
    ) -> Result<(), worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenial>
    {
        Ok(())
    }
}

pub fn installed_application(
    build_identity: ApplicationBuildIdentity,
) -> worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<
    HostIdentitySchema,
> {
    in_memory(
        HostIdentitySchema::declaration().unwrap(),
        ((),),
        build_identity,
        WorthQueryInMemoryApplicationLimits::new(
            world_resources(),
            WorthQueryApplicationQueryResourceProfile::new(
                std::num::NonZeroUsize::new(128).unwrap(),
            ),
            worth_query_host::facade::primary_graph::SignalConditionalEvaluationBudget::development(
            ),
        ),
        |_phase, graph, schema| {
            let binding = schema
                .principal_binding(IdentityBinding::reference())
                .unwrap();
            graph
                .bind_principal(
                    &binding,
                    WorthQueryApplicationPrincipalKey::new("host-principal").unwrap(),
                    1_u64,
                    WorthQueryExternalPrincipalIdentity::new("https://issuer.example", "subject")
                        .unwrap(),
                    WorthQueryPrincipalMappingStatus::Enabled,
                )
                .unwrap();
            graph
                .bind_entity(
                    WorthQueryApplicationEntitySeed::new(
                        Account::reference(),
                        WorthQueryApplicationEntityKey::new("account-row").unwrap(),
                    )
                    .field(
                        AccountNumber::at("AccountNumber"),
                        "account-001".to_string(),
                    ),
                )
                .unwrap();
            inspection_capability::bind(graph);
            Ok(())
        },
    )
    .unwrap()
}

pub fn world_resources() -> WorthQueryProductWorldResources {
    WorthQueryProductWorldResources::install(
        RuntimeWorldBudgetInstallation {
            branches: RuntimeWorldBranchBudgetInstallation {
                live_product_branches: 128,
            },
            history: RuntimeWorldHistoryBudgetInstallation {
                retained_composite_commits: 1_024,
                history_metadata_bytes: 16 * 1024 * 1024,
            },
            observations: RuntimeWorldObservationBudgetInstallation {
                active_observations: 512,
            },
            publication: RuntimeWorldPublicationBudgetInstallation {
                active_publication_attempts: 128,
            },
            recovery: RuntimeWorldRecoveryBudgetInstallation {
                retained_product_unpublished_records: 128,
                retained_partial_metadata_bytes: 16 * 1024 * 1024,
            },
            retention: RuntimeWorldRetentionBudgetInstallation {
                unique_exact_component_pins: 1_024,
                in_flight_pin_acquisition_reservations: 256,
            },
            custody: RuntimeWorldCustodyBudgetInstallation {
                owner_created_component_custody_records: 256,
            },
        },
        WorthQueryProductWorldClock::start(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system()),
        WorthQueryInvalidationResources::new(),
        ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(std::num::NonZeroUsize::MIN, None, Some(8_000_000)),
        ),
    )
    .unwrap()
}

#[path = "application_runtime_surface/comparison_model.rs"]
pub mod comparison_model;
#[path = "application_runtime_surface/inspection_authentication.rs"]
pub mod inspection_authentication;
#[path = "application_runtime_surface/inspection_capability.rs"]
pub mod inspection_capability;
#[path = "application_runtime_surface/inspection_model.rs"]
pub mod inspection_model;
