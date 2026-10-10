//! The build guide's owners run inside the fixture's ordinary mutation host.

use super::demand::{
    RegionTotalsDemand, RegionTotalsDemandBinding, RegionTotalsHandler, TotalRegions,
};
use super::*;
use crate::{sensor_telemetry as telemetry, warehouse_inventory as inventory};
use std::cell::RefCell;
use worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome;

// The five caller-supplied types use the existing computation fixture's declarations.
type S = CheckpointSchema;
type F = PlanarFinalOutputFeature;
type A = demand_policy::FinalArtifact;
type Op = TotalRegions;
type B = RegionTotalsDemandBinding<S>;

#[derive(Debug, PartialEq)]
enum Reduced {
    Inventory(f64),
    Telemetry(u64),
}

thread_local! {
    static RESULTS: RefCell<Vec<Reduced>> = const { RefCell::new(Vec::new()) };
}

trait Example: 'static {
    type Computation: ApplicationManagedComputation<S, F>;
    fn declare() -> ApplicationFeatureSpec;
    fn install(setup: &mut owner::Setup<'_>) -> owner::Installed;
}

struct Warehouse;
impl Example for Warehouse {
    type Computation = inventory::InventoryTotal<A>;
    fn declare() -> ApplicationFeatureSpec {
        inventory::declare_inventory::<S, F, A>()
    }
    fn install(setup: &mut owner::Setup<'_>) -> owner::Installed {
        let installed = inventory::install_inventory::<S, F, A, Op>(setup)?;
        Ok(RegionTotalsHandler::running(move |reader, _| {
            let (total, _) = inventory::run_inventory::<S, F, A, Op, B>(
                &installed,
                reader,
                &inventory::sample_input(),
            )
            .expect("the warehouse example completes");
            RESULTS.with(|results| results.borrow_mut().push(Reduced::Inventory(total)));
        }))
    }
}

struct Telemetry;
impl Example for Telemetry {
    type Computation = telemetry::PeakReading<A>;
    fn declare() -> ApplicationFeatureSpec {
        telemetry::declare_peak::<S, F, A>()
    }
    fn install(setup: &mut owner::Setup<'_>) -> owner::Installed {
        let installed = telemetry::install_peak::<S, F, A, Op>(setup)?;
        Ok(RegionTotalsHandler::running(move |reader, _| {
            let (peak, _) = telemetry::run_peak::<S, F, A, Op, B>(
                &installed,
                reader,
                &telemetry::sample_input(),
            )
            .expect("the telemetry example completes");
            RESULTS.with(|results| results.borrow_mut().push(Reduced::Telemetry(peak)));
        }))
    }
}

struct Contribution<E>(PhantomData<fn() -> E>);
impl<E: Example> ApplicationSchemaContribution<S> for Contribution<E> {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        <TopologyContribution as ApplicationSchemaContribution<S>>::IDENTITY;
    fn register_members(
        builder: ApplicationSchemaDeclarationBuilder<S>,
    ) -> ApplicationSchemaDeclarationBuilder<S> {
        <TopologyContribution as ApplicationSchemaContribution<S>>::register_members(builder)
    }
}
impl<E: Example> WorthQueryApplicationContribution<S> for Contribution<E> {
    type Configuration = TopologyConfiguration;
    fn contracts(
        contracts: &mut WorthQueryApplicationContributionContracts<S>,
    ) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
        <TopologyContribution as WorthQueryApplicationContribution<S>>::contracts(contracts)
    }
    fn configure(
        configuration: Self::Configuration,
        setup: &mut owner::Setup<'_>,
    ) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial> {
        let handler = E::install(setup)?;
        setup.handler::<B, _>(handler)?;
        idle_entry_handlers(setup)?;
        TopologyContribution::configure_topology(configuration, setup)
    }
}

struct Program<E>(PhantomData<fn() -> E>);
impl<E: Example> ApplicationProgramDefinition<S> for Program<E> {
    type Contributions = (Contribution<E>,);
    type Outputs = ApplicationProgramOutputs<CheckpointRoot>;
    type Rules = CheckpointRules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("compiled-build-guide-example");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        // The fixture also declares the operations that serve its output graph.
        let final_output = demand_policy::final_output_feature::<A>()
            .managed_computation::<E::Computation>()
            .mutation::<B>()
            .finish();
        assert_eq!(
            final_output.feature(),
            E::declare().feature(),
            "the host installs the example's artifact and computation declarations"
        );
        demand_policy::feature_specs_with_final_output(
            required_chain::output_feature_spec(),
            final_output,
        )
    }
}

fn first_run<E: Example>() -> Reduced {
    let _guard = checkpoint_recovery_test_guard();
    RESULTS.with(|results| results.borrow_mut().clear());
    let application = support::install_program_with_seed::<Program<E>>(
        None,
        Default::default(),
        32,
        128 * 1_024 * 1_024,
        1_000_000,
        |graph| {
            support::seed_cycle(graph);
            facts::seed(graph, &[("example", &[])]);
        },
    );
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let observed = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .expect("the mutation source is readable");
    let outcome = request
        .mutate(RegionTotalsDemand {
            scope_key: "anchor-a".to_owned(),
            entries: "example".to_owned(),
            replacement_y: length(3),
        })
        .expect_source(observed.observed_sources()[0].clone())
        .idempotency(&1_u64)
        .execute_in_program::<Program<E>>(
            &application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        );
    assert!(
        matches!(
            &outcome,
            Ok(WorthQueryApplicationMutationOutcome::Committed { .. })
        ),
        "the example's host mutation commits: {outcome:?}"
    );
    RESULTS.with(|results| {
        let mut results = results.borrow_mut();
        assert_eq!(results.len(), 1, "one first run reaches the example");
        results.pop().unwrap()
    })
}

#[test]
fn compiled_examples_warehouse_inventory_total() {
    assert_eq!(first_run::<Warehouse>(), Reduced::Inventory(13.0));
}

#[test]
fn compiled_examples_sensor_telemetry_peak() {
    assert_eq!(first_run::<Telemetry>(), Reduced::Telemetry(850));
}
