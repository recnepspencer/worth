//! Actual retained request composition with declared input-operation costs.
use super::code_identity::ChangedCodeTotals;
use super::computation::HistoryOwner;
use super::*;
use worth_query_host::facade::application_installation::ApplicationBuildIdentity;
pub(super) struct HistoryConfiguration {
    topology: TopologyConfiguration,
    code: std::sync::Arc<std::sync::atomic::AtomicBool>,
    reuse: std::sync::Arc<std::sync::atomic::AtomicBool>,
}
impl Default for HistoryConfiguration {
    fn default() -> Self {
        Self {
            topology: TopologyConfiguration {
                final_feature: crate::planar_final_feature_runtime(),
                setup_calls: Default::default(),
                invariant_calls: Default::default(),
                invariant_probe: Default::default(),
                invariant_read_observer: None,
                producer_authorization_denials: Default::default(),
                producer_domain_denial: Default::default(),
                producer_contacts: None,
            },
            code: Default::default(),
            reuse: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(REUSE_ON.get())),
        }
    }
}
pub(super) struct HistoryContribution;
impl ApplicationSchemaContribution<CheckpointSchema> for HistoryContribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        <TopologyContribution as ApplicationSchemaContribution<CheckpointSchema>>::IDENTITY;
    fn register_members(
        builder: ApplicationSchemaDeclarationBuilder<CheckpointSchema>,
    ) -> ApplicationSchemaDeclarationBuilder<CheckpointSchema> {
        <TopologyContribution as ApplicationSchemaContribution<CheckpointSchema>>::register_members(
            builder,
        )
    }
}
impl WorthQueryApplicationContribution<CheckpointSchema> for HistoryContribution {
    type Configuration = HistoryConfiguration;

    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<CheckpointSchema>,
    ) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
        <TopologyContribution as WorthQueryApplicationContribution<CheckpointSchema>>::contracts(
            contracts,
        )?;
        contracts.producer::<RegionOutputProducer<CheckpointSchema>>()?;
        contracts.conditional::<RegionOutputReadiness<CheckpointSchema>>()?;
        Ok(())
    }

    fn configure(
        configuration: Self::Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, CheckpointSchema>,
    ) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
        super::super::collection::lifecycle::idle(setup)?;
        let installed = setup
            .partitioned_computation::<PlanarFinalOutputFeature, OracleTotals, _>(HistoryOwner {
                steps: CARRIED_CEILING.get(),
            })?;
        let changed = setup
            .partitioned_computation::<PlanarFinalOutputFeature, ChangedCodeTotals, _>(
                HistoryOwner { steps: false },
            )?;
        let code = configuration.code;
        let reuse = configuration.reuse;
        let ceiling = CARRIED_CEILING.get();
        let handler = RegionOutputHandler::running(move |reader, set| {
            super::super::decision_plan::begin();
            if code.load(Ordering::Relaxed) {
                let prepared = if reuse.load(Ordering::Relaxed) {
                    changed.prepare(reader, set)
                } else {
                    changed.prepare_without_reuse_for_test(reader, set)
                };
                return super::super::decision_plan::prepared(prepared, |observations| {
                    REPORTS.with(|reports| reports.borrow_mut().extend(observations));
                });
            }
            let prepared = if reuse.load(Ordering::Relaxed) {
                installed.prepare(reader, set)
            } else {
                installed.prepare_without_reuse_for_test(reader, set)
            };
            let prepared = prepared.map(|prepared| {
                if ceiling && COMPUTE_CEILING.get() {
                    prepared.with_compute_ceiling_for_test(16)
                } else {
                    prepared
                }
            });
            super::super::decision_plan::prepared(prepared, |observations| {
                REPORTS.with(|reports| reports.borrow_mut().extend(observations));
            })
        });
        setup.handler::<RegionOutputBinding<CheckpointSchema>, _>(handler)?;
        setup.handler::<RegionTotalsDemandBinding<CheckpointSchema>, _>(
            super::super::super::demand::RegionTotalsHandler::idle(),
        )?;
        setup.handler::<EntryEditBinding<CheckpointSchema>, _>(EntryEditHandler)?;
        super::super::super::entry_correction::configure(setup)?;
        setup.producer::<RegionOutputProducer<CheckpointSchema>>(RegionOutputProvider)?;
        setup.conditional::<RegionOutputReadiness<CheckpointSchema>>(())?;
        TopologyContribution::configure_topology(configuration.topology, setup)
    }
}

pub(super) struct HistoryProgram;
impl ApplicationProgramDefinition<CheckpointSchema> for HistoryProgram {
    type Contributions = (HistoryContribution,);
    type Outputs = ApplicationProgramOutputs<OracleRoot>;
    type Rules = CheckpointRules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("checkpoint-region-output-program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        demand_policy::feature_specs_with_final_output(
            required_chain::output_feature_spec(),
            demand_policy::final_output_feature::<RegionArtifact>()
                .managed_computation::<OracleTotals>()
                .managed_computation::<ChangedCodeTotals>()
                .conditional_operation::<TotalRegionOutput>()
                .native_action::<EntryEditBinding<CheckpointSchema>>()
                .finish_action()
                .finish(),
        )
    }
}

pub(super) type Application =
    application_installation::WorthQueryProgramApplicationRuntime<CheckpointSchema, HistoryProgram>;
pub(super) type Request<'application, 'principal, 'scope> =
    WorthQueryApplicationRequest<'application, 'principal, 'scope, CheckpointSchema>;

