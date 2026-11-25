//! Admit and apply one authenticated surviving writer's own successor commands.
use worth_relational_publication::internal_port::mvcc::publication::application_publication::coupled_assembly::producer_posture::RelationalCoupledProducerPostureOwnerPort as _;
use super::chain::Chain;
pub(super) use super::refusals::{reject, reject_extra_owner};
use super::*;
use crate::authority::commit::phases::prepare::prepare_authoritative_working_state_scope_for_base;
use crate::transactions::data::outcomes::plan_artifacts::AuthoritativeApplyPlan;
use crate::transactions::data::primitives::RecordRef;
use crate::transactions::data::primitives::WorkerIntentBatch;

mod suffix;
pub(super) use super::suffix_scope::SuffixWriter;

impl RelationalAuthoredCoupledCandidate<'_> {
    /// Append a producer part that declares no output names.
    pub fn append_publication(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        publication: RelationalCoupledProducerPosture,
        content: &crate::content::RelationalContentAttachments,
        batches: Vec<WorkerIntentBatch>,
    ) -> Result<(), RelationalCoupledAssemblyFailure> {
        self.append_named_publication(execution, publication, content, batches, None)
    }

    /// Append a producer part with the naming plan its attempt lowered. The
    /// plan's create targets are rescoped with the part's commands, and the
    /// envelope's one plan composes it at completion (`part_naming`).
    pub fn append_named_publication(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        mut publication: RelationalCoupledProducerPosture,
        content: &crate::content::RelationalContentAttachments,
        batches: Vec<WorkerIntentBatch>,
        mut naming: Option<crate::lineage::data::CommitNamingPlan>,
    ) -> Result<(), RelationalCoupledAssemblyFailure> {
        let predecessor_origin = super::participant_admission::admit(
            self.binding,
            self.runtime.services.runtime_instance_id(),
            &self.mutation.outcome.created_entities,
            &self.parts,
            execution,
            &publication,
        )?;
        let source = publication
            .producing_meaning()
            .expect("admitted writer")
            .source_scope;
        let index = self.parts.len();
        let scope = self.applied_parts();
        let head_journal = self.mutation.state().mutation_journal().clone();
        let start = super::succession::segment::SpanStart::of(&self.mutation.outcome.effect);
        let first_batch = self.transaction.batches().len();
        let applied = self.apply_suffix(
            execution,
            SuffixWriter::Producer {
                index,
                source,
                roles: publication.roles_mut(),
                predecessor: predecessor_origin.as_deref(),
            },
            content,
            batches,
            naming.as_mut(),
        )?;
        let span = start.end(&self.mutation.outcome.effect, applied.has_entity_deletions);
        let ordinary_ids = publication
            .resolve_ordinary_roles(execution, &self.mutation.outcome.created_entities)?;
        let references = super::succession::guards::named_records(
            &applied.plan.plan().merged_intents,
            publication.roles(),
            naming.as_ref(),
        );
        let head_ordinal = self.indexes.insert(
            super::succession::indexes::MemberId::Part(index),
            &applied.records,
            references,
        );
        let mut ledger = super::succession::ledger::ChainCreateLedger::default();
        ledger.bind_round(&applied.creates);
        let chain = Chain {
            head_input: applied.read.clone(),
            head_journal,
            head_ordinal,
            scope,
            prior_touches: Vec::new(),
            last_superseded: Vec::new(),
            batches: first_batch..self.transaction.batches().len(),
            superseded: Vec::new(),
            local_keys: applied.local_keys,
            ledger,
            touched: applied.records.iter().cloned().collect(),
            span,
            rounds: 1,
            latest: super::succession::appended_rounds(&self.parts),
        };
        self.parts.push(CoupledPublicationPart {
            inputs: None,
            ordinary_ids,
            input: applied.read,
            publication,
            commands: applied.plan,
            records: applied.records,
            chain,
            naming,
            content: content.clone(),
        });
        let latest = self.parts.last().expect("the actual part just appended");
        self.advance_pin(
            execution,
            &applied.delta_state,
            &applied.content,
            &latest.records,
            &latest.input,
        )
    }
}
