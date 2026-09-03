use std::collections::{BTreeMap, BTreeSet};

use worth_ui_dsl::UiAppearanceAspect;
use worth_ui_inspection::{
    UiAppearanceInspectionCost, UiAppearanceInspectionDecisionCell, UiAppearanceInspectionEvidence,
    UiAppearanceInspectionExplanation, UiAppearanceInspectionInvalidationCause,
    UiAppearanceInspectionMountedMechanic, UiAppearanceInspectionOutcome,
    UiAppearanceInspectionPhysicalSuppression, UiAppearanceInspectionQuery,
    UiAppearanceInspectionSourceSpan, UiAppearanceInspectionSupport, UiAppearanceInspectionValue,
    UiAppearanceInspectionWorld,
};

const UI_APPEARANCE_INSPECTION_CAPACITY: usize = 64;

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
        self.record_projection_with_cause(
            projection,
            consumers_selected,
            receipt,
            invalidation_cause(receipt),
        );
    }

    pub(crate) fn record_projection_with_cause(
        &mut self,
        projection: &super::super::projection::UiAppearanceProjection,
        consumers_selected: u32,
        receipt: super::super::projection::UiAppearanceChangeReceipt,
        invalidation_cause: UiAppearanceInspectionInvalidationCause,
    ) {
        let world = world_for_projection(projection);
        for aspect in projection.aspects() {
            let query = UiAppearanceInspectionQuery::new(
                world,
                projection.target().graph_node().digest(),
                aspect.aspect(),
            );
            let explanation = UiAppearanceInspectionExplanation::new(
                query,
                projection.role().as_str(),
                projection.role_revision().value(),
                projection.theme(),
                projection.theme_revision(),
                aspect.state_classes().to_vec().into_boxed_slice(),
                UiAppearanceInspectionDecisionCell::new(
                    aspect.decision_cell_ordinal(),
                    aspect.state_classes().to_vec().into_boxed_slice(),
                ),
                UiAppearanceInspectionSourceSpan::Unavailable,
                aspect.provenance().selected_slot().as_str(),
                aspect.provenance().terminal_slot().as_str(),
                support(aspect.support()),
                if receipt.denied_before_effects() {
                    UiAppearanceInspectionValue::Missing
                } else {
                    UiAppearanceInspectionValue::Resolved(aspect.value())
                },
                invalidation_cause,
                mounted_mechanic(receipt),
                physical_suppression(receipt),
                aspect.semantic_digest(),
                UiAppearanceInspectionEvidence::new(
                    projection.state().basis().source_basis(),
                    projection.state().basis().turn().as_u64(),
                    *projection.state().basis().owner_revisions(),
                ),
                UiAppearanceInspectionCost::new(
                    projection.state().classes().count() as u8,
                    1,
                    aspect.decision_cells_visited(),
                    aspect.theme_slots_compared(),
                    consumers_selected,
                ),
            );
            self.record(query, explanation);
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

fn invalidation_cause(
    receipt: super::super::projection::UiAppearanceChangeReceipt,
) -> UiAppearanceInspectionInvalidationCause {
    match receipt.outcome() {
        Some(super::super::projection::UiAppearanceChangeOutcome::InputEvidenceChanged) => {
            UiAppearanceInspectionInvalidationCause::InputEvidenceChanged
        }
        Some(super::super::projection::UiAppearanceChangeOutcome::SemanticProjectionChanged) => {
            UiAppearanceInspectionInvalidationCause::SemanticProjectionChanged
        }
        Some(super::super::projection::UiAppearanceChangeOutcome::ResolvedAspectValueChanged) => {
            UiAppearanceInspectionInvalidationCause::ResolvedAspectValueChanged
        }
        Some(
            super::super::projection::UiAppearanceChangeOutcome::MountedMechanicalOutputChanged,
        ) => UiAppearanceInspectionInvalidationCause::MountedMechanicalOutputChanged,
        Some(super::super::projection::UiAppearanceChangeOutcome::EqualOutputSuppressed) => {
            UiAppearanceInspectionInvalidationCause::EqualOutputSuppressed
        }
        Some(super::super::projection::UiAppearanceChangeOutcome::DeniedBeforeEffects) => {
            UiAppearanceInspectionInvalidationCause::DeniedBeforeEffects
        }
        None => UiAppearanceInspectionInvalidationCause::NotAttributed,
    }
}

fn mounted_mechanic(
    receipt: super::super::projection::UiAppearanceChangeReceipt,
) -> UiAppearanceInspectionMountedMechanic {
    if !receipt.mounting_result_available() {
        UiAppearanceInspectionMountedMechanic::NotAttempted
    } else if receipt.mounted_mechanical_output_changed() {
        UiAppearanceInspectionMountedMechanic::Changed
    } else {
        UiAppearanceInspectionMountedMechanic::Unchanged
    }
}

fn physical_suppression(
    receipt: super::super::projection::UiAppearanceChangeReceipt,
) -> UiAppearanceInspectionPhysicalSuppression {
    if !receipt.mounting_result_available() {
        UiAppearanceInspectionPhysicalSuppression::NotAttempted
    } else if receipt.equal_output_suppressed() {
        UiAppearanceInspectionPhysicalSuppression::Suppressed
    } else {
        UiAppearanceInspectionPhysicalSuppression::NotSuppressed
    }
}

impl Default for UiAppearanceInspectionProducer {
    fn default() -> Self {
        Self::new()
    }
}

fn support(
    support: super::super::projection::UiAppearanceSupportPosture,
) -> UiAppearanceInspectionSupport {
    match support {
        super::super::projection::UiAppearanceSupportPosture::Supported => {
            UiAppearanceInspectionSupport::Supported
        }
        super::super::projection::UiAppearanceSupportPosture::Unsupported => {
            UiAppearanceInspectionSupport::Unsupported
        }
        super::super::projection::UiAppearanceSupportPosture::Inapplicable => {
            UiAppearanceInspectionSupport::Inapplicable
        }
    }
}

fn world_for_projection(
    projection: &super::super::projection::UiAppearanceProjection,
) -> UiAppearanceInspectionWorld {
    let basis = projection.state().basis();
    UiAppearanceInspectionWorld::new(
        basis.session().as_u64(),
        basis
            .generation()
            .prepared_generation()
            .semantic_package_identity()
            .narrowing_fingerprint(),
        basis.surface().diagnostic_value(),
    )
}
