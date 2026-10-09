//! Ten thousand neutral logical entries; one entry reads an editable Native value.
//! Other entries have constant declared data. The edit is a real admitted write.
use super::*;
use worth_execution::{ChargedBytes, PartitionItemId};
use worth_query_decl::facade::application_program::ApplicationComputationPartition;

const PARTITIONS: usize = 10_000;
const WORK: usize = 1 << 30;
const EDITED: u32 = 7;

struct Totals;
impl ApplicationManagedComputation<CheckpointSchema, PlanarFinalOutputFeature> for Totals {
    type Input = RegionEntries;
    type Output = RegionArtifact;
    type Partition = RegionKey;
    type Reuse = NoWarmStart;
    type Stopped = RegionStopped;
    const IDENTITY: &'static str = "courtroom-wide-totals";
    const EXECUTION: ApplicationComputationExecution =
        ApplicationComputationExecution::DeterministicPartitioned;
    const RESOURCES: ApplicationComputationResourceCeiling =
        ApplicationComputationResourceCeiling::new(WORK, 8);
}

#[derive(Clone, serde::Serialize)]
struct Item(u32);
impl ApplicationComputationPartition for Item {
    const IDENTITY: &'static str = "courtroom-wide-entry";
}
impl ChargedBytes for Item {
    fn additional_charged_bytes(&self) -> u64 {
        0
    }
}

struct WideOwner;
impl WorthQueryPartitionedComputationOwner<CheckpointSchema, PlanarFinalOutputFeature, Totals>
    for WideOwner
{
    type Operation = TotalRegionOutput;
    type Item = Item;
    type Gathered = f64;
    type PartitionResult = f64;
    type Output = f64;
    type Stopped = u32;
    fn partitions(
        &self,
        reader: &mut Reader<'_, '_, '_, TotalRegionOutput>,
        set: &Set,
    ) -> Result<WorthQueryComputationPartitionPlan<Item>, InputDenial> {
        PLANS.fetch_add(1, Ordering::Relaxed);
        let count = facts::weight(reader, set)? as u32;
        Ok(WorthQueryComputationPartitionPlan::keyed(
            (0..count).map(Item),
            |item| PartitionItemId(u64::from(item.0)),
        ))
    }
    fn partition_key(
        &self,
        _: &mut Reader<'_, '_, '_, TotalRegionOutput>,
        _: &Set,
        item: &Item,
    ) -> Result<RegionKey, InputDenial> {
        KEYS.fetch_add(1, Ordering::Relaxed);
        Ok(RegionKey(item.0))
    }
    fn gather(
        &self,
        reader: &mut Reader<'_, '_, '_, TotalRegionOutput>,
        set: &Set,
        partition: WorthQueryComputationPartitionMembers<'_, RegionKey, Item>,
    ) -> Result<f64, InputDenial> {
        GATHERS.fetch_add(1, Ordering::Relaxed);
        if partition.key().0 == EDITED {
            let entry = reader
                .relations_from(facts::EntrySetMember::reference(), set)?
                .into_iter()
                .next()
                .unwrap()
                .into_to();
            Ok(f64::from_bits(
                reader
                    .field(&entry, facts::EntryValueBits::reference())?
                    .unwrap(),
            ))
        } else {
            Ok(1.0)
        }
    }
    fn compute_partition(
        &self,
        partition: WorthQueryComputationPartitionView<'_, RegionKey, f64>,
        checkpoint: &mut WorthQueryManagedComputationCheckpoint<'_>,
    ) -> Result<f64, WorthQueryManagedComputationDenial<u32>> {
        KERNELS.fetch_add(1, Ordering::Relaxed);
        checkpoint.advance(1)?;
        Ok(*partition.gathered())
    }
    fn reducer(&self) -> WorthQueryDeterministicReducer<f64> {
        WorthQueryDeterministicReducer::canonical(|| -0.0, tree_count::sum)
    }
    fn complete(&self, value: f64) -> Result<f64, u32> {
        Ok(value)
    }
}

