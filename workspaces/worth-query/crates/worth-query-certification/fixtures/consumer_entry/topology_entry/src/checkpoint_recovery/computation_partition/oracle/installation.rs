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
        if matches!(
            MODE,
            collection::LIFECYCLE
                | collection::CONFLICTING_READINESS
                | collection::ORPHAN_OPERATION
                | collection::ORPHAN_STAGE
        ) {
            collection::lifecycle::contracts::<MODE>(contracts)?;
        } else {
            contracts.producer::<RegionOutputProducer<CheckpointSchema, REUSE, MODE>>()?;
            contracts.conditional::<RegionOutputReadiness<CheckpointSchema, REUSE, MODE>>()?;
        }
        Ok(())
    }

    fn configure(
        configuration: Self::Configuration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, CheckpointSchema>,
    ) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
        if matches!(
            MODE,
            collection::LIFECYCLE
                | collection::CONFLICTING_READINESS
                | collection::ORPHAN_OPERATION
                | collection::ORPHAN_STAGE
        ) {
            collection::lifecycle::configure::<WORK, MODE>(setup)?;
            setup
                .handler::<RegionOutputBinding<CheckpointSchema>, _>(RegionOutputHandler::idle())?;
        } else {
            collection::lifecycle::idle(setup)?;
            let installed = if MODE == collection::CHECKED {
                setup.derived_collection::<PlanarFinalOutputFeature, collection::RegionCollection<WORK>, _>(OracleOwner::<MODE>)?
            } else if MODE == collection::MISBOUND {
                setup.derived_collection::<PlanarFinalOutputFeature, collection::MisboundCollection<WORK>, _>(OracleOwner::<MODE>)?
            } else {
                setup.partitioned_computation::<PlanarFinalOutputFeature, OracleTotals<WORK>, _>(
                    OracleOwner::<MODE>,
                )?
            };
            let handler = RegionOutputHandler::running(move |reader, set| {
                let mut plan =
                    worth_query_host::facade::primary_graph::WorthQueryDecisionPlan::immediate(
                        worth_query_host::facade::primary_graph::HandlerResult::Completed(None),
                    );
                for _ in 0..RUNS {
                    decision_plan::begin();
                    let prepared = if MODE == 3 || super::differential::reference::model_replay() {
                        installed.prepare_without_reuse_for_test(reader, set)
                    } else if super::branch_sharing::reinstallation_requested() {
                        installed.prepare_after_reinstallation_for_test(reader, set)
                    } else {
                        installed.prepare(reader, set)
                    };
                    plan = plan
                        .zip(decision_plan::prepared(prepared, |_| {}))
                        .map(|(_, value)| value);
                }
                if collection::declared(MODE) {
                    plan.and_then(|value| {
                        match value {
                    Some(value) => {
                        worth_query_host::facade::primary_graph::HandlerResult::Completed(Some(
                            value,
                        ))
                    }
                    None => worth_query_host::facade::primary_graph::HandlerResult::DomainDenied(
                        worth_query_consumer_values::PlanarMutationDenial::MissingCoordinate,
                    ),
                }
                    })
                } else {
                    plan
                }
            });
            let handler = if MODE == 2 {
                handler.with_generated_payload()
            } else {
                handler.preserving_payload()
            };
            setup.handler::<RegionOutputBinding<CheckpointSchema>, _>(handler)?;
            setup.producer::<RegionOutputProducer<CheckpointSchema, REUSE, MODE>>(
                RegionOutputProvider::<MODE>,
            )?;
            setup.conditional::<RegionOutputReadiness<CheckpointSchema, REUSE, MODE>>(())?;
        }
        let unretained =
            owner::running::<program::UnretainedTotals<WORK>, _>(setup, owner::RegionTotalsOwner)?;
        setup.handler::<RegionTotalsDemandBinding<CheckpointSchema>, _>(unretained)?;
        setup.handler::<EntryEditBinding<CheckpointSchema>, _>(EntryEditHandler)?;
        super::super::entry_correction::configure(setup)?;
        if MODE == 4 {
            let provider = crate::InitialPlanarProvider::new(
                std::sync::Arc::clone(&configuration.producer_authorization_denials),
                std::sync::Arc::clone(&configuration.producer_domain_denial),
            )
            .with_uniform_decimal_key_width();
            TopologyContribution::configure_topology_with_provider(
                configuration,
                setup,
                provider,
                crate::PlanarHandler,
            )
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
    fn adoption_authorization<Visitor: worth_query_decl::facade::application_program::ApplicationProgramAdoptionAuthorizationVisitor<CheckpointSchema>>(visitor: Visitor) -> Visitor::Output{
        visitor
            .capability::<super::super::entry_correction::program_adoption::ProgramAdoptionBinding<
                CheckpointSchema,
            >>()
    }
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        let output = demand_policy::final_output_feature::<RegionArtifact>()
                .managed_computation::<OracleTotals<WORK>>()
                .managed_computation::<program::UnretainedTotals<WORK>>()
                .conditional_operation::<TotalRegionOutput>()
                .action::<EntryEditBinding<CheckpointSchema>, super::source_delivery::RetainedEntry>().finish_action()
                .action::<super::super::entry_correction::EntryCorrectionBinding<CheckpointSchema>, super::source_delivery::CorrectEntryCompletion>().finish_action()
                .native_action::<super::super::entry_correction::EntryCorrectionBinding<CheckpointSchema, true>>().finish_action()
                .native_action::<RegionTotalsDemandBinding<CheckpointSchema>>().finish_action();
        let output = if collection::declared(MODE) {
            output.derived_collection::<collection::RegionCollection<WORK>>()
        } else {
            output
        };
        demand_policy::feature_specs_with_final_output(
            required_chain::output_feature_spec(),
            output.finish(),
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
pub(in super::super) const SCOPE: &str = "anchor-isolated";
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
    let candidates =
        worth_query_host::facade::runtime::WorthQueryApplicationCandidateResourceProfile::policy(
            None, None,
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
    let limits = support::observation::limits_with_history_room(
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
