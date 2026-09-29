use std::collections::BTreeSet;
use std::sync::Arc;

use super::operand_binding::UiExpressionInputs;
use super::records::UiExpressionRecords;
use super::{
    UiExpressionEvaluationRecord, UiExpressionOperandFact, UiExpressionRuntimeState,
    UiExpressionWorkCounters,
};
use crate::runtime::expression::{
    UiExpressionCatalog, UiInstalledExpression, UiResolvedExpressionOperand,
};

impl UiExpressionRuntimeState {
    /// The owner of the generation `inputs` names as active, against the
    /// catalog that generation installs, computed from this owner without
    /// changing it. The successor's counters hold only the work of this
    /// computation.
    ///
    /// An expression whose successor installs the same program over the same
    /// operands in the same slot, and whose every operand fact is still the
    /// one its owner holds, is re-stamped: its record keeps its outcome,
    /// outcome revision and last current value, takes the new generation and
    /// the successor's span, and is not evaluated again. Any other expression
    /// is rebuilt with no prior record, and every expression is rebuilt when
    /// the successor catalog installs different slots. A rebuilt expression
    /// settles its readers through the dependency index, as any change does.
    /// No reference or ticket of this owner's generation is current in the
    /// successor.
    ///
    /// Each operand re-proven counts one operand probe, each indexed
    /// projection slot one probe, and a rebuild counts as any settle does.
    /// What the consumers re-observe is measured when the successor commits.
    pub(super) fn successor(
        &self,
        catalog: &Arc<UiExpressionCatalog>,
        inputs: &UiExpressionInputs<'_>,
    ) -> Self {
        let same_slots = self.catalog.has_same_slots_as(catalog);
        let mut successor = Self::unsettled(
            Arc::clone(catalog),
            inputs.generation.clone(),
            inputs.mounted,
        );
        let mut dirty = BTreeSet::new();
        for slot in catalog.slot_count().slots() {
            let next = catalog.expression(slot);
            let prior_slot = next.and_then(|next| self.catalog.slot_of(next.identity()));
            let prior_record = prior_slot.and_then(|prior| self.records.get(prior));
            let prior_expression = prior_slot.and_then(|prior| self.catalog.expression(prior));
            if let (Some(record), Some(prior), Some(next)) = (prior_record, prior_expression, next)
            {
                if same_slots
                    && prior.installs_same_program_as(next)
                    && restamp(
                        record,
                        next,
                        &successor.records,
                        inputs,
                        &mut successor.counters,
                    )
                    .is_some_and(|restamped| successor.records.admit(restamped).is_ok())
                {
                    continue;
                }
            }
            dirty.insert(slot);
        }
        // The successor has no consumers yet; the commit reports against the
        // owner it replaces.
        let _settled = successor.settle_dirty(dirty, inputs);
        successor
    }
}

/// `record` as a record of the successor generation, when every operand fact
/// it read is still what its owner holds, or `None` when one is not.
/// `records` holds the successor records of every upstream slot already
/// followed.
fn restamp(
    record: &UiExpressionEvaluationRecord,
    successor: &UiInstalledExpression,
    records: &UiExpressionRecords,
    inputs: &UiExpressionInputs<'_>,
    counters: &mut UiExpressionWorkCounters,
) -> Option<UiExpressionEvaluationRecord> {
    if successor.operands().len() != record.operands.len() {
        return None;
    }
    let reproven = successor
        .operands()
        .iter()
        .zip(&record.operands)
        .map(|(operand, read)| {
            counters.operand_probes = counters.operand_probes.saturating_add(1);
            reprove(operand.source(), read, records, inputs)
        })
        .collect::<Option<_>>()?;
    Some(UiExpressionEvaluationRecord {
        operands: reproven,
        generation: inputs.generation.clone(),
        span: successor.body_span().cloned(),
        ..record.clone()
    })
}

/// The fact `read` is in the successor generation, or `None` when its owner
/// no longer holds it. An application fact is issued per generation, so an
/// equal identity, revision and value is re-issued for the successor. An
/// upstream expression that is rebuilt instead settles this reader through
/// the dependency index when its outcome changes.
fn reprove(
    source: &UiResolvedExpressionOperand,
    read: &UiExpressionOperandFact,
    records: &UiExpressionRecords,
    inputs: &UiExpressionInputs<'_>,
) -> Option<UiExpressionOperandFact> {
    let current = match source {
        UiResolvedExpressionOperand::QueryScalar { slot, .. } => {
            let observed = inputs.mounted.current_projection_input(*slot);
            match read {
                UiExpressionOperandFact::Query(reference) => observed.as_ref() == Some(reference),
                UiExpressionOperandFact::Absent { .. } => observed.is_none(),
                UiExpressionOperandFact::Application(_)
                | UiExpressionOperandFact::Expression { .. } => false,
            }
        }
        UiResolvedExpressionOperand::Application { slot, .. } => {
            let held = inputs.facts.input_reference(*slot, inputs.generation);
            match read {
                UiExpressionOperandFact::Application(reference) => {
                    let restamped = reference.restamped(inputs.generation);
                    return (held.as_ref() == Some(&restamped))
                        .then(|| UiExpressionOperandFact::Application(restamped));
                }
                UiExpressionOperandFact::Absent { .. } => held.is_none(),
                UiExpressionOperandFact::Query(_) | UiExpressionOperandFact::Expression { .. } => {
                    false
                }
            }
        }
        UiResolvedExpressionOperand::Expression { slot, .. } => match read {
            UiExpressionOperandFact::Expression {
                slot: read_slot,
                outcome_revision,
            } => {
                read_slot == slot
                    && records
                        .get(*slot)
                        .is_none_or(|upstream| upstream.outcome_revision == *outcome_revision)
            }
            UiExpressionOperandFact::Absent { .. } => records.get(*slot).is_none(),
            UiExpressionOperandFact::Query(_) | UiExpressionOperandFact::Application(_) => false,
        },
    };
    current.then(|| read.clone())
}
