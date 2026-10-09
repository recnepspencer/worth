//! Oracle program installation, with an explicit producer input-reuse contract.

use super::*;

/// The topology plus the region output's producer and the owner of its
/// totals.
pub(super) struct OracleContribution<
    const REUSE: bool,
    const WORK: usize,
    const RUNS: usize,
    const MODE: u8,
>;
impl<const REUSE: bool, const WORK: usize, const RUNS: usize, const MODE: u8>
    ApplicationSchemaContribution<CheckpointSchema>
    for OracleContribution<REUSE, WORK, RUNS, MODE>
{
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
impl<const REUSE: bool, const WORK: usize, const RUNS: usize, const MODE: u8>
    WorthQueryApplicationContribution<CheckpointSchema>
    for OracleContribution<REUSE, WORK, RUNS, MODE>
{
    type Configuration = TopologyConfiguration;

    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<CheckpointSchema>,
    ) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
        <TopologyContribution as WorthQueryApplicationContribution<CheckpointSchema>>::contracts(
            contracts,
        )?;
        contracts.producer::<RegionOutputProducer<CheckpointSchema, REUSE, MODE>>()?;
        contracts.conditional::<RegionOutputReadiness<CheckpointSchema, REUSE, MODE>>()?;
        Ok(())
    }

    fn configure(
        configuration: Self::Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, CheckpointSchema>,
    ) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
        let installed = setup
            .partitioned_computation::<PlanarFinalOutputFeature, OracleTotals<WORK>, _>(
                OracleOwner::<MODE>,
            )?;
        let handler = RegionOutputHandler::running(move |reader, set| {
            let mut last_value = None;
            for _ in 0..RUNS {
                worth_query_host::facade::primary_graph::full_partitioned_computation_preparations_on_this_thread_for_test();
                take_calls();
                COMBINES.store(0, Ordering::Relaxed);
                tree_count::reset();
                worth_query_host::facade::primary_graph::partitioned_computation_tree_work_on_this_thread_for_test();
                let outcome = (|| {
                    let prepared = if MODE == 3 || super::differential::reference::model_replay() {
                        installed.prepare_without_reuse_for_test(reader, set)?
                    } else if super::branch_sharing::reinstallation_requested() {
                        installed.prepare_after_reinstallation_for_test(reader, set)?
                    } else {
                        installed.prepare(reader, set)?
                    };
                    let computed = prepared.compute(reader.managed_computation_execution())?;
                    let charged_work = computed.charged_work();
                    Ok((computed.complete()?.to_bits(), charged_work))
                })();
                last_value = outcome.as_ref().ok().map(|(bits, _)| *bits);
                let runs = runs_on_this_thread().into_iter().map(|(run, _)| run);
                room().push(OracleRun {
                    published: Vec::new(), full_preparations: worth_query_host::facade::primary_graph::full_partitioned_computation_preparations_on_this_thread_for_test(),
                    outcome,
                    runs: runs.collect(),
                    calls: take_calls(),
                    tree_runs: worth_query_host::facade::primary_graph::partitioned_computation_tree_work_on_this_thread_for_test(),
                    combines: COMBINES.swap(0, Ordering::Relaxed),
                tree_nodes: tree_count::take_nodes(),
                placement: tree_count::placement(),
                });
            }
            last_value
        });
        let handler = if MODE == 2 {
            handler.with_generated_payload()
        } else {
            handler.preserving_payload()
        };
        setup.handler::<RegionOutputBinding<CheckpointSchema>, _>(handler)?;
        let unretained =
            owner::running::<program::UnretainedTotals<WORK>, _>(setup, owner::RegionTotalsOwner)?;
        setup.handler::<RegionTotalsDemandBinding<CheckpointSchema>, _>(unretained)?;
        setup.handler::<EntryEditBinding<CheckpointSchema>, _>(EntryEditHandler)?;
        super::super::entry_correction::configure(setup)?;
        setup.producer::<RegionOutputProducer<CheckpointSchema, REUSE, MODE>>(
            RegionOutputProvider::<MODE>,
        )?;
        setup.conditional::<RegionOutputReadiness<CheckpointSchema, REUSE, MODE>>(())?;
        if MODE == 4 {
            let provider = crate::InitialPlanarProvider::new(
                std::sync::Arc::clone(&configuration.producer_authorization_denials),
                std::sync::Arc::clone(&configuration.producer_domain_denial),
            )
            .with_uniform_decimal_key_width();
            TopologyContribution::configure_topology_with_provider(configuration, setup, provider)
        } else {
            TopologyContribution::configure_topology(configuration, setup)
        }
    }
}

pub(super) struct OracleProgram<
    const REUSE: bool = false,
    const WORK: usize = TOTALS_WORK,
    const RUNS: usize = 1,
    const MODE: u8 = 0,
