//! Host, Directory Home, and mutation launchers for Hard DX managed-index courts.
use super::operations::{AssetSpec, CreateAsset, DeleteAsset, UpdateAsset};
use super::schema::*;
use super::{Home, Host, Program};
use std::num::NonZeroUsize;
use worth_execution::{CancellationToken, ExecutionRequest, ExecutionResourceLease, LeaseRequest};
use worth_foundational::facade::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_query_decl::facade::{application_program::*, application_schema::*, authentication::*};
use worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock;
use worth_query_host::facade::application_contribution::with_bootstrap_advancement_from_request;
use worth_query_host::facade::application_entry::*;
use worth_query_host::facade::application_installation::*;
use worth_query_host::facade::primary_graph::*;
use worth_query_host::facade::runtime::{
    ExecutionAllocationPolicy, RuntimeWorldBranchBudgetInstallation,
    RuntimeWorldBudgetInstallation, RuntimeWorldCustodyBudgetInstallation,
    RuntimeWorldHistoryBudgetInstallation, RuntimeWorldObservationBudgetInstallation,
    RuntimeWorldPublicationBudgetInstallation, RuntimeWorldRecoveryBudgetInstallation,
    RuntimeWorldRetentionBudgetInstallation, WorthQueryApplicationQueryResourceProfile,
    WorthQueryInvalidationResources, WorthQueryProductWorldClock, WorthQueryProductWorldResources,
};
use worth_query_host::facade::{
    WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
};
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
    authenticate_operator, operator_identity, request_scope,
};

pub(super) fn home_lease() -> ExecutionResourceLease<'static> {
    ExecutionResourceLease::serial(LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::MIN, Some(128 * 1024 * 1024), None),
        ),
        cancellation: CancellationToken::new(),
        deadline: None,
    })
    .unwrap()
}

pub(super) fn limits(
    memory_bytes: Option<u64>,
    work_ceiling: Option<u64>,
) -> WorthQueryApplicationLimits {
    let clock = WorthQueryAdmissionClock::system();
    let policy = ExecutionRequestPolicy::new(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        ExecutionBudget::new(NonZeroUsize::MIN, memory_bytes, work_ceiling),
    );
    let world = WorthQueryProductWorldResources::install(
        RuntimeWorldBudgetInstallation {
            branches: RuntimeWorldBranchBudgetInstallation {
                live_product_branches: 32,
            },
            history: RuntimeWorldHistoryBudgetInstallation {
                retained_composite_commits: 128,
                history_metadata_bytes: 64 * 1024 * 1024,
            },
            observations: RuntimeWorldObservationBudgetInstallation {
                active_observations: 128,
            },
            publication: RuntimeWorldPublicationBudgetInstallation {
                active_publication_attempts: 32,
            },
            recovery: RuntimeWorldRecoveryBudgetInstallation {
                retained_product_unpublished_records: 32,
                retained_partial_metadata_bytes: 4 * 1024 * 1024,
            },
            retention: RuntimeWorldRetentionBudgetInstallation {
                unique_exact_component_pins: 256,
                in_flight_pin_acquisition_reservations: 64,
            },
            custody: RuntimeWorldCustodyBudgetInstallation {
                owner_created_component_custody_records: 64,
            },
        },
        WorthQueryProductWorldClock::start(clock),
        WorthQueryInvalidationResources::new(),
        policy,
    )
    .unwrap();
    WorthQueryApplicationLimits::new(
        world,
        WorthQueryApplicationQueryResourceProfile::new(NonZeroUsize::new(8).unwrap()),
        SignalConditionalEvaluationBudget::development(),
    )
}

