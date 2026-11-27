//! Re-resolution cost observation (public #429, plan §5): how often a
//! `FamilySelection` fact re-resolved its family and compared a candidate,
//! how many settlement rows a verification compared, and what each
//! certification frame of the closure memo cost. Counts are
//! kept only while an execution observation guard is installed, and end
//! with it.

use std::cell::{Cell, RefCell};

/// Re-resolution work counted on this thread.
#[doc(hidden)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WorthQueryFamilySelectionCostForTest {
    /// `FamilySelection` facts re-resolved at a certification basis.
    pub rewalks: u64,
    /// Family candidates those re-resolutions compared.
    pub candidates_compared: u64,
    /// Settlement rows whose facts a verification compared
    /// (`compare_facts`), a closure memo hit compares none.
    pub rows_compared: u64,
}

thread_local! {
    static COST: Cell<WorthQueryFamilySelectionCostForTest> =
        const { Cell::new(WorthQueryFamilySelectionCostForTest { rewalks: 0, candidates_compared: 0, rows_compared: 0 }) };
    /// The cost of each certification frame, recorded as it closed.
    static FRAMES: RefCell<Vec<WorthQueryFamilySelectionCostForTest>> = const { RefCell::new(Vec::new()) };
    static CLEANUP_REGISTERED: Cell<bool> = const { Cell::new(false) };
}

fn registered() -> bool {
    if !crate::domain_computation::execution_observation::is_installed() {
        return false;
    }
    if CLEANUP_REGISTERED.with(|registered| !registered.replace(true)) {
        crate::domain_computation::execution_observation::register_cleanup(clear_observations);
    }
    true
}

fn count(step: impl FnOnce(&mut WorthQueryFamilySelectionCostForTest) -> &mut u64) {
    if !registered() {
        return;
    }
    let mut cost = COST.get();
    let counter = step(&mut cost);
    *counter = counter
        .checked_add(1)
        .expect("family selection observation counter overflow");
    COST.set(cost);
}

/// A `FamilySelection` fact re-resolved its family.
pub fn record_rewalk() {
    count(|cost| &mut cost.rewalks);
}

/// A re-resolution compared one family candidate.
pub fn record_candidate_compared() {
    count(|cost| &mut cost.candidates_compared);
}

/// A verification compared one settlement row's facts.
pub fn record_row_compared() {
    count(|cost| &mut cost.rows_compared);
}

/// The cost counted so far, to measure a frame from; `None` while no
/// observation guard is installed.
pub fn frame_start() -> Option<WorthQueryFamilySelectionCostForTest> {
    registered().then(|| COST.get())
}

/// A certification frame opened at `start` closed: record what it cost,
/// nested frames included.
pub fn record_frame(start: Option<WorthQueryFamilySelectionCostForTest>) {
    let Some(start) = start.filter(|_| registered()) else {
        return;
    };
    let now = COST.get();
    FRAMES.with_borrow_mut(|frames| {
        frames.push(WorthQueryFamilySelectionCostForTest {
            rewalks: now.rewalks - start.rewalks,
            candidates_compared: now.candidates_compared - start.candidates_compared,
            rows_compared: now.rows_compared - start.rows_compared,
        })
    });
}

/// Re-resolution work counted on this thread.
#[doc(hidden)]
pub fn family_selection_cost_on_this_thread_for_test() -> WorthQueryFamilySelectionCostForTest {
    COST.get()
}

/// The cost of each certification frame closed on this thread, in closing
/// order; a frame's cost includes the frames nested in it.
#[doc(hidden)]
pub fn family_selection_frame_costs_on_this_thread_for_test() -> Vec<WorthQueryFamilySelectionCostForTest> {
    FRAMES.with_borrow(Clone::clone)
}

// Law 2, scope: passive counts end with their owned observation guard.
fn clear_observations() {
    CLEANUP_REGISTERED.set(false);
    COST.set(WorthQueryFamilySelectionCostForTest::default());
    FRAMES.with_borrow_mut(Vec::clear);
}
