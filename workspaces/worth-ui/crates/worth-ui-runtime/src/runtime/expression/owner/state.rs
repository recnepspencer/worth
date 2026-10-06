use std::collections::BTreeMap;
use std::sync::Arc;

use worth_ui_query_binding::{UiProjectionInputFactReference, UiProjectionInputSlot};

use super::evaluation::{
    begin, complete, evaluate, UiExpressionCompletion, UiExpressionCompletionReceipt,
    UiExpressionEvaluationTicket,
};
use super::operand_binding::UiExpressionInputs;
use super::records::UiExpressionRecords;
use super::{UiExpressionEvaluationRecord, UiExpressionResultReference, UiExpressionWorkCounters};
use crate::runtime::expression::{UiExpressionCatalog, UiExpressionSlot};
use crate::runtime::WorthUiActiveApplicationGenerationIdentity;

/// The one owner of expression evaluation for one active application
/// generation. It retains one record per installed expression and settles
/// them through the dependency index only.
pub(crate) struct UiExpressionRuntimeState {
    pub(super) catalog: Arc<UiExpressionCatalog>,
    pub(super) generation: WorthUiActiveApplicationGenerationIdentity,
    pub(super) records: UiExpressionRecords,
    /// The projection input each indexed slot held when its readers last
    /// settled. A slot changed only when the mounted state now holds a
    /// different reference.
    pub(super) projections: BTreeMap<UiProjectionInputSlot, Option<UiProjectionInputFactReference>>,
    pub(super) counters: UiExpressionWorkCounters,
}

impl UiExpressionRuntimeState {
    /// Settles every expression once, in topological order, against the
    /// current owners of the active generation.
    pub(crate) fn activate(
        catalog: Arc<UiExpressionCatalog>,
        inputs: &UiExpressionInputs<'_>,
    ) -> Self {
        let mut state = Self::unsettled(catalog, inputs.generation.clone(), inputs.mounted);
        for slot in state.catalog.slot_count().slots() {
            state.settle(slot, inputs);
        }
        state
    }

    /// The state activation starts from: the indexed projection slots probed
    /// once and no expression yet holding a record.
    pub(crate) fn unsettled(
        catalog: Arc<UiExpressionCatalog>,
        generation: WorthUiActiveApplicationGenerationIdentity,
        mounted: &crate::mounting::WorthUiMountedSessionState,
    ) -> Self {
        let mut counters = UiExpressionWorkCounters::default();
        let projections = probe_projections(&catalog, mounted, &mut counters);
        Self {
            records: UiExpressionRecords::unsettled(catalog.slot_count()),
            catalog,
            generation,
            projections,
            counters,
        }
    }

    pub(crate) fn catalog(&self) -> &UiExpressionCatalog {
        &self.catalog
    }

    pub(crate) const fn counters(&self) -> UiExpressionWorkCounters {
        self.counters
    }

    pub(crate) fn record(&self, slot: UiExpressionSlot) -> Option<&UiExpressionEvaluationRecord> {
        self.records.get(slot)
    }

    pub(crate) fn record_by_identity(
        &self,
        identity: &str,
    ) -> Option<&UiExpressionEvaluationRecord> {
        self.record(self.catalog.slot_of(identity)?)
    }

    /// The reference to the retained result of `slot`, whatever its posture.
    pub(crate) fn result(&self, slot: UiExpressionSlot) -> Option<UiExpressionResultReference> {
        self.record(slot)
            .map(UiExpressionEvaluationRecord::result_reference)
    }

    /// Whether this owner holds the records of `active`, the generation the
    /// session runs now. An owner that has not followed a generation change
    /// holds nothing current: it begins no evaluation, admits no completion,
    /// settles no invalidation and reports no result as current.
    pub(super) fn follows(&self, active: &WorthUiActiveApplicationGenerationIdentity) -> bool {
        self.generation == *active
    }

    /// Whether `reference` names the retained result of `active`, the
    /// generation the session runs now.
    pub(crate) fn is_current_result(
        &self,
        reference: &UiExpressionResultReference,
        active: &WorthUiActiveApplicationGenerationIdentity,
    ) -> bool {
        self.follows(active)
            && reference.generation() == active
            && self.record(reference.slot()).is_some_and(|record| {
                record.outcome_revision() == reference.outcome_revision()
                    && record.identity() == reference.identity()
            })
    }

    /// Reads the operands of `slot` and returns the ticket its evaluation may
    /// complete with, or `None` when the catalog has no such expression or
    /// this owner does not follow the active generation.
    pub(crate) fn begin_evaluation(
        &mut self,
        slot: UiExpressionSlot,
        inputs: &UiExpressionInputs<'_>,
    ) -> Option<UiExpressionEvaluationTicket> {
        if !self.follows(inputs.generation) {
            return None;
        }
        begin(
            &self.catalog,
            &self.records,
            slot,
            inputs,
            &mut self.counters,
        )
    }

    pub(crate) fn evaluate_ticket(
        &mut self,
        ticket: UiExpressionEvaluationTicket,
    ) -> UiExpressionCompletion {
        evaluate(ticket, &mut self.counters)
    }

    /// Admits `completion` only while its generation is the active one, this
    /// owner follows it, and every operand fact it read is still current.
    pub(crate) fn complete_evaluation(
        &mut self,
        completion: UiExpressionCompletion,
        inputs: &UiExpressionInputs<'_>,
    ) -> UiExpressionCompletionReceipt {
        let follows = self.follows(inputs.generation);
        complete(
            &mut self.records,
            completion,
            inputs,
            follows,
            &mut self.counters,
        )
    }

    /// The single production path: begin, evaluate, complete.
    pub(super) fn settle(
        &mut self,
        slot: UiExpressionSlot,
        inputs: &UiExpressionInputs<'_>,
    ) -> Option<UiExpressionCompletionReceipt> {
        let ticket = self.begin_evaluation(slot, inputs)?;
        let completion = self.evaluate_ticket(ticket);
        Some(self.complete_evaluation(completion, inputs))
    }
}

/// The projection input each indexed slot of `catalog` holds now, one operand
/// probe per slot.
pub(super) fn probe_projections(
    catalog: &UiExpressionCatalog,
    mounted: &crate::mounting::WorthUiMountedSessionState,
    counters: &mut UiExpressionWorkCounters,
) -> BTreeMap<UiProjectionInputSlot, Option<UiProjectionInputFactReference>> {
    catalog
        .dependencies()
        .projection_slots()
        .map(|slot| {
            counters.operand_probes = counters.operand_probes.saturating_add(1);
            (slot, mounted.current_projection_input(slot))
        })
        .collect()
}
