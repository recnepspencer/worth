use std::sync::{Arc, Mutex, MutexGuard};

use super::{effect_relation, PhysicalEffectFootprint, PhysicalEffectRelation};

#[derive(Clone, Debug)]
pub(in crate::physical_runtime) struct PhysicalEffectAdmission {
    state: Arc<Mutex<AdmissionState>>,
}

#[derive(Debug)]
struct AdmissionState {
    next_id: u64,
    limit: usize,
    active: Vec<ActiveEffect>,
}

#[derive(Debug)]
struct ActiveEffect {
    id: u64,
    footprint: PhysicalEffectFootprint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::physical_runtime) enum PhysicalEffectAdmissionDenial {
    Conflict,
    SlotsExhausted,
}

pub(in crate::physical_runtime) struct PhysicalEffectAdmissionLease {
    state: Arc<Mutex<AdmissionState>>,
    id: u64,
}

impl PhysicalEffectAdmission {
    pub(in crate::physical_runtime) fn new(limit: usize) -> Self {
        Self {
            state: Arc::new(Mutex::new(AdmissionState {
                next_id: 1,
                limit: limit.max(1),
                active: Vec::new(),
            })),
        }
    }

    pub(in crate::physical_runtime) fn admit(
        &self,
        footprint: PhysicalEffectFootprint,
    ) -> Result<PhysicalEffectAdmissionLease, PhysicalEffectAdmissionDenial> {
        let mut state = lock(&self.state);
        if state.active.len() == state.limit {
            return Err(PhysicalEffectAdmissionDenial::SlotsExhausted);
        }
        if state.active.iter().any(|active| {
            matches!(
                effect_relation(&active.footprint, &footprint),
                PhysicalEffectRelation::Conflict
            )
        }) {
            return Err(PhysicalEffectAdmissionDenial::Conflict);
        }
        let id = state.next_id;
        state.next_id = state.next_id.saturating_add(1);
        state.active.push(ActiveEffect { id, footprint });
        drop(state);
        Ok(PhysicalEffectAdmissionLease {
            state: Arc::clone(&self.state),
            id,
        })
    }
}

impl Drop for PhysicalEffectAdmissionLease {
    fn drop(&mut self) {
        let mut state = lock(&self.state);
        state.active.retain(|active| active.id != self.id);
    }
}

fn lock(state: &Arc<Mutex<AdmissionState>>) -> MutexGuard<'_, AdmissionState> {
    state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