fn seed_floor(
    graph: &mut WorthQueryPrimaryGraphBootstrap<ShopFloorSchema>,
    installed: &worth_query_host::facade::domain::WorthQueryInstalledApplicationSchema<
        ShopFloorSchema,
    >,
    initial: &[AssetSpec],
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    graph.bind_principal(
        &installed
            .principal_binding(PrincipalBinding::reference())
            .unwrap(),
        WorthQueryApplicationPrincipalKey::new("operator").unwrap(),
        1_u64,
        operator_identity(),
        WorthQueryPrincipalMappingStatus::Enabled,
    )?;
    graph.bind_entity(
        WorthQueryApplicationEntitySeed::new(
            Floor::reference(),
            WorthQueryApplicationEntityKey::new("floor-1").unwrap(),
        )
        .field(FloorId::at("FloorId"), "floor-1".to_owned()),
    )?;
    for s in initial {
        let id = format!("asset-{}", s.key);
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(
                Asset::reference(),
                WorthQueryApplicationEntityKey::new(id.clone()).unwrap(),
            )
            .field(AssetKey::at("AssetKey"), s.key)
            .field(Title::at("Title"), s.title.clone())
            .field(StartMin::at("StartMin"), s.start_min)
            .field(EndMin::at("EndMin"), s.end_min)
            .field(MinX::at("MinX"), s.min[0])
            .field(MinY::at("MinY"), s.min[1])
            .field(MinZ::at("MinZ"), s.min[2])
            .field(MaxX::at("MaxX"), s.max[0])
            .field(MaxY::at("MaxY"), s.max[1])
            .field(MaxZ::at("MaxZ"), s.max[2])
            .field(FminX::at("FminX"), CanonicalF64(s.fmin_bits[0]))
            .field(FminY::at("FminY"), CanonicalF64(s.fmin_bits[1]))
            .field(FmaxX::at("FmaxX"), CanonicalF64(s.fmax_bits[0]))
            .field(FmaxY::at("FmaxY"), CanonicalF64(s.fmax_bits[1])),
        )?;
        graph.bind_relation(WorthQueryApplicationRelationSeed::new(
            ContainsAsset::reference(),
            format!("contains-{}", s.key),
            WorthQueryApplicationEntityKey::new("floor-1").unwrap(),
            WorthQueryApplicationEntityKey::new(id).unwrap(),
        ))?;
    }
    Ok(())
}

pub(super) fn install(
    initial: &[AssetSpec],
    memory_bytes: Option<u64>,
    work_ceiling: Option<u64>,
) -> Host {
    let declaration = ShopFloorSchema::declaration().unwrap();
    let specs = initial.to_vec();
    in_memory_program(
        ApplicationProgramAuthoring::<ShopFloorSchema, Program>::begin()
            .with_schema(&declaration)
            .validated_program()
            .unwrap(),
        declaration,
        ((),),
        ApplicationBuildIdentity::parse("8c8c8c8c-8c8c-8c8c-8c8c-8c8c8c8c8c8c").unwrap(),
        limits(memory_bytes, work_ceiling),
        move |_, graph, installed| seed_floor(graph, installed, &specs),
    )
    .unwrap()
}

pub(super) fn open_directory_home_with_limits(
    location: &std::path::Path,
    execution: &ExecutionRequest<'_, '_>,
    initialize: bool,
    initial: &[AssetSpec],
    app_limits: WorthQueryApplicationLimits,
) -> Home {
    let declaration = ShopFloorSchema::declaration().unwrap();
    let program = ApplicationProgramAuthoring::<ShopFloorSchema, Program>::begin()
        .with_schema(&declaration)
        .validated_program()
        .unwrap();
    let specs = initial.to_vec();
    let request = WorthQueryApplicationHomeRequest::new(
        WorthQueryApplicationHomeLocation::Directory(location.to_path_buf()),
        program,
        WorthQueryApplicationProgramRoster::new(),
        declaration,
        ((),),
        ApplicationBuildIdentity::parse("8c8c8c8c-8c8c-8c8c-8c8c-8c8c8c8c8c8c").unwrap(),
        app_limits,
        move |_, graph, installed| {
            assert!(initialize, "reopened Home must not replay initializer");
            seed_floor(graph, installed, &specs)
        },
    );
    with_bootstrap_advancement_from_request(*execution, |phase| request.open_in_advancement(&phase))
        .unwrap()
        .unwrap_or_else(|refused| panic!("directory Home open failed: {refused:?}"))
}

pub(super) fn create_asset(host: &Host, spec: AssetSpec, idem: u64) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let out = host
        .runtime()
        .request(&principal, &scope)
        .advanced()
        .mutate(CreateAsset {
            floor: "floor-1".into(),
            spec,
        })
        .without_source()
        .idempotency(&idem)
        .execute_in_program(host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(
        out,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
}

pub(super) fn update_asset(host: &Host, spec: AssetSpec, idem: u64) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let out = host
        .runtime()
        .request(&principal, &scope)
        .advanced()
        .mutate(UpdateAsset {
            key: spec.key,
            spec,
        })
        .without_source()
        .idempotency(&idem)
        .execute_in_program(host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(
        out,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
}

pub(super) fn delete_asset(host: &Host, key: u64, idem: u64) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let out = host
        .runtime()
        .request(&principal, &scope)
        .advanced()
        .mutate(DeleteAsset {
            floor: "floor-1".into(),
            key,
        })
        .without_source()
        .idempotency(&idem)
        .execute_in_program(host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(matches!(
        out,
        WorthQueryApplicationMutationOutcome::Committed { .. }
    ));
}

pub(super) fn keys(rows: &[AssetRow]) -> Vec<u64> {
    rows.iter().map(|r| r.key).collect()
}
