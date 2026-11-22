//! Shared Directory Home builders for NumericSchema correction kill courts.
//! Uses a Directory-admissible contribution without the testkit's MemoryOnly wake rules.
mod contribution;
use self::contribution::DirectoryNumericContribution;
use super::*;
use std::num::NonZeroUsize;
use worth_query_decl::facade::{application_program::*, authentication::*};
use worth_query_host::facade::{
    application_entry::*,
    application_installation::*,
    primary_graph::*,
    product::WorthQueryProductBranch,
    runtime::WorthQueryApplicationQueryResourceProfile,
    WorthQueryApplicationHome, WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
};
pub(super) use worth_query_testkit_certification_correction::facade::certification::application_graph::correction::model;
use model::{
    authentication::*,
    calculation_query,
    output_program::OutputProgram,
    program::Program,
    query::Row,
    schema::*,
};

pub(super) struct DirectoryProgram;
impl ApplicationProgramDefinition<NumericSchema> for DirectoryProgram {
    type Contributions = (DirectoryNumericContribution,);
    type Outputs = ApplicationProgramOutputs<ApplicationNoOutputGraph>;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("numeric-correction.directory-program.v1");
    fn adoption_authorization<V: ApplicationProgramAdoptionAuthorizationVisitor<NumericSchema>>(
        visitor: V,
    ) -> V::Output {
        visitor.capability::<model::adoption::AdoptBinding>()
    }
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        <Program as ApplicationProgramDefinition<NumericSchema>>::feature_specs()
    }
}

pub(super) struct DirectoryOutputProgram;
impl ApplicationProgramDefinition<NumericSchema> for DirectoryOutputProgram {
    type Contributions = (DirectoryNumericContribution,);
    type Outputs = <OutputProgram as ApplicationProgramDefinition<NumericSchema>>::Outputs;
    type Rules = ApplicationRuleLeaf;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("numeric-correction.directory-output-program.v1");
    fn adoption_authorization<V: ApplicationProgramAdoptionAuthorizationVisitor<NumericSchema>>(
        visitor: V,
    ) -> V::Output {
        visitor.capability::<model::adoption::AdoptBinding>()
    }
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        <OutputProgram as ApplicationProgramDefinition<NumericSchema>>::feature_specs()
    }
}

pub(super) type NumericHome = WorthQueryApplicationHome<NumericSchema, DirectoryProgram>;
pub(super) type OutputNumericHome = WorthQueryApplicationHome<NumericSchema, DirectoryOutputProgram>;

pub(super) fn limits() -> WorthQueryCorrectionHistoryLimits {
    WorthQueryCorrectionHistoryLimits::new(None, NonZeroUsize::new(8).unwrap())
}

fn app_limits() -> WorthQueryApplicationLimits {
    WorthQueryApplicationLimits::new(
        identity::world_resources(),
        WorthQueryApplicationQueryResourceProfile::default(),
        SignalConditionalEvaluationBudget::development(),
    )
}

fn seed_numeric(
    graph: &mut WorthQueryPrimaryGraphBootstrap<NumericSchema>,
    installed: &WorthQueryInstalledApplicationSchema<NumericSchema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    let binding = installed
        .principal_binding(ActorBinding::reference())
        .unwrap();
    graph.bind_principal(
        &binding,
        WorthQueryApplicationPrincipalKey::new("numeric-operator").unwrap(),
        1_u64,
        operator_identity(),
        WorthQueryPrincipalMappingStatus::Enabled,
    )?;
    graph.bind_principal(
        &binding,
        WorthQueryApplicationPrincipalKey::new("numeric-reviewer").unwrap(),
        2_u64,
        reviewer_identity(),
        WorthQueryPrincipalMappingStatus::Enabled,
    )?;
    graph.bind_entity(
        WorthQueryApplicationEntitySeed::new(
            Number::reference(),
            WorthQueryApplicationEntityKey::new("numeric").unwrap(),
        )
        .field(Key::at("Key"), "numeric".to_owned())
        .field(Primary::at("Primary"), Counter(5_u64))
        .field(Secondary::at("Secondary"), 7_u64)
        .field(ReadEnabled::at("ReadEnabled"), true)
        .field(Ceiling::at("Ceiling"), 100_u64)
        .field(Calculated::at("Calculated"), 0_u64)
        .field(
            model::write_admission::Binary::at("Binary"),
            BinaryCounter(5_u64),
        )
        .field(
            model::write_admission::Canonical::at("Canonical"),
            model::write_admission::CanonicalCounter(5),
        )
        .field(
            model::write_admission::Delegated::at("Delegated"),
            DelegatedCounter(5_u64),
        )
        .field(
            model::composite_definition_model::DefinitionRevision::at("DefinitionRevision"),
            5_u64,
        ),
    )?;
    model::adoption::seed(graph);
    model::causal_inspection::capability::seed(graph);
    Ok(())
}

