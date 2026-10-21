use worth_query_host::facade::application_installation::ApplicationBuildIdentity;
use worth_query_host::facade::{
    application_installation::{
        in_memory_program, WorthQueryInMemoryApplicationLimits, WorthQueryProgramApplicationRuntime,
    },
    declaration::authentication::{
        WorthQueryExternalPrincipalIdentity, WorthQueryPrincipalMappingStatus,
    },
    primary_graph::{
        SignalConditionalEvaluationBudget, WorthQueryApplicationEntityKey,
        WorthQueryApplicationEntitySeed, WorthQueryApplicationPrincipalKey,
        WorthQueryPrimaryGraphInstallationDenial,
    },
    runtime::{
        RuntimeWorldBranchBudgetInstallation, RuntimeWorldBudgetInstallation,
        RuntimeWorldCustodyBudgetInstallation, RuntimeWorldHistoryBudgetInstallation,
        RuntimeWorldObservationBudgetInstallation, RuntimeWorldPublicationBudgetInstallation,
        RuntimeWorldRecoveryBudgetInstallation, RuntimeWorldRetentionBudgetInstallation,
        WorthQueryApplicationQueryResourceProfile, WorthQueryInvalidationResources,
        WorthQueryProductWorldClock, WorthQueryProductWorldResources,
    },
};

use crate::declaration::{
    IdentityIdField, QueryRevisionValueField, QueryTextStatusField, WorthUiApplicationSchema,
    WorthUiPrincipal, WorthUiPrincipalBinding, WorthUiRecord,
};

use super::application_program::{validated_status_program, WorthUiStatusProgram};
use super::WorthUiStatusOwnerError;

pub(super) type WorthUiStatusApplication =
    WorthQueryProgramApplicationRuntime<WorthUiApplicationSchema, WorthUiStatusProgram>;

pub(super) fn install_status_application(
) -> Result<WorthUiStatusApplication, WorthUiStatusOwnerError> {
    let program = validated_status_program()
        .map_err(|error| WorthUiStatusOwnerError::Installation(format!("{error:?}")))?;
    let declaration = WorthUiApplicationSchema::declaration()
        .map_err(|error| WorthUiStatusOwnerError::Installation(format!("{error:?}")))?;
    in_memory_program(
        program,
        declaration,
        ((),),
        ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID"))
            .expect("embedded application build identity"),
        resource_limits(),
        |_phase, graph, installed| {
            let binding = installed
                .principal_binding(WorthUiPrincipalBinding::reference())
                .map_err(|error| {
                    WorthQueryPrimaryGraphInstallationDenial::binding_not_installed(
                        error.to_string(),
                    )
                })?;
            let principal = WorthQueryApplicationPrincipalKey::<
                WorthUiApplicationSchema,
                WorthUiPrincipal,
            >::new("platform-pulse-source".to_owned())
            .expect("the static UI principal key is valid");
            graph.bind_principal(
                &binding,
                principal,
                1,
                external_identity(),
                WorthQueryPrincipalMappingStatus::Enabled,
            )?;
            let record =
                WorthQueryApplicationEntityKey::<WorthUiApplicationSchema, WorthUiRecord>::new(
                    "platform-pulse-status".to_owned(),
                )
                .expect("the static UI record key is valid");
            graph.bind_entity(
                WorthQueryApplicationEntitySeed::new(WorthUiRecord::reference(), record)
                    .field(
                        IdentityIdField::reference(),
                        "platform.pulse.status".to_owned(),
                    )
                    .field(QueryTextStatusField::reference(), "PENDING".to_owned())
                    .field(QueryRevisionValueField::reference(), 0),
            )?;
            Ok(())
        },
    )
    .map_err(|error| WorthUiStatusOwnerError::Installation(format!("{error:?}")))
}

pub(super) fn external_identity() -> WorthQueryExternalPrincipalIdentity {
    WorthQueryExternalPrincipalIdentity::new("https://worth.ui/local", "platform-pulse-source")
        .expect("the local UI source identity is canonical")
}

fn resource_limits() -> WorthQueryInMemoryApplicationLimits {
    let world = WorthQueryProductWorldResources::install(
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
        WorthQueryProductWorldClock::start(),
        WorthQueryInvalidationResources::new(),
        crate::query_runtime_resources::UI_EXECUTION_POLICY,
    )
    .expect("the UI product resources are statically valid");
    WorthQueryInMemoryApplicationLimits::new(
        world,
        WorthQueryApplicationQueryResourceProfile::new(
            std::num::NonZeroUsize::new(16).expect("concurrent graph admission is nonzero"),
        ),
        SignalConditionalEvaluationBudget::development(),
    )
}