struct Contribution<const FRESH: bool>;
impl<const FRESH: bool> ApplicationSchemaContribution<CheckpointSchema> for Contribution<FRESH> {
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
impl<const FRESH: bool> WorthQueryApplicationContribution<CheckpointSchema>
    for Contribution<FRESH>
{
    type Configuration = TopologyConfiguration;
    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<CheckpointSchema>,
    ) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
        <installation::OracleContribution<false, WORK, 1, 0> as WorthQueryApplicationContribution<
            CheckpointSchema,
        >>::contracts(contracts)
    }
    fn configure(
        configuration: TopologyConfiguration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, CheckpointSchema>,
    ) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
        let installed =
            setup.partitioned_computation::<PlanarFinalOutputFeature, Totals, _>(WideOwner)?;
        setup.handler::<RegionOutputBinding<CheckpointSchema>, _>(RegionOutputHandler::running(move |reader, set| {
            worth_query_host::facade::primary_graph::full_partitioned_computation_preparations_on_this_thread_for_test();
            take_calls(); COMBINES.store(0, Ordering::Relaxed); tree_count::reset();
            worth_query_host::facade::primary_graph::partitioned_computation_tree_work_on_this_thread_for_test();
            let outcome = (|| {
                let prepared = if FRESH { installed.prepare_without_reuse_for_test(reader, set)? } else { installed.prepare(reader, set)? };
                let computed = prepared.compute(reader.managed_computation_execution())?;
                let charged = computed.charged_work();
                Ok((computed.complete()?.to_bits(), charged))
            })();
            let value = outcome.as_ref().ok().map(|(bits, _)| *bits);
            room().push(OracleRun { published: Vec::new(), full_preparations: worth_query_host::facade::primary_graph::full_partitioned_computation_preparations_on_this_thread_for_test(), outcome, runs: runs_on_this_thread().into_iter().map(|(run,_)| run).collect(), calls: take_calls(), tree_runs: worth_query_host::facade::primary_graph::partitioned_computation_tree_work_on_this_thread_for_test(), combines: COMBINES.swap(0, Ordering::Relaxed), tree_nodes: tree_count::take_nodes(), placement: tree_count::placement() });
            value
        }).preserving_payload())?;
        let unretained =
            owner::running::<program::UnretainedTotals<WORK>, _>(setup, owner::RegionTotalsOwner)?;
        setup.handler::<RegionTotalsDemandBinding<CheckpointSchema>, _>(unretained)?;
        setup.handler::<EntryEditBinding<CheckpointSchema>, _>(EntryEditHandler)?;
        super::super::super::entry_correction::configure(setup)?;
        setup.producer::<RegionOutputProducer<CheckpointSchema>>(RegionOutputProvider::<0>)?;
        setup.conditional::<RegionOutputReadiness<CheckpointSchema>>(())?;
        TopologyContribution::configure_topology(configuration, setup)
    }
}
struct Program<const FRESH: bool>;
impl<const FRESH: bool> ApplicationProgramDefinition<CheckpointSchema> for Program<FRESH> {
    type Contributions = (Contribution<FRESH>,);
    type Outputs = ApplicationProgramOutputs<OracleRoot>;
    type Rules = CheckpointRules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("courtroom-wide-program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        demand_policy::feature_specs_with_final_output(
            required_chain::output_feature_spec(),
            demand_policy::final_output_feature::<RegionArtifact>()
                .managed_computation::<Totals>()
                .managed_computation::<program::UnretainedTotals<WORK>>()
                .conditional_operation::<TotalRegionOutput>()
                .mutation::<EntryEditBinding<CheckpointSchema>>()
                .mutation::<RegionTotalsDemandBinding<CheckpointSchema>>()
                .finish(),
        )
    }
}
fn application<const FRESH: bool>(
    value: f64,
) -> application_installation::WorthQueryProgramApplicationRuntime<CheckpointSchema, Program<FRESH>>
{
    let candidates =
        worth_query_host::facade::runtime::WorthQueryApplicationCandidateResourceProfile::bounded(
            8192,
            1024 * 1024,
            8192,
        )
        .unwrap();
    let limits = support::limits_with_room(
        32,
        16,
        64,
        support::invalidation(128 * 1024 * 1024, 1_000_000, 4),
        candidates,
    );
    support::install_program_with_limits::<Program<FRESH>>(
        None,
        Default::default(),
        limits,
        |graph| {
            support::seed_cycle(graph);
            facts::seed_set(graph, "even", PARTITIONS as f64);
            seed_entry(
                graph,
                &["even"],
                EDITED as usize,
                RegionEntry {
                    id: u64::from(EDITED),
                    region: EDITED,
                    value,
                    work: 1,
                    fault: None,
                },
            );
        },
    )
}
fn run<const FRESH: bool>(
    app: &application_installation::WorthQueryProgramApplicationRuntime<
        CheckpointSchema,
        Program<FRESH>,
    >,
) -> OracleRun {
    let (scope, principal) = authenticate(app);
    let request = app.request(&principal, &scope);
    room().clear();
    published_states();
    let mut demand = request
        .demand(RegionOutputDemand(SCOPE.to_owned()))
        .start_dependent_in_program::<Program<FRESH>, RegionConnection>(app)
        .unwrap();
    let WorthQueryApplicationOutputDemandProgress::Settled(settled) =
        demand.advance(&request).unwrap()
    else {
        panic!("one advance settles the wide demand")
    };
    assert_eq!(settled.producer_contacts_in_this_demand(), 1);
    let mut runs = take_runs(None);
    assert_eq!(runs.len(), 1);
    runs.pop().unwrap()
}

#[test]
fn ten_thousand_neutral_partitions_isolate_a_real_one_entry_edit() {
    let _guard = checkpoint_recovery_test_guard();
    let kept = application::<false>(1.0);
    let first = run(&kept);
    assert_eq!(
        first.calls,
        OwnerCalls {
            plans: 1,
            keys: PARTITIONS,
            gathers: PARTITIONS,
            kernels: PARTITIONS
        }
    );
    let (scope, principal) = authenticate(&kept);
    let edited = EntryEdit::new(
        "even",
        u64::from(EDITED),
        super::super::super::entry_edit::EntryFact::Value,
        2.5_f64.to_bits(),
    );
    let edited = at_demand_scope(edited).commanded(0x612_10000);
    let outcome = kept
        .request(&principal, &scope)
        .mutate(edited)
        .without_source()
        .idempotency(&0x612_10000_u64)
        .execute_in_program::<Program<false>>(&kept);
    assert!(
        matches!(
            outcome,
            Ok(WorthQueryApplicationMutationOutcome::Committed { .. })
        ),
        "wide edit {outcome:?}"
    );
    let next = run(&kept);
    let fresh = run(&application::<true>(2.5));
    assert_eq!(
        next.calls,
        OwnerCalls {
            plans: 0,
            keys: 0,
            gathers: 1,
            kernels: 1
        }
    );
    assert_eq!(next.outcome, fresh.outcome);
    assert_eq!(
        next.outcome.as_ref().unwrap().0,
        (PARTITIONS as f64 + 1.5).to_bits()
    );
    assert_eq!(
        next.outcome.as_ref().unwrap().1,
        first.outcome.as_ref().unwrap().1
    );
}