fn validated<P: ApplicationProgramDefinition<NumericSchema>>(
) -> ValidatedApplicationProgram<NumericSchema, P> {
    ApplicationProgramAuthoring::<NumericSchema, P>::begin()
        .validated_program()
        .unwrap()
}

pub(super) fn open_output_home(
    label: &str,
    location: std::path::PathBuf,
    execution: worth_execution::ExecutionRequest<'_, '_>,
    seed: bool,
) -> OutputNumericHome {
    let declaration = NumericSchema::declaration().unwrap();
    let request = WorthQueryApplicationHomeRequest::new(
        WorthQueryApplicationHomeLocation::Directory(location),
        validated::<DirectoryOutputProgram>(),
        WorthQueryApplicationProgramRoster::new().support(validated::<DirectoryProgram>()),
        declaration,
        ((),),
        ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
        app_limits(),
        move |_, graph, installed| {
            assert!(seed, "reopened OutputNumericHome ({label}) must not re-initialize");
            seed_numeric(graph, installed)
        },
    );
    with_bootstrap_advancement_from_request(execution, |phase| {
        request.open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|refused| panic!("OutputNumericHome open ({label}): {refused:?}"))
}

pub(super) fn open_program_home(
    location: std::path::PathBuf,
    execution: worth_execution::ExecutionRequest<'_, '_>,
    seed: bool,
) -> NumericHome {
    let declaration = NumericSchema::declaration().unwrap();
    let request = WorthQueryApplicationHomeRequest::new(
        WorthQueryApplicationHomeLocation::Directory(location),
        validated::<DirectoryProgram>(),
        WorthQueryApplicationProgramRoster::new().support(validated::<DirectoryOutputProgram>()),
        declaration,
        ((),),
        ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
        app_limits(),
        move |_, graph, installed| {
            assert!(seed, "reopened NumericHome must not re-initialize");
            seed_numeric(graph, installed)
        },
    );
    with_bootstrap_advancement_from_request(execution, |phase| {
        request.open_in_advancement(&phase)
    })
    .unwrap()
    .unwrap_or_else(|refused| panic!("NumericHome open: {refused:?}"))
}

pub(super) fn read_rows<P: ApplicationProgramDefinition<NumericSchema>>(
    runtime: &WorthQueryProgramApplicationRuntime<NumericSchema, P>,
    branch: WorthQueryProductBranch,
) -> Vec<Row> {
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    runtime
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch)
        .query(model::query::Read {
            key: "numeric".to_owned(),
        })
        .execute()
        .unwrap()
        .rows()
        .to_vec()
}

pub(super) fn read_calc(
    runtime: &WorthQueryProgramApplicationRuntime<NumericSchema, DirectoryOutputProgram>,
) -> u64 {
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    runtime
        .runtime()
        .request(&principal, &scope)
        .query(calculation_query::Read {
            key: "numeric".to_owned(),
        })
        .execute()
        .unwrap()
        .rows()[0]
        .value
}

pub(super) fn head_ordinal<P: ApplicationProgramDefinition<NumericSchema>>(
    runtime: &WorthQueryProgramApplicationRuntime<NumericSchema, P>,
    branch: WorthQueryProductBranch,
) -> u64 {
    runtime
        .with_host_advancement(|phase| {
            runtime
                .runtime()
                .on_branch(branch)
                .select(&phase)
                .unwrap()
                .product()
                .selected_commit()
                .ordinal()
        })
        .unwrap()
}

pub(super) fn fork_branch(home: &NumericHome) -> WorthQueryProductBranch {
    use worth_query_host::facade::product::{
        WorthQueryProductBranchCreateError as Create,
        WorthQueryProductBranchCreationContinuation as Continued,
    };
    let app = home.runtime().runtime();
    let scope = request_scope();
    let created = app
        .with_application_advancement(&scope, |phase| {
            app.branches()
                .fork(app.current_world())
                .components(|parts| parts.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
        })
        .unwrap();
    let mut original = match created {
        Ok(branch) => return branch,
        Err(Create::ProductUnpublished(original)) => original,
        Err(cause) => panic!("fork branch: {cause:?}"),
    };
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        assert!(Instant::now() < deadline, "fork settles");
        let scope = request_scope();
        match app
            .with_application_advancement(&scope, |phase| original.continue_in_advancement(&phase))
            .unwrap()
        {
            Ok(Continued::Performed(branch)) => return branch,
            Ok(other) => panic!("unexpected fork continuation: {other:?}"),
            Err(failure) => {
                original = failure.into_recovery();
                std::thread::yield_now();
            }
        }
    }
}

pub(super) fn find_fork_branch(home: &NumericHome) -> WorthQueryProductBranch {
    let app = home.runtime().runtime();
    let root = app.current_world();
    let scope = request_scope();
    let inventory = app
        .with_application_advancement(&scope, |phase| app.branches().inventory(&phase).unwrap())
        .unwrap();
    let forks: Vec<_> = inventory.iter().filter(|branch| *branch != root).collect();
    let [fork] = forks.as_slice() else {
        panic!("expected one fork branch, found {}", forks.len());
    };
    *fork
}
