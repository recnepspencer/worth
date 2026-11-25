//! Completion of a coupled candidate's succession (#119 §2.5b, §2.7, §2.10).
use worth_relational_publication::internal_port::mvcc::publication::application_publication::coupled_assembly::assembly_failure::RelationalCoupledAssemblyFailureOwnerPort as _;
use worth_relational_mutation::internal_port::mvcc::validation::proposal_identity::RelationalMutationProposalIdentityOwnerPort as _;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasisFieldsPort as _;
use super::super::*;
use worth_relational_contracts::internal_port::transactions::data::outcomes::commit_conflict::CommitConflictOwnerPort as _;

impl RelationalAuthoredCoupledCandidate<'_> {
    /// The union of every live member's attachments, each content record once.
    pub(super) fn live_attachments(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<crate::content::RelationalContentAttachments, TransactionCommitError> {
        let mut union = self.prefix_content.clone();
        let members = self
            .stages
            .iter()
            .map(|stage| &stage.content)
            .chain(self.parts.iter().map(|part| &part.content));
        for content in members {
            union
                .attach_retained_once(execution, content)
                .map_err(crate::content::publication::commit_denial)?;
        }
        Ok(union)
    }

    /// The whole candidate's content publication over the basis: every live
    /// changed record's references against `attachments`, as one round
    /// writing the whole candidate would prepare it. O(changed + content).
    pub(super) fn whole_content(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        attachments: &crate::content::RelationalContentAttachments,
    ) -> Result<crate::content::publication::ContentPublication, RelationalCoupledAssemblyFailure>
    {
        let runtime = self
            .runtime
            .with_candidate_symbols(self.mutation.candidate_symbols());
        let interpretation = self
            .identity
            .source_interpretation()
            .expect("owner-issued writer identity");
        crate::authority::commit::phases::applied_mutation::content::prepare(
            execution,
            &runtime,
            &self.binding.owners.before().owner_inner().root,
            &self.prepared.schema_authority,
            attachments,
            &self.mutation.state,
            &self.mutation.outcome.effect.publication.changed_records,
            self.identity.proposed_version_id(),
            interpretation.restoration_content(),
        )
        .map_err(RelationalCoupledAssemblyFailure::from_apply)
    }

    /// Recapture the aggregate proposal over the live accumulators.
    pub(super) fn recapture_aggregate(&mut self) {
        let publication = &self.mutation.outcome.effect.publication;
        self.mutation.aggregate = crate::aggregate_identity::AggregateProposal::capture(
            &self.prepared.schema_authority,
            &self.prepared.selected_branch_state,
            self.identity,
            &publication.changed_records,
            &self.mutation.outcome.effect.adjacency.deltas,
            publication.has_entity_deletions,
            None,
        );
    }

    /// Once, before final composition: cancel every identity no live round
    /// creates (N2), recompute deletion presence over live members, require
    /// exactly the surviving deltas' revision references (3c), and make the
    /// content the live members' union prepared against the basis (D6).
    pub(crate) fn complete_succession(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(), RelationalCoupledAssemblyFailure> {
        let conflict = |denial| {
            TransactionCommitError::conflict(crate::transactions::data::CommitConflict::new(
                crate::transactions::data::ConflictClass::RecordAllocationDenied { denial },
            ))
        };
        let dormant = self
            .parts
            .iter()
            .flat_map(|part| part.chain.ledger.dormant().cloned())
            .collect::<Vec<_>>();
        self.charge(
            execution,
            dormant.len() * (self.mutation.identities.records.reservation_count() + 1),
        )?;
        for record in &dormant {
            self.mutation
                .identities
                .records
                .cancel(record)
                .map_err(conflict)?;
        }
        let deletions = self
            .live_spans_except(None)
            .iter()
            .any(|span| span.has_entity_deletions);
        self.mutation
            .outcome
            .effect
            .publication
            .has_entity_deletions = deletions;
        let deltas = &self.mutation.outcome.effect.publication.canonical_deltas;
        self.charge(execution, deltas.len())?;
        worth_relational_mutation::internal_port::authority::mutation::delta_symbols::rebuild_required_revision_references(
            deltas,
            &mut self.mutation.identities.symbols,
        )
        .map_err(|denial| RelationalCoupledAssemblyFailure::from_apply(denial.into()))?;
        let attachments = self.live_attachments(execution)?;
        self.mutation.content = self.whole_content(execution, &attachments)?;
        self.transaction.replace_content(attachments);
        Ok(())
    }
}
