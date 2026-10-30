use worth_query_decl::facade::application_schema::{
    StringApplicationValueBinding, U64ApplicationValueBinding,
};
use worth_query_decl::facade::authentication::{
    WorthQueryExternalPrincipalIdentity, WorthQueryExternalPrincipalIdentityBinding,
    WorthQueryPrincipalMappingStatus, WorthQueryPrincipalMappingStatusBinding,
};
use worth_query_decl::facade::{
    worth_query_application, worth_query_application_contribution, worth_query_aspect,
    worth_query_entity, worth_query_field, worth_query_principal_binding, worth_query_relation,
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

worth_query_application! {
    pub HostIdentitySchema {
        owner: "host_identity_test",
        version: (1, 0),
        contributions: [HostIdentitySurface],
    }
}
worth_query_application_contribution! {
    pub contribution HostIdentitySurface in HostIdentitySchema {
        identity: "worth.query.host.identity-inspection.surface.v1",
        members: |schema| {
            inspection_capability::install(schema)
                .entity(ExternalMapping::reference())
                .entity(Principal::reference())
                .entity(Account::reference())
                .aspect(ExternalMapping::reference(), ExternalIdentity::reference())
                .aspect(Principal::reference(), PrincipalIdentity::reference())
                .field(ExternalMapping::reference(), ExternalIdentityField::reference())
                .field(ExternalMapping::reference(), MappingStatusField::reference())
                .field(Principal::reference(), PrincipalIdentityField::reference())
                .aspect(Account::reference(), AccountIdentity::reference())
                .field(Account::reference(), AccountNumber::reference())
                .relation(
                    MappingTarget::reference(),
                    ExternalMapping::reference(),
                    Principal::reference(),
                )
                .principal_binding(IdentityBinding::reference())
                .application_query(inspection_model::definition())
                .application_query_binding::<inspection_model::AccountReadBinding>()
                .application_query_inspection(inspection_model::inspection_definition())
                .application_query(comparison_model::definition())
                .application_query_binding::<comparison_model::GovernedAccountReadBinding>()
                .application_query_inspection(comparison_model::inspection_definition())
        }
    }
}
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

worth_query_entity!(pub ExternalMapping for HostIdentitySchema);
worth_query_entity!(pub Principal for HostIdentitySchema);
worth_query_entity!(pub Account for HostIdentitySchema);
worth_query_aspect!(pub ExternalIdentity for HostIdentitySchema, ExternalMapping; identity = AspectIdentity(0x9161103e), revision = AspectContractRevision(1),);
worth_query_field!(
    pub ExternalIdentityField for HostIdentitySchema, ExternalMapping, ExternalIdentity:
    WorthQueryExternalPrincipalIdentity => WorthQueryExternalPrincipalIdentityBinding, read_only, equality
);
worth_query_aspect!(pub PrincipalIdentity for HostIdentitySchema, Principal; identity = AspectIdentity(0x9161103f), revision = AspectContractRevision(1),);
worth_query_field!(
    pub PrincipalIdentityField for HostIdentitySchema, Principal, PrincipalIdentity:
    u64 => U64ApplicationValueBinding, read_only, equality
);
worth_query_field!(
    pub MappingStatusField for HostIdentitySchema, ExternalMapping, ExternalIdentity:
    WorthQueryPrincipalMappingStatus => WorthQueryPrincipalMappingStatusBinding, read_write, equality
);
worth_query_relation!(
    pub MappingTarget in HostIdentitySchema,
    ExternalMapping => Principal; integrity = same_context_unbounded_retain_dangling);
worth_query_principal_binding!(
    pub IdentityBinding in HostIdentitySchema,
    mapping ExternalMapping {
        identity: ExternalIdentityField,
        status: MappingStatusField,
        target: MappingTarget => Principal,
        principal_identity: PrincipalIdentityField
    }
);
worth_query_aspect!(pub AccountIdentity for HostIdentitySchema, Account; identity = AspectIdentity(0x91611040), revision = AspectContractRevision(1),);
worth_query_field!(
    pub AccountNumber for HostIdentitySchema, Account, AccountIdentity:
    String => StringApplicationValueBinding, read_only, equality
);

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
                    .field(AccountNumber::reference(), "account-001".to_string()),
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
        WorthQueryProductWorldClock::start(),
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