/// The scope whose output the producer keeps. Its ordinate names the set.
const SCOPE: &str = "anchor-isolated";
/// Installs the default host, widened so its operations may be as wide as
/// the largest set's decision.
pub(super) fn install(seed: impl FnOnce(&mut Graph)) -> Application {
    install_after_authority_issuances(seed, 0)
}
/// Public bootstrap issuance advances the same counter as the runtime's
/// operation authority, without exposing or resetting any private counter.
pub(super) fn install_after_authority_issuances(
    seed: impl FnOnce(&mut Graph),
    issuances: usize,
) -> Application {
    install_configured(seed, issuances, Default::default())
}
pub(super) fn install_with_controls(
    seed: impl FnOnce(&mut Graph),
    code: std::sync::Arc<std::sync::atomic::AtomicBool>,
    reuse: std::sync::Arc<std::sync::atomic::AtomicBool>,
) -> Application {
    install_configured(
        seed,
        0,
        HistoryConfiguration {
            code,
            reuse,
            ..Default::default()
        },
    )
}
fn install_configured(
    seed: impl FnOnce(&mut Graph),
    issuances: usize,
    configuration: HistoryConfiguration,
) -> Application {
    const RETAINED_COMMITS: u64 = 32;
    let invalidation = support::invalidation();
    let limits = support::limits_with_room(RETAINED_COMMITS, 16, 64, invalidation);
    let declaration = CheckpointSchema::declaration().unwrap();
    let program = ApplicationProgramAuthoring::<CheckpointSchema, HistoryProgram>::begin()
        .with_schema(&declaration)
        .validated_program()
        .unwrap();
    application_installation::in_memory_program(
        program, declaration, (configuration,), ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(), limits,
        |_phase, graph, installed| {
            let principal = installed.principal_binding(
                ConsumerPrincipalBinding::reference::<CheckpointSchema>()
            ).unwrap();
            graph.bind_principal(
                &principal,
                primary_graph::WorthQueryApplicationPrincipalKey::new("model-owner").unwrap(),
                1_u64, support::external_identity(),
                worth_query_host::facade::declaration::authentication::WorthQueryPrincipalMappingStatus::Enabled,
            )?;
            support::seed_cycle(graph);
            seed(graph);
            for _ in 0..issuances {
                drop(graph.retain_invariant_projection_authority());
            }
            Ok(())
        },
    ).unwrap()
}

/// Demands the scope's output until it settles: the producer's contacts in
/// the demand, and the runs its decisions made.
pub(super) fn demand(
    request: &Request<'_, '_, '_>,
    application: &Application,
) -> (usize, Vec<OracleRun>, Vec<Option<u64>>) {
    room().clear();
    published_states();
    REPORTS.with(|reports| reports.borrow_mut().clear());
    let mut demand = request
        .demand(RegionOutputDemand(SCOPE.to_owned()))
        .start_dependent_in_program::<HistoryProgram, RegionConnection>(application)
        .expect("the region output demand starts");
    let settled = (0..256)
        .find_map(|_| {
            match demand
                .advance(request)
                .expect("the region output demand advances")
            {
                WorthQueryApplicationOutputDemandProgress::Pending
                | WorthQueryApplicationOutputDemandProgress::Progressed => None,
                WorthQueryApplicationOutputDemandProgress::Settled(settled) => Some(settled),
            }
        })
        .expect("the region output demand settles");
    let contacts = settled.producer_contacts_in_this_demand();
    let mut runs = take_runs(None);
    let published = published_states();
    if let Some(last) = runs.last_mut() {
        last.published = published;
    } else {
        assert!(
            published.is_empty(),
            "a demand with no run publishes no computation state"
        );
    }
    (
        contacts,
        runs,
        REPORTS.with(|reports| std::mem::take(&mut *reports.borrow_mut())),
    )
}

/// Commits one entry edit.
pub(super) fn edit(
    request: &Request<'_, '_, '_>,
    application: &Application,
    edit: EntryEdit,
    command: u64,
) {
    let made = format!("{edit:?}");
    let outcome = request
        .advanced()
        .mutate(edit.commanded(command))
        .without_source()
        .idempotency(&command)
        .execute_in_program::<HistoryProgram>(
            application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        );
    assert!(
        matches!(
            &outcome,
            Ok(WorthQueryApplicationMutationOutcome::Committed { .. })
        ),
        "the edit {made} commits: {outcome:?}"
    );
}

/// Moves the scope's ordinate to `y`, which names the set its output totals.
pub(super) fn adjust(
    request: &Request<'_, '_, '_>,
    application: &Application,
    y: u64,
    command: u64,
) {
    let observed = request
        .query(PlanarRead {
            body_key: SCOPE.to_owned(),
        })
        .execute()
        .expect("the scope's source is readable");
    request
        .advanced()
        .mutate(PlanarSourceAdjustment {
            scope_key: SCOPE.to_owned(),
            replacement_y: length(y),
        })
        .expect_source(observed.observed_sources()[0].clone())
        .idempotency(&command)
        .execute_performed::<HistoryProgram, OracleRoot>(
            application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .expect("the scope's ordinate moves");
}

// These are observed full-run charges from the existing execution observer,
// never an inventory of inputs filled by the handler.
thread_local! {
    static REPORTS: std::cell::RefCell<Vec<Option<u64>>> = const { std::cell::RefCell::new(Vec::new()) };
}

thread_local! {pub(super) static REUSE_ON:std::cell::Cell<bool>=const{std::cell::Cell::new(true)};}

thread_local! { pub(super) static CARRIED_CEILING: std::cell::Cell<bool> = const {std::cell::Cell::new(false)}; pub(super) static COMPUTE_CEILING: std::cell::Cell<bool> = const {std::cell::Cell::new(false)}; }