>;
impl<const REUSE: bool, const WORK: usize, const RUNS: usize, const MODE: u8>
    ApplicationProgramDefinition<CheckpointSchema> for OracleProgram<REUSE, WORK, RUNS, MODE>
{
    type Contributions = (OracleContribution<REUSE, WORK, RUNS, MODE>,);
    type Outputs = ApplicationProgramOutputs<OracleRoot>;
    type Rules = CheckpointRules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("checkpoint-region-output-program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        demand_policy::feature_specs_with_final_output(
            required_chain::output_feature_spec(),
            demand_policy::final_output_feature::<RegionArtifact>()
                .managed_computation::<OracleTotals<WORK>>()
                .managed_computation::<program::UnretainedTotals<WORK>>()
                .conditional_operation::<TotalRegionOutput>()
                .mutation::<EntryEditBinding<CheckpointSchema>>()
                .mutation::<super::super::entry_correction::EntryCorrectionBinding<CheckpointSchema>>()
                .mutation::<super::super::entry_correction::EntryCorrectionBinding<CheckpointSchema, true>>()
                .mutation::<RegionTotalsDemandBinding<CheckpointSchema>>()
                .finish(),
        )
    }
}

pub(super) type Application<
    const REUSE: bool = false,
    const WORK: usize = TOTALS_WORK,
    const RUNS: usize = 1,
    const MODE: u8 = 0,
> = application_installation::WorthQueryProgramApplicationRuntime<
    CheckpointSchema,
    OracleProgram<REUSE, WORK, RUNS, MODE>,
>;
pub(super) type Request<'application, 'principal, 'scope> =
    WorthQueryApplicationRequest<'application, 'principal, 'scope, CheckpointSchema>;

/// The scope whose output the producer keeps. Its ordinate names the set.
pub(super) const SCOPE: &str = "anchor-isolated";
/// The scope's seeded ordinate, which names the even set, and the one that
/// names the odd set.
pub(super) const RETAINED_POSITIONS: usize = 4;
pub(super) const EVEN_Y: u64 = 50;
pub(super) const ODD_Y: u64 = 51;

/// Installs the default host, widened so its operations may be as wide as
/// the largest set's decision.
pub(super) fn install(seed: impl FnOnce(&mut Graph)) -> Application {
    install_with_reuse::<false>(seed)
}

pub(super) fn install_with_reuse<const REUSE: bool>(
    seed: impl FnOnce(&mut Graph),
) -> Application<REUSE> {
    install_configured::<REUSE, TOTALS_WORK, 1>(None, Default::default(), seed)
}

pub(super) fn install_variant<
    const REUSE: bool,
    const WORK: usize,
    const RUNS: usize,
    const MODE: u8,
>(
    checkpoint: Option<application_installation::WorthQueryApplicationCheckpoint>,
    profile: worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile,
    seed: impl FnOnce(&mut Graph),
) -> Application<REUSE, WORK, RUNS, MODE> {
    install_variant_with_observations::<REUSE, WORK, RUNS, MODE>(checkpoint, profile, 0, seed)
}

pub(super) fn install_variant_with_observations<
    const REUSE: bool,
    const WORK: usize,
    const RUNS: usize,
    const MODE: u8,
>(
    checkpoint: Option<application_installation::WorthQueryApplicationCheckpoint>,
    profile: worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile,
    additional_observations: u64,
    seed: impl FnOnce(&mut Graph),
) -> Application<REUSE, WORK, RUNS, MODE> {
    let width = u64::try_from(DECISION_FACT_BUDGET + WIDTH_BESIDE_DECISION).unwrap();
    let host = support::candidates();
    let candidates =
        worth_query_host::facade::runtime::WorthQueryApplicationCandidateResourceProfile::physical_resources(
            host.maximum_items().max(width),
            host.maximum_retained_representation_bytes()
                .max((1024 * LARGEST_SET + 320 * 256) as u64),
        )
        .and_then(|candidates| candidates.with_maximum_operation_width(width))
        .unwrap();
    const RETAINED_COMMITS: u64 = 32;
    let positions = RETAINED_POSITIONS;
    // The scale host gives all retention owners the same ample ceiling;
    // this proof judges request work rather than retained-capacity stops.
    // Preparation can coexist with four complete images.
    let retained_bytes = if additional_observations == 0 {
        128 * 1024 * 1024
    } else {
        1 << 40
    };
    let invalidation = worth_query_host::facade::runtime::WorthQueryInvalidationResources::install(
        worth_query_host::facade::runtime::WorthQueryInvalidationResourceInstallation::bounded(
            1_000_000,
            4 * retained_bytes,
            retained_bytes,
            positions,
        ),
    )
    .expect("four retained image bounds fit preparation");
    let limits = support::limits_with_history_room(
        RETAINED_COMMITS + additional_observations,
        16 + additional_observations,
        64 + 3 * additional_observations,
        invalidation,
        candidates,
        524_288 + 4_096 * additional_observations,
    );
    support::install_program_with_limits::<OracleProgram<REUSE, WORK, RUNS, MODE>>(
        checkpoint,
        profile,
        limits,
        |graph| {
            support::seed_cycle(graph);
            seed(graph);
        },
    )
}

pub(super) fn install_configured<const REUSE: bool, const WORK: usize, const RUNS: usize>(
    checkpoint: Option<application_installation::WorthQueryApplicationCheckpoint>,
    profile: worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile,
    seed: impl FnOnce(&mut Graph),
) -> Application<REUSE, WORK, RUNS> {
    install_variant::<REUSE, WORK, RUNS, 0>(checkpoint, profile, seed)
}
