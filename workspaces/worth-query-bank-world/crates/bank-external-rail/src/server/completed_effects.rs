//! Physical completion consequences owned independently from idempotency state.

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::Mutex;

use crate::protocol::correlation::RailCorrelation;
use crate::protocol::notice::{EstateDeathNotice, RailDomainEffect};

/// The rail-side consequence of completing one admitted death notice.
///
/// This is deliberately separate from the dispatch ledger. Ledger status says
/// what the protocol owner knows; this store proves how many domain effects
/// were actually applied.
#[derive(Default)]
pub struct CompletedEffects {
    effects: Mutex<HashMap<RailCorrelation, RailDomainEffect>>,
}

impl CompletedEffects {
    pub fn apply_once(
        &self,
        correlation: RailCorrelation,
        effect: RailDomainEffect,
    ) -> Result<(), CompletedEffectConflict> {
        match self.lock().entry(correlation) {
            Entry::Vacant(entry) => {
                entry.insert(effect);
                Ok(())
            }
            Entry::Occupied(entry) if entry.get() == &effect => {
                Err(CompletedEffectConflict::Repeat)
            }
            Entry::Occupied(_) => Err(CompletedEffectConflict::MeaningDrift),
        }
    }

    pub fn count(&self) -> u64 {
        u64::try_from(self.lock().len()).expect("rail consequence count fits in u64")
    }

    pub fn notice_of(&self, correlation: &RailCorrelation) -> Option<EstateDeathNotice> {
        self.lock()
            .get(correlation)
            .and_then(|effect| match effect {
                RailDomainEffect::EstateDeathNotice(notice) => Some(*notice),
                RailDomainEffect::ApprovedPaymentSettlement(_) => None,
            })
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<RailCorrelation, RailDomainEffect>> {
        self.effects
            .lock()
            .expect("rail completed-effect mutex is never poisoned")
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompletedEffectConflict {
    Repeat,
    MeaningDrift,
}
