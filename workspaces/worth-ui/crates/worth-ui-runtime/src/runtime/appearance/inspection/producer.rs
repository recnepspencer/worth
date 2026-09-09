use std::collections::{BTreeMap, BTreeSet};

use super::denial;
mod generation;
pub use generation::UiAppearanceInspectionGenerationSuccessionDenial;
pub(super) use generation::UiAppearanceInspectionScope;
pub(crate) use generation::UiPreparedAppearanceInspectionGenerationSuccession;
use worth_ui_dsl::{UiAppearanceAspect, UiAppearanceStateAxis};
use worth_ui_inspection::{
    UiAppearanceInspectionExplanation, UiAppearanceInspectionOutcome, UiAppearanceInspectionQuery,
    UiAppearanceInspectionSupport, UiAppearanceInspectionWorld, UiEvidenceAuthorityGeneration,
};

const UI_APPEARANCE_INSPECTION_CAPACITY: usize = 64;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiAppearanceInspectionDenial {
    MissingOperabilityRoute,
    AmbiguousOperabilityRoute { routes: usize },
    OperabilitySourceUnavailable,
    InteractionSourceUnavailable(UiAppearanceStateAxis),
    Basis,
    Resolution,
    MountAffinity,
    MountLowering,
}

impl UiAppearanceInspectionDenial {
    pub(crate) const fn blocks_mounted_output(self) -> bool {
        !matches!(
            self,
            Self::MissingOperabilityRoute
                | Self::AmbiguousOperabilityRoute { .. }
                | Self::OperabilitySourceUnavailable
                | Self::InteractionSourceUnavailable(_)
                | Self::Resolution
        )
    }
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
    current_scope: Option<UiAppearanceInspectionScope>,
    evidence_generation: UiEvidenceAuthorityGeneration,
    next_sequence: u64,
    #[cfg(test)]
    test_current_world: Option<UiAppearanceInspectionWorld>,
}

impl UiAppearanceInspectionProducer {
    pub(crate) fn record_projection(
        &mut self,
        projection: &super::super::projection::UiAppearanceProjection,
        consumers_selected: u32,
        receipt: super::super::projection::UiAppearanceChangeReceipt,
    ) {
        let basis = projection.state().basis();
        let scope = UiAppearanceInspectionScope::from_parts(basis.session(), basis.generation());
        super::projection_record::record_projection(
            self,
            &scope,
            projection,
            consumers_selected,
            receipt,
        );
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
                } => {
                    let scope = UiAppearanceInspectionScope::from_parts(
                        context.target().session(),
                        context.generation(),
                    );
                    denial::record_attempt_denial(self, &scope, &context, denial, receipt)
                }
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
                let scope = UiAppearanceInspectionScope::from_parts(
                    context.target().session(),
                    context.generation(),
                );
                denial::record_attempt_denial(self, &scope, &context, denial, receipt);
            }
        }
    }

    pub(super) fn record_scoped(
        &mut self,
        scope: &UiAppearanceInspectionScope,
        query: UiAppearanceInspectionQuery,
        explanation: UiAppearanceInspectionExplanation,
    ) {
        let Some(current_scope) = self.current_scope.as_ref() else {
            return;
        };
        let world = query.world();
        if current_scope != scope
            || world != scope.world(self.evidence_generation, world.surface_identity())
        {
            return;
        }
        self.record_entry(query, explanation);
    }

    fn record_entry(
        &mut self,
        query: UiAppearanceInspectionQuery,
        explanation: UiAppearanceInspectionExplanation,
    ) {
        let world = query.world();
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

    #[cfg(test)]
    pub(crate) fn record(
        &mut self,
        query: UiAppearanceInspectionQuery,
        explanation: UiAppearanceInspectionExplanation,
    ) {
        let world = query.world();
        if self.test_current_world.is_none() {
            self.test_current_world = Some(world);
        }
        assert_eq!(
            self.test_current_world,
            Some(world),
            "test inspection records must belong to one inert test world"
        );
        self.record_entry(query, explanation);
    }

    #[cfg(test)]
    pub(crate) fn replace_test_world(&mut self, world: UiAppearanceInspectionWorld) {
        self.test_current_world = Some(world);
        self.entries.clear();
        self.expired.clear();
    }

    pub(crate) fn query(
        &self,
        query: UiAppearanceInspectionQuery,
    ) -> UiAppearanceInspectionOutcome {
        let world = query.world();
        let Some(scope) = self.current_scope.as_ref() else {
            #[cfg(test)]
            {
                if self.test_current_world != Some(world) {
                    return UiAppearanceInspectionOutcome::WrongWorld;
                }
                return self.query_known_world(query);
            }
            #[cfg(not(test))]
            return UiAppearanceInspectionOutcome::WrongWorld;
        };
        if world.session_identity() != scope.session.as_u64()
            || world.evidence_generation() != self.evidence_generation
        {
            return UiAppearanceInspectionOutcome::WrongWorld;
        }
        self.query_known_world(query)
    }

    fn query_known_world(
        &self,
        query: UiAppearanceInspectionQuery,
    ) -> UiAppearanceInspectionOutcome {
        let world = query.world();
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
        } else if self.surface_is_recorded(world) {
            UiAppearanceInspectionOutcome::Unavailable
        } else {
            UiAppearanceInspectionOutcome::WrongWorld
        }
    }

    fn surface_is_recorded(&self, world: UiAppearanceInspectionWorld) -> bool {
        self.entries.keys().any(|(entry_world, _, _)| {
            entry_world.session_identity() == world.session_identity()
                && entry_world.evidence_generation() == world.evidence_generation()
                && entry_world.surface_identity() == world.surface_identity()
        }) || self.expired.iter().any(|(entry_world, _, _)| {
            entry_world.session_identity() == world.session_identity()
                && entry_world.evidence_generation() == world.evidence_generation()
                && entry_world.surface_identity() == world.surface_identity()
        })
    }
}
