//! One registered native participant prepares on two distinct branch cells.

use std::fmt;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::Mutex;

use worth_foundational::facade::AspectFieldLocator;
use worth_foundational::facade::AspectValue;
use worth_foundational::facade::InternedString;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::mvcc::CompanionBranchCell;
use worth_relational::facade::mvcc::CompanionPreflightStop;
use worth_relational::facade::mvcc::CompanionPublicationCompletion;
use worth_relational::facade::mvcc::CompanionPublicationCompletionObserver;
use worth_relational::facade::mvcc::PreparedPublicationCompanionEffect;
use worth_relational::facade::mvcc::PreparedRelationalCommitCandidate;
use worth_relational::facade::mvcc::PublicationCompanionPreflight;
use worth_relational::facade::mvcc::RelationalPublicationCompanion;
use worth_relational::facade::mvcc::RelationalPublicationDeferred;
use worth_relational::facade::mvcc::RelationalPublicationOutcome;
use worth_relational::facade::transactions::AspectFieldPatch;
use worth_relational::facade::transactions::EntityMutationIntent;
use worth_relational::facade::transactions::MutationIntent;
use worth_relational::facade::transactions::UpdateEntityFieldsIntent;
use worth_relational::facade::transactions::WorkerIntentBatch;

use super::*;
use crate::domain_computation::execution_runtime::source_invalidation::WorthQueryInvalidationResources;
use crate::domain_computation::primary_graph::application_output_demand::RequiredWorkMembership;
use crate::domain_computation::primary_graph::application_output_demand::SelectedRequiredWorkKind;
use crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputDemandRegistry;

#[path = "required_hints/gate.rs"]
mod gate;
use gate::TwoPrepareGate;

struct TwoCellHints {
    cells: BTreeMap<BranchId, CompanionBranchCell<u64>>,
    member: Arc<RequiredWorkMembership>,
    resources: WorthQueryInvalidationResources,
    observers: Arc<Mutex<Vec<CompanionPublicationCompletionObserver>>>,
    saw_two_prepared: Arc<AtomicBool>,
    gate: Arc<TwoPrepareGate>,
    retained: Arc<()>,
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
        let retained: Arc<dyn Send + Sync> = self.retained.clone();
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
        let branch = RequiredWorkMembership::prepared_native_branch(context.branch_id().clone());
        self.member
            .prepare_hint(RequiredWorkMembership::prepared_native_hint(
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

#[path = "required_hints/prepare_update.rs"]
mod prepare_update;
use prepare_update::prepare_update;

#[test]
fn two_branch_cells_preserve_prepared_fences_and_selected_branch_identity() {
    let host_request = crate::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    {
        for allowed_hints in [2_u64] {
            let world = installed_authorization_world(true);
            let (entity, product) = world
                .application
                .with_host_advancement(|phase| {
                    let host_custody = phase.test_host_request();

                    let selected = world.selected_product(
                        &(host_custody.phase_for_application(&world.application))
                            .test_host_request(),
                    );
                    let entity = selected
                        .resolve_entity(
                            &host_custody.phase_for_application(&world.application),
                            AccountStatus::reference(),
                            "open".to_owned(),
                            &live_scope(),
                            WorthQueryPrincipalResolutionMode::Ordinary,
                        )
                        .unwrap()
                        .entity_id();
                    let (_, product, _) = selected.into_parts();
                    (entity, product)
                })
                .expect("the fixture source read owns its host call");
            let registry = WorthQueryOutputDemandRegistry::default();
            let (interest, member) = registry
                .fixture_admitted_work_membership(product.observation().lifecycle_incarnation());
            world
                .application
                .with_host_advancement(|phase| {
                    let host_custody = phase.test_host_request();
                    let resources = WorthQueryInvalidationResources::new();
                    let graph = world.application.runtime.primary_graph().unwrap();
                    let handle = graph.integration_handle();
                    let owner = &handle.source_owner.invalidation_owner;
                    let status_ref = AccountStatus::reference();
                    let locator = graph
                        .layout()
                        .field_locator(status_ref.entity(), status_ref.aspect(), status_ref.field())
                        .unwrap()
                        .clone();
                    let observers = Arc::new(Mutex::new(Vec::with_capacity(2)));
                    let saw_two_prepared = Arc::new(AtomicBool::new(false));
                    let gate = Arc::new(TwoPrepareGate::default());
                    handle.with_runtime_mut(&host_custody.borrowed_request_for_test(), |runtime| {
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
                    resources: resources.clone(),
                    observers: Arc::clone(&observers),
                    saw_two_prepared: Arc::clone(&saw_two_prepared),
                    gate: Arc::clone(&gate),
                    retained: Arc::new(()),
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
                        RelationalPublicationOutcome::Deferred(
                            RelationalPublicationDeferred::CompanionPreflight(
                                CompanionPreflightStop::RetainedCompanionCapacityExhausted {
                                    ..
                                },
                            ),
                        ) => {}
                        _ => panic!("only native hint retention can stop the selected write"),
                    }
                }
                assert_eq!(performed, allowed_hints);

let mut selected_main = registry
                    .next_required_claim_for_selected(
                        product.read_lease_ref(),
                        &main_root,
                        owner,
                    )
                    .unwrap()
                    .expect("main-cell hint remains selectable behind fork-cell hint");



                let selected_main = registry.select_required_claim(&mut selected_main, &main_root)
                    .unwrap().expect("the main branch claim selects its actual native hint");
                assert!(matches!(
                    selected_main.kind(),
                    SelectedRequiredWorkKind::Native { branch, .. } if branch.branch_id() == &main
                ));
                assert!(selected_main.acknowledge_fixture().is_some());
                port.remove_required(&registration).unwrap();
            });

                    if allowed_hints == 2 {


let first = registry
                            .next_required_work(owner)
                            .unwrap()
                            .expect("fork-cell hint remains after main acknowledgement");
assert!(matches!(
                            first.kind(),
                            SelectedRequiredWorkKind::Native { branch, .. }
                                if branch.branch_id() == &BranchId("hint-fork".to_owned())
                        ));
                        registry.fixture_requeue_admitted_work(&interest);
let stale = registry
                            .next_required_work(owner)
                            .unwrap()
                            .expect("one native hint can have two selected readers");


                        assert!(matches!(
                            stale.kind(),
                            SelectedRequiredWorkKind::Native { branch, .. }
                                if branch.branch_id() == &BranchId("hint-fork".to_owned())
                        ));
                        assert!(first.acknowledge_fixture().is_some());

                        drop(stale);

                    }


let initial = registry
                        .next_required_work(owner)
                        .unwrap()
                        .expect("the original required interest is unresolved");
assert!(matches!(
                        initial.kind(),
                        SelectedRequiredWorkKind::UnresolvedInitial
                    ));
                    assert!(initial.acknowledge_fixture().is_some());
                    assert!(registry
                        .next_required_work(owner)
                        .unwrap()
                        .is_none());
                    drop(interest);

                })
                .expect("the required-hints journey owns its installed host call");
        }
    }
}
