//! Class selection for the one physical scheduler.
//!
//! Foreground and background share one permit stream. A ready background head
//! earns a dispatch after at most three foreground dispatches. Once that turn
//! is owed, further foreground refills wait until the background head is
//! dispatched or cancelled. This owner does not infer effect disjointness.

use std::sync::{Arc, Mutex, MutexGuard};

const FOREGROUND_WEIGHT: u8 = 3;

#[derive(Clone, Debug)]
pub struct PhysicalDispatchSelection {
    state: Arc<Mutex<SelectionState>>,
}

#[derive(Debug)]
struct SelectionState {
    foreground_since_background: u8,
    ready_background: u32,
    background_owed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OwedBackgroundTurn;

pub struct ForegroundDispatchTurn {
    selection: PhysicalDispatchSelection,
    armed: bool,
}

pub struct BackgroundDispatchAttempt {
    selection: PhysicalDispatchSelection,
    open: bool,
}

impl PhysicalDispatchSelection {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(SelectionState {
                foreground_since_background: 0,
                ready_background: 0,
                background_owed: false,
            })),
        }
    }

    pub fn begin_foreground(&self) -> Result<ForegroundDispatchTurn, OwedBackgroundTurn> {
        let mut state = lock(&self.state);
        if state.background_owed {
            return Err(OwedBackgroundTurn);
        }
        state.foreground_since_background = state.foreground_since_background.saturating_add(1);
        recompute_owed(&mut state);
        drop(state);
        Ok(ForegroundDispatchTurn {
            selection: self.clone(),
            armed: true,
        })
    }

    /// Queue one background head for the lifetime of the returned attempt.
    pub fn begin_background(&self) -> BackgroundDispatchAttempt {
        self.note_ready_background();
        BackgroundDispatchAttempt {
            selection: self.clone(),
            open: true,
        }
    }

    /// Record a background job that stays ready until its owner releases it.
    /// Capacity shortage must not drop this count.
    pub fn note_ready_background(&self) {
        let mut state = lock(&self.state);
        state.ready_background = state.ready_background.saturating_add(1);
        recompute_owed(&mut state);
    }

    /// The owner cancelled the job or lost a non-capacity prerequisite.
    pub fn release_ready_background(&self) {
        let mut state = lock(&self.state);
        state.ready_background = state.ready_background.saturating_sub(1);
        recompute_owed(&mut state);
    }

    /// One background quantum has been reserved. The job may remain ready.
    pub fn commit_background_quantum(&self) {
        let mut state = lock(&self.state);
        state.foreground_since_background = 0;
        recompute_owed(&mut state);
    }

    pub fn background_owed(&self) -> bool {
        lock(&self.state).background_owed
    }

    pub fn foreground_since_background(&self) -> u8 {
        lock(&self.state).foreground_since_background
    }

    fn abort_foreground(&self) {
        let mut state = lock(&self.state);
        state.foreground_since_background = state.foreground_since_background.saturating_sub(1);
        recompute_owed(&mut state);
    }
}

fn recompute_owed(state: &mut SelectionState) {
    state.background_owed =
        state.ready_background > 0 && state.foreground_since_background >= FOREGROUND_WEIGHT;
}

impl Default for PhysicalDispatchSelection {
    fn default() -> Self {
        Self::new()
    }
}

impl ForegroundDispatchTurn {
    pub fn commit(mut self) {
        self.armed = false;
    }
}

impl Drop for ForegroundDispatchTurn {
    fn drop(&mut self) {
        if self.armed {
            self.selection.abort_foreground();
        }
    }
}

impl BackgroundDispatchAttempt {
    pub fn commit(mut self) {
        self.selection.commit_background_quantum();
        self.selection.release_ready_background();
        self.open = false;
    }
}

impl Drop for BackgroundDispatchAttempt {
    fn drop(&mut self) {
        if self.open {
            self.selection.release_ready_background();
        }
    }
}

fn lock(state: &Arc<Mutex<SelectionState>>) -> MutexGuard<'_, SelectionState> {
    state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_foreground_dispatches_owe_one_background_turn() {
        let selection = PhysicalDispatchSelection::new();
        let background = selection.begin_background();
        for _ in 0..3 {
            selection.begin_foreground().unwrap().commit();
        }
        assert!(selection.background_owed());
        assert!(selection.begin_foreground().is_err());
        background.commit();
        assert!(!selection.background_owed());
        assert_eq!(selection.foreground_since_background(), 0);
        selection.begin_foreground().unwrap().commit();
    }

    #[test]
    fn failed_foreground_reserve_does_not_consume_the_weight() {
        let selection = PhysicalDispatchSelection::new();
        let _background = selection.begin_background();
        selection.begin_foreground().unwrap().commit();
        selection.begin_foreground().unwrap().commit();
        drop(selection.begin_foreground().unwrap());
        assert_eq!(selection.foreground_since_background(), 2);
        assert!(!selection.background_owed());
        selection.begin_foreground().unwrap().commit();
        assert!(selection.background_owed());
    }

    #[test]
    fn foreground_without_a_background_head_is_not_held() {
        let selection = PhysicalDispatchSelection::new();
        for _ in 0..5 {
            selection.begin_foreground().unwrap().commit();
        }
        assert!(!selection.background_owed());
    }

    #[test]
    fn cancelled_background_releases_the_hold() {
        let selection = PhysicalDispatchSelection::new();
        let background = selection.begin_background();
        selection.begin_foreground().unwrap().commit();
        selection.begin_foreground().unwrap().commit();
        selection.begin_foreground().unwrap().commit();
        drop(background);
        assert!(!selection.background_owed());
        selection.begin_foreground().unwrap().commit();
    }

    #[test]
    fn a_noted_background_head_survives_until_its_owner_releases_it() {
        let selection = PhysicalDispatchSelection::new();
        selection.note_ready_background();
        selection.begin_foreground().unwrap().commit();
        selection.begin_foreground().unwrap().commit();
        selection.begin_foreground().unwrap().commit();
        assert!(selection.begin_foreground().is_err());
        selection.commit_background_quantum();
        selection.begin_foreground().unwrap().commit();
        selection.release_ready_background();
        for _ in 0..4 {
            selection.begin_foreground().unwrap().commit();
        }
    }

    #[test]
    fn both_classes_ready_selects_background_on_the_fourth_turn() {
        let selection = PhysicalDispatchSelection::new();
        let background = selection.begin_background();
        let mut order = Vec::new();
        for _ in 0..3 {
            selection.begin_foreground().unwrap().commit();
            order.push("foreground");
        }
        assert!(selection.begin_foreground().is_err());
        background.commit();
        order.push("background");
        assert_eq!(
            order,
            ["foreground", "foreground", "foreground", "background"]
        );
    }
}
