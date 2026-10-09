//! Program adoption retains the same installed producer and computation owner.
use super::super::differential::alphabet::{Change, Lcg};
use super::*;
use std::sync::Arc;
use worth_query_host::facade::application_entry::WorthQueryApplicationRequestExt;
use worth_query_host::facade::application_installation::WorthQueryProgramOwner;

pub(super) struct Adopted;
impl ApplicationProgramDefinition<CheckpointSchema> for Adopted {
    type Contributions = (installation::OracleContribution<false, TOTALS_WORK, 1, 0>,);
    type Outputs = ApplicationProgramOutputs<OracleRoot>;
    type Rules = CheckpointRules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("courtroom-adopted-program");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        OracleProgram::<false>::feature_specs()
    }
}

pub(super) struct Unowned;
impl ApplicationProgramDefinition<CheckpointSchema> for Unowned {
    type Contributions = (installation::OracleContribution<false, TOTALS_WORK, 1, 0>,);
    type Outputs = ApplicationProgramOutputs<OracleRoot>;
    type Rules = CheckpointRules;
    const IDENTITY: ApplicationProgramIdentity =
        ApplicationProgramIdentity::new("courtroom-program-without-correction");
    fn feature_specs() -> Vec<ApplicationFeatureSpec> {
        demand_policy::feature_specs_with_final_output(
            required_chain::output_feature_spec(),
            demand_policy::final_output_feature::<RegionArtifact>()
                .managed_computation::<OracleTotals<TOTALS_WORK>>()
                .managed_computation::<program::UnretainedTotals<TOTALS_WORK>>()
                .conditional_operation::<TotalRegionOutput>()
                .mutation::<EntryEditBinding<CheckpointSchema>>()
                .mutation::<RegionTotalsDemandBinding<CheckpointSchema>>()
                .finish(),
        )
    }
}

pub(super) fn install_rostered(model: &Model) -> Application {
    let initial = ApplicationProgramAuthoring::<CheckpointSchema, OracleProgram>::begin()
        .validated_program()
        .unwrap();
    let adopted = ApplicationProgramAuthoring::<CheckpointSchema, Adopted>::begin()
        .validated_program()
        .unwrap();
    let roster = application_installation::WorthQueryApplicationProgramRoster::new()
        .support(adopted)
        .support(
            ApplicationProgramAuthoring::<CheckpointSchema, Unowned>::begin()
                .validated_program()
                .unwrap(),
        );
    let configuration = TopologyConfiguration {
        setup_calls: Arc::new(AtomicUsize::new(0)),
        invariant_calls: Arc::new(AtomicUsize::new(0)),
        invariant_probe: Arc::new(AtomicUsize::new(0)),
        producer_authorization_denials: Arc::new(AtomicUsize::new(0)),
        producer_domain_denial: Arc::new(std::sync::atomic::AtomicBool::new(false)),
    };
    let candidates =
        worth_query_host::facade::runtime::WorthQueryApplicationCandidateResourceProfile::physical_resources(
            8192,
            1024 * 1024,
        )
        .unwrap();
    let limits = support::limits_with_room(
        32,
        16,
        64,
        support::invalidation(128 * 1024 * 1024, 1_000_000, 4),
        candidates,
    );
    application_installation::in_memory_rostered_program(initial, roster, CheckpointSchema::declaration().unwrap(), (configuration,), limits, |_bootstrap_phase, graph, installed| {
        let binding = installed.principal_binding(ConsumerPrincipalBinding::reference::<CheckpointSchema>()).unwrap();
        let external = worth_query_host::facade::declaration::authentication::WorthQueryExternalPrincipalIdentity::new("https://checkpoint.invalid/local", "model-owner").unwrap();
        graph.bind_principal(&binding, primary_graph::WorthQueryApplicationPrincipalKey::new("model-owner").unwrap(), 1_u64, external, worth_query_host::facade::declaration::authentication::WorthQueryPrincipalMappingStatus::Enabled)?;
        support::seed_cycle(graph); model.seed(graph); super::super::super::entry_correction::seed_correction_grant(graph, SCOPE); Ok(())
    }).unwrap()
}

fn active(app: &Application) -> (usize, Vec<OracleRun>) {
    let (scope, principal) = authenticate(app);
    let request = app.request(&principal, &scope);
    room().clear();
    published_states();
    let mut demand = request
        .demand(RegionOutputDemand(SCOPE.to_owned()))
        .start_dependent_in_program::<OracleProgram, RegionConnection>(app)
        .unwrap();
    let WorthQueryApplicationOutputDemandProgress::Settled(settled) =
        demand.advance(&request).unwrap()
    else {
        panic!("one advance settles the adopted output")
    };
    let mut runs = take_runs(None);
    if let Some(last) = runs.last_mut() {
        last.published = published_states();
    }
    (settled.producer_contacts_in_this_demand(), runs)
}

#[test]
fn program_adoption_carries_retained_state_and_reuses_only_unchanged_partitions() {
    let _guard = checkpoint_recovery_test_guard();
    for seed in SEEDS {
        let mut rng = Lcg(seed);
        let mut model = Model::new(&mut rng);
        let app = install_rostered(&model);
        let (scope, principal) = authenticate(&app);
        demand(&app.request(&principal, &scope), &app);
        for round in 0..ROUNDS {
            let target = *app.supported_program::<Adopted>().unwrap().owned_revision();
            let programs = app.runtime().request(&principal, &scope).programs();
            let requirements = programs.compare(&target).unwrap();
            let performed = programs
                .adopt(&requirements)
                .prepare(4096)
                .unwrap()
                .publish();
            assert!(matches!(performed, worth_query_host::facade::primary_graph::WorthQueryBranchAdoptionPublicationOutcome::Performed(_)), "adoption must publish");
            let prior = model.clone();
            let step = model.step(Kind::Value, &mut rng);
            for change in step.changes {
                let Change::Entry(change) = change else {
                    panic!("a value edit")
                };
                let outcome = app
                    .request(&principal, &scope)
                    .mutate(change.commanded(0x612_ad0 + round as u64))
                    .without_source()
                    .idempotency(&(0x612_ad0_u64 + round as u64))
                    .execute_in_program::<OracleProgram>(&app, worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation);
                assert!(
                    matches!(
                        outcome,
                        Ok(WorthQueryApplicationMutationOutcome::Committed { .. })
                    ),
                    "adopted edit {outcome:?}"
                );
            }
            // Direct demand selection still presents the initial program (Phase 7.6).
            // Return to that rostered program before demanding; the retained state
            // must survive both adoptions and the edit made under the other owner.
            let initial = *app.installed_program().revision();
            let programs = app.runtime().request(&principal, &scope).programs();
            let requirements = programs.compare(&initial).unwrap();
            let performed = programs
                .adopt(&requirements)
                .prepare(4096)
                .unwrap()
                .publish();
            assert!(matches!(performed, worth_query_host::facade::primary_graph::WorthQueryBranchAdoptionPublicationOutcome::Performed(_)), "adoption must publish");
            let (contacts, runs) = active(&app);
            assert_eq!((contacts, runs.len()), (1, 1));
            assert_eq!(runs[0].calls, model.expected_calls(Some(&prior)));
            let fresh = install(|graph| model.seed(graph));
            let (scope, principal) = authenticate(&fresh);
            let (_, reference) = demand(&fresh.request(&principal, &scope), &fresh);
            assert_eq!(runs[0].outcome, reference[0].outcome);
        }
    }
}
