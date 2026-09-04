use std::collections::{BTreeMap, BTreeSet};

use super::denial;
use worth_ui_dsl::UiAppearanceAspect;
use worth_ui_inspection::{
    UiAppearanceInspectionExplanation, UiAppearanceInspectionOutcome, UiAppearanceInspectionQuery,
    UiAppearanceInspectionSupport, UiAppearanceInspectionWorld,
};

const UI_APPEARANCE_INSPECTION_CAPACITY: usize = 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiAppearanceInspectionDenial {
    Basis,
    Resolution,
    MountAffinity,
    MountLowering,
}

pub(crate) enum UiAppearanceInspectionRecord {
    Projection {
        projection: super::super::projection::UiAppearanceProjection,
        consumers_selected: u32,
        receipt: super::super::projection::UiAppearanceChangeReceipt,
    },
    Denial {
        context: super::super::projection::UiAppearanceAttemptContext,
        denial: UiAppearanceInspectionDenial,
        receipt: super::super::projection::UiAppearanceChangeReceipt,
    },
}

pub(crate) struct UiAppearanceInspectionAttemptBatch {
    invalidation: Option<super::super::invalidation::UiAppearanceInvalidationBatch>,
    records: Vec<UiAppearanceInspectionRecord>,
}

impl UiAppearanceInspectionAttemptBatch {
    pub(crate) fn new(
        invalidation: Option<super::super::invalidation::UiAppearanceInvalidationBatch>,
        records: Vec<UiAppearanceInspectionRecord>,
    ) -> Self {
        Self {
            invalidation,
            records,
        }
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        Option<super::super::invalidation::UiAppearanceInvalidationBatch>,
        Vec<UiAppearanceInspectionRecord>,
    ) {
        (self.invalidation, self.records)
    }
}

#[derive(Clone)]
struct Entry {
    explanation: UiAppearanceInspectionExplanation,
    sequence: u64,
}

type InspectionKey = (UiAppearanceInspectionWorld, u64, UiAppearanceAspect);

pub(crate) struct UiAppearanceInspectionProducer {
    entries: BTreeMap<InspectionKey, Entry>,
    expired: BTreeSet<InspectionKey>,
    current_world: Option<UiAppearanceInspectionWorld>,
    retired_world: Option<UiAppearanceInspectionWorld>,
    next_sequence: u64,
}

impl UiAppearanceInspectionProducer {
    pub(crate) fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
            expired: BTreeSet::new(),
            current_world: None,
            retired_world: None,
            next_sequence: 1,
        }
    }

    pub(crate) fn record_projection(
        &mut self,
        projection: &super::super::projection::UiAppearanceProjection,
        consumers_selected: u32,
        receipt: super::super::projection::UiAppearanceChangeReceipt,
    ) {
        super::projection_record::record_projection(self, projection, consumers_selected, receipt);
    }

    pub(crate) fn record_frame_attempts(
        &mut self,
        records: impl IntoIterator<Item = UiAppearanceInspectionRecord>,
    ) {
        for record in records {
            match record {
                UiAppearanceInspectionRecord::Projection {
                    projection,
                    consumers_selected,
                    receipt,
                } => self.record_projection(&projection, consumers_selected, receipt),
                UiAppearanceInspectionRecord::Denial {
                    context,
                    denial,
                    receipt,
                } => denial::record_attempt_denial(self, &context, denial, receipt),
            }
        }
    }

    pub(crate) fn record_pre_effect_denials(
        &mut self,
        records: impl IntoIterator<Item = UiAppearanceInspectionRecord>,
    ) {
        for record in records {
            if let UiAppearanceInspectionRecord::Denial {
                context,
                denial,
                receipt,
            } = record
            {
                denial::record_attempt_denial(self, &context, denial, receipt);
            }
        }
    }

    pub(crate) fn reset_for_new_generation(&mut self) {
        if let Some(current_world) = self.current_world.take() {
            self.retired_world = Some(current_world);
        }
        let retired = self.entries.keys().copied().collect::<Vec<_>>();
        self.entries.clear();
        self.expired.extend(retired);
        while self.expired.len() > UI_APPEARANCE_INSPECTION_CAPACITY {
            let oldest = *self
                .expired
                .iter()
                .next()
                .expect("expired appearance inspection entries are non-empty");
            self.expired.remove(&oldest);
        }
    }

    pub(crate) fn record(
        &mut self,
        query: UiAppearanceInspectionQuery,
        explanation: UiAppearanceInspectionExplanation,
    ) {
        let world = query.world();
        if self.current_world.is_none() {
            self.current_world = Some(world);
        } else {
            debug_assert_eq!(
                self.current_world,
                Some(world),
                "appearance inspection records must belong to the active world"
            );
        }
        let key = (world, query.graph_node_digest(), query.aspect());
        self.expired.remove(&key);
        let sequence = self.next_sequence;
        self.next_sequence = self
            .next_sequence
            .checked_add(1)
            .expect("bounded appearance inspection sequence exhausted");
        if let std::collections::btree_map::Entry::Occupied(mut entry) = self.entries.entry(key) {
            entry.insert(Entry {
                explanation,
                sequence,
            });
            return;
        }
        if self.entries.len() == UI_APPEARANCE_INSPECTION_CAPACITY {
            let oldest = self
                .entries
                .iter()
                .min_by_key(|(_, entry)| entry.sequence)
                .map(|(key, _)| *key)
                .expect("a full inspection store has an oldest entry");
            self.entries.remove(&oldest);
            self.expired.insert(oldest);
            while self.expired.len() > UI_APPEARANCE_INSPECTION_CAPACITY {
                let oldest_expired = *self
                    .expired
                    .iter()
                    .next()
                    .expect("expired inspection set is non-empty");
                self.expired.remove(&oldest_expired);
            }
        }
        self.entries.insert(
            key,
            Entry {
                explanation,
                sequence,
            },
        );
    }

    pub(crate) fn query(
        &self,
        query: UiAppearanceInspectionQuery,
    ) -> UiAppearanceInspectionOutcome {
        let world = query.world();
        if self.current_world.is_some_and(|current| current != world)
            || self.retired_world == Some(world)
        {
            return UiAppearanceInspectionOutcome::WrongWorld;
        }
        let key = (world, query.graph_node_digest(), query.aspect());
        if let Some(entry) = self.entries.get(&key) {
            if matches!(
                entry.explanation.support(),
                UiAppearanceInspectionSupport::Unsupported
                    | UiAppearanceInspectionSupport::Inapplicable
            ) {
                return UiAppearanceInspectionOutcome::Unsupported;
            }
            return UiAppearanceInspectionOutcome::Found(entry.explanation.clone());
        }
        if self.expired.contains(&key) {
            UiAppearanceInspectionOutcome::Expired
        } else {
            UiAppearanceInspectionOutcome::Unavailable
        }
    }
}

impl Default for UiAppearanceInspectionProducer {
    fn default() -> Self {
        Self::new()
    }
}
