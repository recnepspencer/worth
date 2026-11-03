//! Independent model history preserves Native bindings without reading the tested runtime.
use super::*;
use std::any::Any;
use std::cell::{Cell, RefCell};
use worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile;

// The four-round independent model captured 77 checkpoints through 162 events.
// Its original request peaked at 19,484,190 Work; shared refusal Worlds stay at 8m.
const FULL_RUN_HISTORY_POLICY: worth_foundational::ExecutionRequestPolicy =
    worth_foundational::ExecutionRequestPolicy::new(
        worth_foundational::ExecutionPosture::Serial,
        worth_foundational::DeterminismContract::CanonicalBitwise,
        worth_foundational::ExecutionBudget::new(
            std::num::NonZeroUsize::MIN,
            None,
            Some(20_000_000),
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
    application: Box<dyn Any>,
    applied: usize,
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
    /// The independent model runtime applies each model command once. Each
    /// comparison installs a fresh runtime from that model's Native truth alone;
    /// it never reads the tested runtime's records or snapshots.
    /// Source entity generations need their creation/deletion history. Generated
    /// output allocations additionally need the model's earlier demands, run
    /// without computation reuse; the final runtime receives Native truth only.
    pub(in super::super) fn install<
        const REUSE: bool,
        const WORK: usize,
        const RUNS: usize,
        const MODE: u8,
    >(
        &self,
        profile: WorthQueryOutputDemandResourceProfile,
        seed: impl FnOnce(&mut Graph, &Model),
    ) -> Application<REUSE, WORK, RUNS, MODE> {
        let execution_policy = if MODE == 2 {
            model_history_policy()
        } else {
            FULL_RUN_HISTORY_POLICY
        };
        let mut model = self.model.borrow_mut();
        if !model.as_ref().is_some_and(|model| {
            model
                .application
                .is::<Application<REUSE, WORK, RUNS, MODE>>()
        }) {
            *model = Some(ModelRuntime {
                application: Box::new(installation::install_variant_with_observations::<
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
                )),
                applied: 0,
            });
        }
        let model = model
            .as_mut()
            .expect("the independently seeded model runtime exists");
        let application = model
            .application
            .downcast_ref::<Application<REUSE, WORK, RUNS, MODE>>()
            .expect("the model runtime's concrete program was selected above");
        let (scope, principal) = authenticate(application);
        let request = application.request(&principal, &scope);
        let mut command = 0x69_f000 + model.applied as u64;
        MODEL_REPLAY.set(true);
        let replay = ModelReplay;
        for event in &self.history[model.applied..] {
            command += 1;
            match event {
                Event::Edit(Change::Entry(change)) => {
                    edit(&request, application, change.clone(), command)
                }
                Event::Edit(Change::Ordinate(y)) => adjust(&request, application, *y, command),
                Event::Demand(write) if MODE == 2 => {
                    arm_own_write(*write);
                    demand(&request, application);
                    arm_own_write(None);
                }
                Event::Demand(Some(write)) => edit(
                    &request,
                    application,
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
        model.applied = self.history.len();
        let native = application
            .with_host_advancement(|host_phase| {
                application
                    .capture_native_truth_checkpoint_for_test(&host_phase)
                    .unwrap()
            })
            .expect("installed host call opens");
        installation::install_variant_with_observations::<REUSE, WORK, RUNS, MODE>(
            Some(native),
            profile,
            0,
            execution_policy,
            |_| panic!("independent Native model truth does not reseed"),
        )
    }
}

// Generated payloads retain their Native creation/deletion history. Its positive
// checkpoint journey funds content closure, encoding and integrity work separately
// from the logical partition ceilings; the shared refusal World stays at 8m.
pub(in super::super) fn model_history_policy() -> worth_foundational::ExecutionRequestPolicy {
    worth_foundational::ExecutionRequestPolicy::new(
        worth_foundational::ExecutionPosture::Serial,
        worth_foundational::DeterminismContract::CanonicalBitwise,
        worth_foundational::ExecutionBudget::new(
            std::num::NonZeroUsize::MIN,
            None,
            Some(16_000_000),
        ),
    )
}
