use std::sync::{atomic::Ordering, Arc};

use super::{PhysicalEffectActivity, PhysicalSubmissionState, PhysicalWorkSubmissionStale};

impl PhysicalSubmissionState {
    pub(super) fn enter(
        &self,
        generation: crate::physical_runtime::LifecycleGeneration,
    ) -> Result<SubmissionActivity<'_>, PhysicalWorkSubmissionStale> {
        let _guard = self
            .active_wait
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        self.require_admission(generation)?;
        self.active_submissions.fetch_add(1, Ordering::AcqRel);
        Ok(SubmissionActivity { state: self })
    }

    fn require_admission(
        &self,
        generation: crate::physical_runtime::LifecycleGeneration,
    ) -> Result<(), PhysicalWorkSubmissionStale> {
        let lifecycle = self.lifecycle.snapshot();
        if lifecycle.generation != generation || lifecycle.phase != self.lifecycle_phase {
            return Err(PhysicalWorkSubmissionStale::LifecycleGenerationAdvanced);
        }
        if !self.accepting.load(Ordering::Acquire) {
            return Err(PhysicalWorkSubmissionStale::AdmissionStopped);
        }
        if !self.signal_admission.is_available() {
            return Err(PhysicalWorkSubmissionStale::SignalOwnerUnavailable);
        }
        Ok(())
    }

    fn leave(&self) {
        let _guard = self
            .active_wait
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let previous = {
            let update_counter = &self.active_submissions;
            let mut value = update_counter.load(Ordering::Acquire);
            loop {
                let next_value = value.checked_sub(1);
                let Some(updated_value) = next_value else {
                    break Err(value);
                };
                match update_counter.compare_exchange_weak(
                    value,
                    updated_value,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                ) {
                    Ok(previous) => break Ok(previous),
                    Err(reloaded_value) => value = reloaded_value,
                }
            }
        }
        .expect("submission activity is released exactly once");
        if previous == 1 {
            self.active_changed.notify_all();
        }
    }

    pub(super) fn stop_accepting(&self) {
        let _guard = self
            .active_wait
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        self.accepting.store(false, Ordering::Release);
    }

    pub(super) fn await_idle(&self) {
        let mut guard = self
            .active_wait
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        while self.active_submissions.load(Ordering::Acquire) != 0
            || self.active_effects.load(Ordering::Acquire) != 0
        {
            guard = self
                .active_changed
                .wait(guard)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
    }

    pub(super) fn await_submissions(&self) {
        let mut guard = self
            .active_wait
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        while self.active_submissions.load(Ordering::Acquire) != 0 {
            guard = self
                .active_changed
                .wait(guard)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
    }

    pub(super) fn begin_effect(
        self: &Arc<Self>,
        identity: super::PhysicalWorkIdentity,
    ) -> Option<PhysicalEffectActivity> {
        let _guard = self
            .active_wait
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if !self.accepting.load(Ordering::Acquire) || !self.commands.begin_dispatch(identity) {
            return None;
        }
        self.active_effects.fetch_add(1, Ordering::AcqRel);
        Some(PhysicalEffectActivity::new(self))
    }

    pub(super) fn finish_effect(&self) {
        let _guard = self
            .active_wait
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let previous = {
            let update_counter = &self.active_effects;
            let mut value = update_counter.load(Ordering::Acquire);
            loop {
                let next_value = value.checked_sub(1);
                let Some(updated_value) = next_value else {
                    break Err(value);
                };
                match update_counter.compare_exchange_weak(
                    value,
                    updated_value,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                ) {
                    Ok(previous) => break Ok(previous),
                    Err(reloaded_value) => value = reloaded_value,
                }
            }
        }
        .expect("physical effect enrollment is released exactly once");
        if previous == 1 {
            self.active_changed.notify_all();
        }
    }
}

pub(super) struct SubmissionActivity<'state> {
    state: &'state PhysicalSubmissionState,
}

impl Drop for SubmissionActivity<'_> {
    fn drop(&mut self) {
        self.state.leave();
    }
}
