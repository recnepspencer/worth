//! One registered native participant prepares on two distinct branch cells.

use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use worth_foundational::facade::{AspectFieldLocator, AspectValue, InternedString};
use worth_relational::facade::{
    history::BranchId,
    mvcc::{
        CompanionBranchCell, CompanionPreflightBudget, CompanionPreflightStop,
        CompanionPublicationCompletion, CompanionPublicationCompletionObserver,
        PreparedPublicationCompanionEffect, PreparedRelationalCommitCandidate,
        PublicationCompanionPreflight, RelationalPublicationCompanion,
        RelationalPublicationDeferred, RelationalPublicationOutcome,
    },
    transactions::{
        AspectFieldPatch, EntityMutationIntent, MutationIntent, UpdateEntityFieldsIntent,
        WorkerIntentBatch,
    },
};

use super::*;
use crate::domain_computation::execution_runtime::source_invalidation::{
    RetainedInvalidationCapacity, WorthQueryInvalidationResourceInstallation,
    WorthQueryInvalidationResources,
};
use crate::domain_computation::primary_graph::application_output_demand::{
    RequiredWorkMembership, SelectedRequiredWorkKind, WorthQueryOutputDemandRegistry,
};

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
        let observer = effect.attach_completion_observer(context, retained)?;
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
        let branch_bytes = context.branch_id().0.len() as u64;
        if index == 1 {
            self.gate.main_linked.wait()?;
        }
        let hint_bytes = RequiredWorkMembership::native_hint_bytes();
        let retained_branch_bytes =
            RequiredWorkMembership::native_branch_retained_bytes(context.branch_id().0.len())
                .unwrap();
        let prepared = (|| {
            context.claim_bytes(hint_bytes)?;
            context.claim_bytes(retained_branch_bytes)?;
            context.claim_work(branch_bytes + 7)?;
            let hint_capacity =
                super::super::retention::reserve(&self.resources, hint_bytes, context)?;
            let branch_capacity =
                super::super::retention::reserve(&self.resources, retained_branch_bytes, context)?;
            let branch = RequiredWorkMembership::prepared_native_branch(
                context.branch_id().clone(),
                branch_capacity,
            );
            Ok::<_, CompanionPreflightStop>(RequiredWorkMembership::prepared_native_hint(
                observer,
                branch,
                hint_capacity,
            ))
        })();
        let stop = match prepared {
            Ok(hint) => {
                self.member.prepare_hint(hint);
                None
            }
            Err(stop) => Some(stop),
        };
        if index == 0 {
            self.gate.main_linked.open();
        }
        // Both successful hints are linked before either branch may install.
        // A denied second preparation still releases the other branch.
        self.gate.meet()?;
        if let Some(stop) = stop {
            return Err(stop);
        }
        if index == 1 {
            self.gate.main_published.wait()?;
        }
        Ok(effect)
    }
}

fn prepare_update(
    runtime: &RelationalRuntime,
    branch: &BranchId,
    entity: EntityId,
    locator: AspectFieldLocator,
) -> PreparedRelationalCommitCandidate {
    let identity = runtime.branch_identity(branch).unwrap();
    let basis = runtime.admit_branch_basis(&identity).unwrap();
    let mut transaction = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .unwrap();
    transaction
        .push_batch(
            WorkerIntentBatch::new("two-native-hints").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: entity,
                    fields: AspectFieldPatch::from(BTreeMap::from([(
                        locator,
                        AspectValue::String(InternedString::Raw("closed".to_owned())),
                    )])),
                }),
            )),
        )
        .unwrap();
    runtime.prepare_branch_transaction(transaction).unwrap()
}

