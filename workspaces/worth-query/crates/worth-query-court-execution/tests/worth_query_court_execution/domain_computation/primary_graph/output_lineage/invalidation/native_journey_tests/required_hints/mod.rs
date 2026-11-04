//! One registered native participant prepares on two distinct branch cells.

pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::required_work::RequiredWorkWorthQueryOutputDemandRegistryForTest;

pub use std::fmt;
pub use std::sync::atomic::AtomicBool;
pub use std::sync::atomic::Ordering;
pub use std::sync::Arc;
pub use std::sync::Mutex;

pub use worth_foundational::facade::AspectFieldLocator;
pub use worth_foundational::facade::AspectValue;
pub use worth_foundational::facade::InternedString;
pub use worth_relational::facade::history::BranchId;
pub use worth_relational::facade::mvcc::CompanionBranchCell;
pub use worth_relational::facade::mvcc::CompanionPreflightStop;
pub use worth_relational::facade::mvcc::CompanionPublicationCompletion;
pub use worth_relational::facade::mvcc::CompanionPublicationCompletionObserver;
pub use worth_relational::facade::mvcc::PreparedPublicationCompanionEffect;
pub use worth_relational::facade::mvcc::PreparedRelationalCommitCandidate;
pub use worth_relational::facade::mvcc::PublicationCompanionPreflight;
pub use worth_relational::facade::mvcc::RelationalPublicationCompanion;
pub use worth_relational::facade::mvcc::RelationalPublicationOutcome;
pub use worth_relational::facade::transactions::AspectFieldPatch;
pub use worth_relational::facade::transactions::EntityMutationIntent;
pub use worth_relational::facade::transactions::MutationIntent;
pub use worth_relational::facade::transactions::UpdateEntityFieldsIntent;
pub use worth_relational::facade::transactions::WorkerIntentBatch;

pub use crate::domain_computation::primary_graph::output_lineage::invalidation::native_journey_tests::*;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::required_work::RequiredWorkMembership;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::required_work::SelectedRequiredWorkKind;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryOutputDemandRegistry;

pub use crate::domain_computation::primary_graph::output_lineage::invalidation::native_journey_tests::required_hints::gate::TwoPrepareGate;

pub struct TwoCellHints {
    pub cells: BTreeMap<BranchId, CompanionBranchCell<u64>>,
    pub member: Arc<RequiredWorkMembership>,
    pub observers: Arc<Mutex<Vec<CompanionPublicationCompletionObserver>>>,
    pub saw_two_prepared: Arc<AtomicBool>,
    pub gate: Arc<TwoPrepareGate>,
}

impl fmt::Debug for TwoCellHints {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("TwoCellHints").finish()
    }
}

impl RelationalPublicationCompanion for TwoCellHints {
    fn prepare(
        &self,
        context: &mut PublicationCompanionPreflight<'_>,
    ) -> Result<PreparedPublicationCompanionEffect, CompanionPreflightStop> {
        let index = usize::from(context.branch_id().0.as_str() != "main");
        let cell = self
            .cells
            .get(context.branch_id())
            .expect("both real branch cells are registered");
        let reserved = cell.reserve_preflight(context)?;
        let mut effect = context.seal_replacement(reserved, Arc::new(index as u64 + 1))?;
        let observer = effect.attach_completion_observer(context)?;
        {
            let mut observers = self.observers.lock().unwrap();
            observers.push(observer.clone());
            if observers.len() == 2
                && observers
                    .iter()
                    .all(|observer| observer.state() == CompanionPublicationCompletion::Prepared)
            {
                self.saw_two_prepared.store(true, Ordering::Relaxed);
            }
        }

        if index == 1 {
            self.gate.main_linked.wait()?;
        }
        let branch =
            RequiredWorkMembership::prepared_native_branch_for_test(context.branch_id().clone());
        self.member
            .prepare_hint_for_test(RequiredWorkMembership::prepared_native_hint_for_test(
                observer, branch,
            ));
        if index == 0 {
            self.gate.main_linked.open();
        }
        // Both successful hints are linked before either branch may install.
        // A denied second preparation still releases the other branch.
        self.gate.meet()?;
        if index == 1 {
            self.gate.main_published.wait()?;
        }
        Ok(effect)
    }
}

