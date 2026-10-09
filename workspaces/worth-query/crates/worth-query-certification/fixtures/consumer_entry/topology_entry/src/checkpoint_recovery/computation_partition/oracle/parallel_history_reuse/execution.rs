//! Actual retained request composition with declared input-operation costs.
use super::computation::HistoryOwner;
use super::*;
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
    type Configuration = TopologyConfiguration;

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
        let installed = setup
            .partitioned_computation::<PlanarFinalOutputFeature, OracleTotals, _>(HistoryOwner)?;
        let handler = RegionOutputHandler::running(move |reader, set| {
            take_calls();
            COMBINES.store(0, Ordering::Relaxed);
            tree_count::reset();
            worth_query_host::facade::primary_graph::partitioned_computation_tree_work_on_this_thread_for_test();
            let outcome = (|| {
                let computed = installed
                    .prepare(reader, set)?
                    .compute(reader.managed_computation_execution())?;
                let charged_work = computed.charged_work();
                Ok((computed.complete()?.to_bits(), charged_work))
            })();
            let observations = runs_on_this_thread();
            REPORTS.with(|reports| {
                reports.borrow_mut().extend(
                    observations
                        .iter()
                        .map(|(_, report)| report.map(|report| report.charged_work())),
                )
            });
            let runs = observations.into_iter().map(|(run, _)| run);
            let value = outcome.as_ref().ok().map(|(bits, _)| *bits);
            room().push(OracleRun {
                published: Vec::new(),
                outcome,
                runs: runs.collect(),
                calls: take_calls(),
                tree_runs: worth_query_host::facade::primary_graph::partitioned_computation_tree_work_on_this_thread_for_test(),
                combines: COMBINES.swap(0, Ordering::Relaxed),
                tree_nodes: tree_count::take_nodes(),
                placement: tree_count::placement(),
            });
            value
        });
        setup.handler::<RegionOutputBinding<CheckpointSchema>, _>(handler)?;
        setup.handler::<RegionTotalsDemandBinding<CheckpointSchema>, _>(
            super::super::super::demand::RegionTotalsHandler::idle(),
        )?;
        setup.handler::<EntryEditBinding<CheckpointSchema>, _>(EntryEditHandler)?;
        setup.producer::<RegionOutputProducer<CheckpointSchema>>(RegionOutputProvider)?;
        setup.conditional::<RegionOutputReadiness<CheckpointSchema>>(())?;
        TopologyContribution::configure_topology(configuration, setup)
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
                .conditional_operation::<TotalRegionOutput>()
                .mutation::<EntryEditBinding<CheckpointSchema>>()
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
    let width = u64::try_from(DECISION_FACT_BUDGET + WIDTH_BESIDE_DECISION).unwrap();
    let host = support::candidates();
    let candidates =
        worth_query_host::facade::runtime::WorthQueryApplicationCandidateResourceProfile::bounded(
            host.maximum_items().max(width),
            host.maximum_retained_representation_bytes()
                .max((1024 * LARGEST_SET + 320 * 256) as u64),
            host.maximum_validator_work().max(width),
        )
        .and_then(|candidates| candidates.with_maximum_operation_width(width))
        .unwrap();
    const RETAINED_COMMITS: u64 = 32;
    let invalidation =
        support::invalidation(128 * 1_024 * 1_024, 1_000_000, RETAINED_COMMITS as usize);
    let limits = support::limits_with_room(RETAINED_COMMITS, 16, 64, invalidation, candidates);
    support::install_program_with_limits::<HistoryProgram>(
        None,
        Default::default(),
        limits,
        |graph| {
            support::seed_cycle(graph);
            seed(graph);
            for _ in 0..issuances {
                drop(graph.retain_invariant_projection_authority());
            }
        },
    )
}

/// Demands the scope's output until it settles: the producer's contacts in
/// the demand, and the runs its decisions made.
pub(super) fn demand(
    request: &Request<'_, '_, '_>,
    application: &Application,
) -> (usize, Vec<OracleRun>, Vec<Option<u64>>) {
    room().clear();
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
                WorthQueryApplicationOutputDemandProgress::Pending => None,
                WorthQueryApplicationOutputDemandProgress::Settled(settled) => Some(settled),
            }
        })
        .expect("the region output demand settles");
    let contacts = settled.producer_contacts_in_this_demand();
    let runs = take_runs(None);
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
        .mutate(edit.commanded(command))
        .without_source()
        .idempotency(&command)
        .execute_in_program::<HistoryProgram>(application);
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
        .mutate(PlanarSourceAdjustment {
            scope_key: SCOPE.to_owned(),
            replacement_y: length(y),
        })
        .expect_source(observed.observed_sources()[0].clone())
        .idempotency(&command)
        .execute_performed::<HistoryProgram, OracleRoot>(application)
        .expect("the scope's ordinate moves");
}

// These are observed full-run charges from the existing execution observer,
// never an inventory of inputs filled by the handler.
thread_local! {
    static REPORTS: std::cell::RefCell<Vec<Option<u64>>> = const { std::cell::RefCell::new(Vec::new()) };
}
