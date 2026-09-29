use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use worth_ui_query_binding::UiProjectionInputSlot;

use super::slot::UiExpressionSlotCount;
use super::{UiExpressionSlot, UiInstalledExpression, UiResolvedExpressionOperand};
use crate::declaration::UiIntentApplicationFactSlot;

/// Which expressions read each operand owner, so a change reaches only its
/// readers. Application slots and expression slots are dense; projection
/// slots are sparse because only some inputs feed expressions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiExpressionDependencyIndex {
    by_projection: BTreeMap<UiProjectionInputSlot, Box<[UiExpressionSlot]>>,
    by_application: Box<[Box<[UiExpressionSlot]>]>,
    dependents: Box<[Box<[UiExpressionSlot]>]>,
}

impl UiExpressionDependencyIndex {
    pub(super) fn build(
        expressions: &[Arc<UiInstalledExpression>],
        slot_count: UiExpressionSlotCount,
        application_fact_count: usize,
    ) -> Self {
        let mut by_projection =
            BTreeMap::<UiProjectionInputSlot, BTreeSet<UiExpressionSlot>>::new();
        let mut by_application = vec![BTreeSet::new(); application_fact_count];
        let mut dependents = vec![BTreeSet::new(); expressions.len()];
        for (expression, reader) in expressions.iter().zip(slot_count.slots()) {
            for operand in expression.operands() {
                match operand.source() {
                    UiResolvedExpressionOperand::QueryScalar { slot, .. } => {
                        by_projection.entry(*slot).or_default().insert(reader);
                    }
                    UiResolvedExpressionOperand::Application { slot, .. } => {
                        by_application[slot.index()].insert(reader);
                    }
                    UiResolvedExpressionOperand::Expression { slot: upstream, .. } => {
                        dependents[upstream.index()].insert(reader);
                    }
                }
            }
        }
        Self {
            by_projection: by_projection
                .into_iter()
                .map(|(slot, readers)| (slot, readers.into_iter().collect()))
                .collect(),
            by_application: by_application
                .into_iter()
                .map(|readers| readers.into_iter().collect())
                .collect(),
            dependents: dependents
                .into_iter()
                .map(|readers| readers.into_iter().collect())
                .collect(),
        }
    }

    pub(crate) fn projection_slots(&self) -> impl Iterator<Item = UiProjectionInputSlot> + '_ {
        self.by_projection.keys().copied()
    }

    pub(crate) fn readers_of_projection(&self, slot: UiProjectionInputSlot) -> &[UiExpressionSlot] {
        self.by_projection.get(&slot).map_or(&[], |readers| readers)
    }

    pub(crate) fn readers_of_application(
        &self,
        slot: UiIntentApplicationFactSlot,
    ) -> &[UiExpressionSlot] {
        self.by_application
            .get(slot.index())
            .map_or(&[], |readers| readers)
    }

    pub(crate) fn dependents_of(&self, slot: UiExpressionSlot) -> &[UiExpressionSlot] {
        self.dependents
            .get(slot.index())
            .map_or(&[], |readers| readers)
    }
}