#[test]
fn two_branch_cells_preserve_prepared_fences_and_refund_partial_denial() {
    for allowed_hints in [1_u64, 2] {
        let world = installed_authorization_world(true);
        let selected = world.selected_product();
        let entity = selected
            .resolve_entity(
                AccountStatus::reference(),
                "open".to_owned(),
                &live_scope(),
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap()
            .entity_id();
        let (_, product, _) = selected.into_parts();
        let registry = WorthQueryOutputDemandRegistry::default();
        let (interest, member) = registry
            .fixture_admitted_work_membership(product.observation().lifecycle_incarnation());
        let hint_header = RequiredWorkMembership::native_hint_bytes();
        let ticket_bytes =
            super::super::index_capacity::arc_bytes::<RetainedInvalidationCapacity>().unwrap();
        let main_hint = hint_header
            + RequiredWorkMembership::native_branch_retained_bytes("main".len()).unwrap()
            + 2 * ticket_bytes;
        let fork_hint = hint_header
            + RequiredWorkMembership::native_branch_retained_bytes("hint-fork".len()).unwrap()
            + 2 * ticket_bytes;
        let one_hint = fork_hint;
        let resources = WorthQueryInvalidationResources::install(
            WorthQueryInvalidationResourceInstallation::bounded(
                1_000_000,
                8 * 1024 * 1024,
                one_hint * allowed_hints,
                8,
            ),
        )
        .unwrap();
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
        handle.with_open_runtime_mut(|runtime| {
            let main = BranchId("main".to_owned());
            let fork = BranchId("hint-fork".to_owned());
            let (_, source) = runtime.observe_fork_source(&main).unwrap();
            runtime.fork_branch(fork.clone(), source).unwrap();
            let (_main_handle, main_root) = snapshot(runtime);
            let fork_identity = runtime.branch_identity(&fork).unwrap();
            let fork_basis = runtime.admit_branch_basis(&fork_identity).unwrap();
            let fork_handle = runtime
                .snapshots()
                .snapshot_for_observation(&fork_basis.observation())
                .unwrap();
            let fork_root = runtime.read_truth().positioned_snapshot(&fork_handle).unwrap();
            let port = runtime.publication_companion_port();
            let pending = port.begin_required_registration().unwrap();
            let main_cell = pending.mint_branch_cell(&main_root, Arc::new(0_u64)).unwrap();
            let fork_cell = pending.mint_branch_cell(&fork_root, Arc::new(0_u64)).unwrap();
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
                .activate(
                    participant,
                    CompanionPreflightBudget {
                        maximum_work_visits: 1_000_000,
                        maximum_preparation_bytes: 8 * 1024 * 1024,
                    },
                )
                .unwrap();
            let main_candidate = prepare_update(runtime, &main, entity, locator.clone());
            let fork_candidate = prepare_update(runtime, &fork, entity, locator.clone());
            let main_port = runtime.publication_port();
            let fork_port = runtime.publication_port();
            let outcomes = std::thread::scope(|scope| {
                let main_write = scope.spawn(|| {
                    let outcome = main_port.compare_and_publish(main_candidate);
                    gate.main_published.open();
                    outcome
                });
                let fork_write = scope.spawn(move || fork_port.compare_and_publish(fork_candidate));
                [main_write.join().unwrap(), fork_write.join().unwrap()]
            });
            assert!(saw_two_prepared.load(Ordering::Relaxed));
            let observed = observers.lock().unwrap();
            assert_eq!(observed.len(), 2);
            assert_eq!(
                observed
                    .iter()
                    .filter(|observer| observer.state() == CompanionPublicationCompletion::Installed)
                    .count() as u64,
                allowed_hints,
                "{outcomes:?}"
            );
            assert_eq!(
                observed
                    .iter()
                    .filter(|observer| observer.state() == CompanionPublicationCompletion::Aborted)
                    .count() as u64,
                2 - allowed_hints
            );
            drop(observed);
            let mut performed = 0;
            for outcome in outcomes {
                match outcome {
                    RelationalPublicationOutcome::Performed(published) => {
                        performed += 1;
                        let committed = runtime.settle_performed_publication(published).unwrap();
                        release_test_commit_snapshot(runtime, &committed);
                    }
                    RelationalPublicationOutcome::Deferred(
                        RelationalPublicationDeferred::CompanionPreflight(
                            CompanionPreflightStop::RetainedCompanionCapacityExhausted { .. },
                        ),
                    ) => {}
                    _ => panic!("only native hint retention can stop the selected write"),
                }
            }
            assert_eq!(performed, allowed_hints);
            assert_eq!(
                resources.retained_capacity_bytes(),
                main_hint + fork_hint * (allowed_hints - 1)
            );
            let selected_main = registry
                .next_required_work_for_selected(
                    product.read_lease_ref(),
                    &main_root,
                    owner,
                    &mut owner.edit_admission(),
                )
                .unwrap()
                .expect("main-cell hint remains selectable behind fork-cell hint");
            assert!(matches!(
                selected_main.kind(),
                SelectedRequiredWorkKind::Native { branch, .. } if branch.branch_id() == &main
            ));
            assert!(selected_main.acknowledge().is_some());
            port.remove_required(&registration).unwrap();
        });
        assert_eq!(
            resources.retained_capacity_bytes(),
            fork_hint * (allowed_hints - 1)
        );
        if allowed_hints == 2 {
            let first = registry
                .next_required_work(owner, &mut owner.edit_admission())
                .unwrap()
                .expect("fork-cell hint remains after main acknowledgement");
            assert!(matches!(
                first.kind(),
                SelectedRequiredWorkKind::Native { branch, .. }
                    if branch.branch_id() == &BranchId("hint-fork".to_owned())
            ));
            registry.fixture_requeue_admitted_work(&interest);
            let stale = registry
                .next_required_work(owner, &mut owner.edit_admission())
                .unwrap()
                .expect("one native hint can have two selected readers");
            assert!(matches!(
                stale.kind(),
                SelectedRequiredWorkKind::Native { branch, .. }
                    if branch.branch_id() == &BranchId("hint-fork".to_owned())
            ));
            assert!(first.acknowledge().is_some());
            assert_eq!(
                resources.retained_capacity_bytes(),
                RequiredWorkMembership::native_branch_retained_bytes("hint-fork".len()).unwrap()
                    + ticket_bytes
            );
            drop(stale);
            assert_eq!(resources.retained_capacity_bytes(), 0);
        }
        let initial = registry
            .next_required_work(owner, &mut owner.edit_admission())
            .unwrap()
            .expect("the original required interest is unresolved");
        assert!(matches!(
            initial.kind(),
            SelectedRequiredWorkKind::UnresolvedInitial
        ));
        assert!(initial.acknowledge().is_some());
        assert!(registry
            .next_required_work(owner, &mut owner.edit_admission())
            .unwrap()
            .is_none());
        drop(interest);
        assert_eq!(resources.retained_capacity_bytes(), 0);
    }
}
