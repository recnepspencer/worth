//! Independent model history preserves Native bindings without reading the tested runtime.
use super::*;
use std::any::TypeId;
use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile;

const FULL_RUN_HISTORY_POLICY: worth_foundational::ExecutionRequestPolicy =
    worth_foundational::ExecutionRequestPolicy::new(
        worth_foundational::ExecutionPosture::Serial,
        worth_foundational::DeterminismContract::CanonicalBitwise,
        worth_foundational::ExecutionBudget::new(
            std::num::NonZeroUsize::MIN,
            None,
            Some(128_000_000),
        ),
    );

thread_local! { static MODEL_REPLAY: Cell<bool> = const { Cell::new(false) }; }
pub(in super::super) fn model_replay() -> bool {
    MODEL_REPLAY.get()
}
struct ModelReplay;
impl Drop for ModelReplay {
    fn drop(&mut self) {
        MODEL_REPLAY.set(false);
    }
}

enum Event {
    Edit(Change),
    Demand(Option<OwnWrite>),
}
pub(in super::super) struct Reference {
    seed: Model,
    history: Vec<Event>,
    model: RefCell<Option<ModelRuntime>>,
}
struct ModelRuntime {
    type_id: TypeId,
    directory: PathBuf,
    snapshot: Option<support::directory::ClosedDirectorySnapshot>,
    applied: usize,
}
impl Drop for ModelRuntime {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.directory);
    }
}
impl Reference {
    pub(in super::super) fn new(seed: &Model) -> Self {
        Self {
            seed: seed.clone(),
            history: Vec::new(),
            model: RefCell::new(None),
        }
    }
    pub(in super::super) fn edit(&mut self, change: &Change) {
        self.history.push(Event::Edit(change.clone()));
    }
    pub(in super::super) fn demanded(&mut self, write: Option<OwnWrite>) {
        self.history.push(Event::Demand(write));
    }
    /// The independent model runtime applies each model command once into a
    /// closed Directory Home. Each comparison restores that closed snapshot
    /// in-place into the same Home directory and opens a fresh runtime with a
    /// fresh build identity so no carried producer output cache is reused; it
    /// never reads the tested runtime.
    pub(in super::super) fn install<
        const REUSE: bool,
        const WORK: usize,
        const RUNS: usize,
        const MODE: u8,
    >(
        &self,
        profile: WorthQueryOutputDemandResourceProfile,
        seed: impl FnOnce(&mut Graph, &Model),
    ) -> installation::DirApplication<REUSE, WORK, RUNS, MODE> {
        let execution_policy = if MODE == 2 {
            model_history_policy()
        } else {
            FULL_RUN_HISTORY_POLICY
        };
        let target_type = TypeId::of::<installation::DirApplication<REUSE, WORK, RUNS, MODE>>();
        let mut model = self.model.borrow_mut();
        let (application, applied) = match model.take() {
            Some(mut existing) if existing.type_id == target_type => {
                if let Some(snapshot) = existing.snapshot.take() {
                    snapshot.restore_into(&existing.directory);
                }
                let dir = existing.directory.clone();
                let applied = existing.applied;
                std::mem::forget(existing);
                let app = installation::open_directory_variant_with_observations::<
                    REUSE,
                    WORK,
                    RUNS,
                    MODE,
                >(
                    Some(dir),
                    profile,
                    0,
                    execution_policy,
                    |_| panic!("independent Native model truth does not reseed"),
                );
                (app, applied)
            }
            other => {
                drop(other);
                let app = installation::open_directory_variant_with_observations::<
                    REUSE,
                    WORK,
                    RUNS,
                    MODE,
                >(
                    None,
                    profile,
                    0,
                    execution_policy,
                    |graph| seed(graph, &self.seed),
                );
                (app, 0)
            }
        };
        {
            let (scope, principal) = authenticate(&application);
            let request = application.request(&principal, &scope);
            let mut command = 0x69_f000 + applied as u64;
            MODEL_REPLAY.set(true);
            let replay = ModelReplay;
            for event in &self.history[applied..] {
                command += 1;
                match event {
                    Event::Edit(Change::Entry(change)) => {
                        edit(&request, &application, change.clone(), command)
                    }
                    Event::Edit(Change::Ordinate(y)) => {
                        adjust(&request, &application, *y, command)
                    }
                    Event::Demand(write) if MODE == 2 => {
                        arm_own_write(*write);
                        demand(&request, &application);
                        arm_own_write(None);
                    }
                    Event::Demand(Some(write)) => edit(
                        &request,
                        &application,
                        at_demand_scope(EntryEdit::new(
                            "even",
                            write.number,
                            super::super::super::entry_edit::EntryFact::Value,
                            write.bits,
                        )),
                        command,
                    ),
                    Event::Demand(None) => {}
                }
            }
            drop(replay);
        }
        let closed_dir = application.checkpoint_and_close_for_reopen();
        let snapshot =
            support::directory::ClosedDirectorySnapshot::capture(&closed_dir, "oracle-reference");
        *model = Some(ModelRuntime {
            type_id: target_type,
            directory: closed_dir.clone(),
            snapshot: Some(snapshot),
            applied: self.history.len(),
        });
        let fresh_build = application_installation::ApplicationBuildIdentity::parse(
            "22222222-2222-2222-2222-222222222222",
        )
        .unwrap();
        installation::open_directory_variant_with_build::<REUSE, WORK, RUNS, MODE>(
            Some(closed_dir),
            profile,
            0,
            execution_policy,
            |_| panic!("independent Native model truth does not reseed"),
            fresh_build,
        )
        .without_dir_cleanup()
    }
}

pub(in super::super) fn model_history_policy() -> worth_foundational::ExecutionRequestPolicy {
    worth_foundational::ExecutionRequestPolicy::new(
        worth_foundational::ExecutionPosture::Serial,
        worth_foundational::DeterminismContract::CanonicalBitwise,
        worth_foundational::ExecutionBudget::new(
            std::num::NonZeroUsize::MIN,
            None,
            Some(128_000_000),
        ),
    )
}
