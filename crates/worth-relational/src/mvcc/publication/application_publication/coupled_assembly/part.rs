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
        let source = publication.producing_meaning().expect("admitted writer").source_scope;
        let index = self.parts.len();
        let pub_effect = &self.mutation.outcome.effect.publication;
        let (first_record, first_fragment) = (pub_effect.changed_records.len(), pub_effect.patch_fragments.len());
        let first_batch = self.transaction.batches().len();
        let applied = self.apply_suffix(
            execution,
            SuffixWriter::Producer { index, source, roles: publication.roles_mut(), predecessor: predecessor_origin.as_deref() },
            content, batches, naming.as_mut(),
        )?;
        let ordinary_ids = publication.resolve_ordinary_roles(execution, &self.mutation.outcome.created_entities)?;
        let publication_effect = &self.mutation.outcome.effect.publication;
        let latest = super::succession::appended_rounds(&self.parts);
        let chain = Chain {
            head_input: applied.read.clone(),
            prior_touches: Vec::new(),
            batches: first_batch..self.transaction.batches().len(),
            fragments: (first_fragment..publication_effect.patch_fragments.len()).map(|p| (p, applied.read.clone())).collect(),
            superseded: Vec::new(),
            local_keys: applied.local_keys,
            created: applied.created,
            rounds: vec![first_record..publication_effect.changed_records.len()],
            latest,
        };
        self.parts.push(CoupledPublicationPart {
            inputs: None, ordinary_ids, input: applied.read, publication,
            commands: applied.plan, records: applied.records, chain, naming,
        });
        let latest = self.parts.last().expect("the actual part just appended");
        self.advance_pin(execution, &applied.delta_state, &applied.content, &latest.records, &latest.input)
    }
}