pub use crate::domain_computation::primary_graph::output_lineage::invalidation::native_journey_tests::required_hints::prepare_update::prepare_update;

#[test]
fn two_branch_cells_preserve_prepared_fences_and_selected_branch_identity() {
    {
        let allowed_hints = 2_u64;
        {
            let world = installed_authorization_world(true);
            let (entity, product) = world
                .application
                .with_host_advancement(|phase| {
                    let host_custody = phase.test_host_request_for_test();

                    let selected = world.selected_product(
                        &(host_custody.phase_for_application_for_test(&world.application))
                            .test_host_request_for_test(),
                    );
                    let entity = selected
                        .resolve_entity(
                            &host_custody.phase_for_application_for_test(&world.application),
                            AccountStatus::reference(),
                            "open".to_owned(),
                            &live_scope(),
                            WorthQueryPrincipalResolutionMode::Ordinary,
                        )
                        .unwrap()
                        .entity_id_for_test();
                    let (_, product, _) = selected.into_parts_for_test();
                    (entity, product)
                })
                .expect("the fixture source read owns its host call");
            let registry = WorthQueryOutputDemandRegistry::default();
            let (interest, member) = registry.fixture_admitted_work_membership(
                product.observation_for_test().lifecycle_incarnation(),
            );
            world
                .application
                .with_host_advancement(|phase| {
                    let host_custody = phase.test_host_request_for_test();
                    let execution = &host_custody.borrowed_request_for_test();
                    let graph = (*world.application.runtime_for_test()).primary_graph().unwrap();
                    let handle = graph.integration_handle_for_test();
                    let owner = &(*(*handle.source_owner_for_test()).invalidation_owner_for_test());
                    let status_ref = AccountStatus::reference();
                    let locator = graph
                        .layout_for_test()
                        .field_locator_for_test(status_ref.entity(), status_ref.aspect(), status_ref.field())
                        .unwrap()
                        .clone();
                    let observers = Arc::new(Mutex::new(Vec::with_capacity(2)));
                    let saw_two_prepared = Arc::new(AtomicBool::new(false));
                    let gate = Arc::new(TwoPrepareGate::default());
                    handle.with_runtime_mut_for_test(&host_custody.borrowed_request_for_test(), |runtime| {
                let main = BranchId("main".to_owned());
                let fork = BranchId("hint-fork".to_owned());
                let (_, source) = runtime.observe_fork_source(&main).unwrap();
                runtime.fork_branch(execution, fork.clone(), source).unwrap();
                let (_main_handle, main_root) = snapshot(runtime);
                let fork_identity = runtime.branch_identity(&fork).unwrap();
                let fork_basis = runtime.admit_branch_basis(&fork_identity).unwrap();
                let _fork_handle = runtime
                    .snapshots()
                    .snapshot_for_observation(&fork_basis.observation())
                    .unwrap();
                let port = runtime.publication_companion_port();
                let pending = port.begin_required_registration().unwrap();
                let main_cell = pending
                    .with_branch_cell_at_head(
                        runtime,
                        &runtime.main_branch_identity(),
                        Arc::new(0_u64),
                        |cell| cell,
                    )
                    .unwrap();
                let fork_cell = pending
                    .with_branch_cell_at_head(runtime, &fork_identity, Arc::new(0_u64), |cell| cell)
                    .unwrap();
                let participant = Arc::new(TwoCellHints {
                    cells: BTreeMap::from([(main.clone(), main_cell), (fork.clone(), fork_cell)]),
                    member: Arc::clone(&member),
                    observers: Arc::clone(&observers),
                    saw_two_prepared: Arc::clone(&saw_two_prepared),
                    gate: Arc::clone(&gate),
                });
                let registration = pending
                    .activate(participant)
                    .unwrap();
                let main_candidate = prepare_update(execution, runtime, &main, entity, locator.clone());
                let fork_candidate = prepare_update(execution, runtime, &fork, entity, locator.clone());
                let main_port = runtime.publication_port();
                let fork_port = runtime.publication_port();
                let outcomes = std::thread::scope(|scope| {
                    let main_write = scope.spawn(|| {
                        let outcome = main_port.compare_and_publish(execution, main_candidate);
                        gate.main_published.open();
                        outcome
                    });
                    let fork_write =
                        scope.spawn(move || fork_port.compare_and_publish(execution, fork_candidate));
                    [main_write.join().unwrap(), fork_write.join().unwrap()]
                });
                assert!(saw_two_prepared.load(Ordering::Relaxed));
                let observed = observers.lock().unwrap();
                assert_eq!(observed.len(), 2);
                assert_eq!(
                    observed
                        .iter()
                        .filter(|observer| observer.state()
                            == CompanionPublicationCompletion::Installed)
                        .count() as u64,
                    allowed_hints,
                    "{outcomes:?}"
                );
                assert_eq!(
                    observed
                        .iter()
                        .filter(
                            |observer| observer.state() == CompanionPublicationCompletion::Aborted
                        )
                        .count() as u64,
                    2 - allowed_hints
                );
                drop(observed);
                let mut performed = 0;
                for outcome in outcomes {
                    match outcome {
                        RelationalPublicationOutcome::Performed(published) => {
                            performed += 1;
                            let committed =
                                runtime.settle_performed_publication(published).unwrap();
                            release_test_commit_snapshot(runtime, &committed);
                        }
                        // ee07da2b0e retired hint byte-forecast capacity refusal;
                        // both admitted hints must install in this journey.
                        _ => panic!("only native hint retention can stop the selected write"),
                    }
                }
                assert_eq!(performed, allowed_hints);

let mut selected_main = registry
                    .next_required_claim_for_selected_for_test(
                        product.read_lease_ref_for_test(),
                        &main_root,
                        owner,
                    )
                    .unwrap()
                    .expect("main-cell hint remains selectable behind fork-cell hint");



                let selected_main = registry.select_required_claim_for_test(&mut selected_main, &main_root)
                    .unwrap().expect("the main branch claim selects its actual native hint");
                assert!(matches!(
                    selected_main.kind_for_test(),
                    SelectedRequiredWorkKind::Native { branch, .. } if branch.branch_id_for_test() == &main
                ));
                assert!(selected_main.acknowledge_fixture().is_some());
                port.remove_required(&registration).unwrap();
            });

                    if allowed_hints == 2 {


let first = registry
                            .next_required_work_for_test(owner)
                            .unwrap()
                            .expect("fork-cell hint remains after main acknowledgement");
assert!(matches!(
                            first.kind_for_test(),
                            SelectedRequiredWorkKind::Native { branch, .. }
                                if branch.branch_id_for_test() == &BranchId("hint-fork".to_owned())
                        ));
                        registry.fixture_requeue_admitted_work(&interest);
let stale = registry
                            .next_required_work_for_test(owner)
                            .unwrap()
                            .expect("one native hint can have two selected readers");


                        assert!(matches!(
                            stale.kind_for_test(),
                            SelectedRequiredWorkKind::Native { branch, .. }
                                if branch.branch_id_for_test() == &BranchId("hint-fork".to_owned())
                        ));
                        assert!(first.acknowledge_fixture().is_some());

                        drop(stale);

                    }


let initial = registry
                        .next_required_work_for_test(owner)
                        .unwrap()
                        .expect("the original required interest is unresolved");
assert!(matches!(
                        initial.kind_for_test(),
                        SelectedRequiredWorkKind::UnresolvedInitial
                    ));
                    assert!(initial.acknowledge_fixture().is_some());
                    assert!(registry
                        .next_required_work_for_test(owner)
                        .unwrap()
                        .is_none());
                    drop(interest);

                })
                .expect("the required-hints journey owns its installed host call");
        }
    }
}

#[path = "gate/mod.rs"]
pub mod gate;
#[path = "prepare_update/mod.rs"]
pub mod prepare_update;
