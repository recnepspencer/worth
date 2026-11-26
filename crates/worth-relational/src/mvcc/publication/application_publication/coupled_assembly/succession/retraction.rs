//! §2.3 step 5: retract a chain's live round to the image before its head.
use worth_relational_publication::internal_port::visibility::materialization::read_records::reader::pinned_read::RelationalPinnedReadOwnerPort as _;
use super::super::*;
use super::indexes::MemberId;
use super::segment::MemberSpan;
use crate::transactions::data::primitives::RecordRef;
use worth_relational_mutation::internal_port::authority::mutation::chain_retraction::{
    retract_chain_segment, OtherTouches, RetractionBase,
};

impl RelationalAuthoredCoupledCandidate<'_> {
    /// Every live member's span other than part `index`'s chain.
    pub(super) fn live_spans_except(&self, index: Option<usize>) -> Vec<MemberSpan> {
        super::segment::live_spans(&self.prefix_span, &self.stages, &self.parts, index)
    }

    /// Retract part `index`'s live round: its records, adjacency deltas,
    /// accumulator entries, bindings, fresh custody, index entries and
    /// content leave the candidate, which then reads as it did before the
    /// chain's head for everything the chain changed. Returns the round's
    /// exact touches; the kind touches retraction implies (§3.7 H-Q) are
    /// added only to the successor's append touches, at adoption.
    /// O(|segment| · log n + entries after it + content).
    pub(super) fn retract_chain(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        index: usize,
    ) -> Result<
        Vec<crate::mvcc::validation::proposal_touches::ValidatedMutationTouch>,
        RelationalCoupledAssemblyFailure,
    > {
        let superseded = crate::mvcc::validation::proposal_touches::project_applied_intents(
            execution,
            &self.mutation.state,
            &self.parts[index].commands.plan().merged_intents,
        )?;
        let span = self.parts[index].chain.span.clone();
        let effect = &self.mutation.outcome.effect;
        let records = effect.publication.changed_records[span.records.clone()].to_vec();
        let deltas = effect.adjacency.deltas[span.adjacency.clone()].to_vec();
        let mut others = OtherTouches::default();
        for other in self.live_spans_except(Some(index)) {
            for delta in &effect.adjacency.deltas[other.adjacency.clone()] {
                others.note(Some(delta), None);
            }
            for record in &effect.publication.changed_records[other.records.clone()] {
                others.note(None, Some(record));
            }
        }
        self.charge(
            execution,
            records.len() + deltas.len() + others.anchors.len(),
        )?;
        let chain = &self.parts[index].chain;
        let root = chain.head_input.resident_basis().basis();
        retract_chain_segment(
            execution,
            &mut self.mutation.state,
            &RetractionBase {
                image: root,
                journal: &chain.head_journal,
            },
            &records,
            &deltas,
            &others,
        )?;
        span.remove_from(&mut self.mutation.outcome.effect);
        for stage in &mut self.stages {
            stage.span.shift_after(&span);
        }
        for (other, part) in self.parts.iter_mut().enumerate() {
            if other != index {
                part.chain.span.shift_after(&span);
            }
        }
        let (entities, relations) = self.parts[index].chain.ledger.withdraw();
        self.mutation.outcome.created_entities.withdraw(&entities);
        self.mutation.outcome.created_relations.withdraw(&relations);
        let fresh = entities
            .iter()
            .map(|id| RecordRef::Entity(*id))
            .chain(relations.iter().map(|id| RecordRef::Relation(*id)))
            .collect::<Vec<_>>();
        self.fresh_records.forget(execution, &fresh)?;
        self.indexes.remove(MemberId::Part(index));
        self.parts[index].chain.span = MemberSpan::default();
        self.parts[index].content = Default::default();
        let attachments = self.live_attachments(execution)?;
        self.mutation.content = self.whole_content(execution, &attachments)?;
        self.recapture_aggregate();
        // The round reads the retracted image through `retracted_pin` only:
        // another chain's records may still name the ones retracted here
        // until the round re-creates them, so no pin of it is cached.
        self.read.replace(None);
        Ok(superseded)
    }
}
