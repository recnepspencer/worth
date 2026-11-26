//! The names an unpublished coupled envelope gives its records so far, as a
//! later member of the same envelope reads them (#426).
//!
//! The envelope's members are applied one after another over one applied
//! mutation, each with its own naming plan. A later member reading a record
//! an earlier member created and named reads the name that record's birth is
//! minted as: the same preview of the same composed plan over the same
//! applied state that finalization validates and mints, guarded against a
//! naming basis that moved past the envelope's basis. Nothing is reserved,
//! allocated or written; an envelope that refuses or aborts leaves no trace.

use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis;
use worth_relational_mutation::internal_port::authority::commit::semantic_plan::meaning::RelationalSemanticPlan;
use worth_relational_mutation::internal_port::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity;

use super::{contracts, ProposalNameCarry, ProposalNameInputs, ProposalNameSource};
use crate::authority::commit::lineage_preparation::LineagePreparationAuthority;
use crate::authority::commit::phases::applied_mutation::AppliedMutationCore;

impl<'a> ProposalNameSource<'a> {
    /// The names `mutation`, the applied state of a coupled envelope
    /// proposed on `before`, gives its records under `naming_plan`: the
    /// envelope's plans composed so far, in the order completion composes
    /// them. Its proposed posture is the overlay a later member reads.
    /// `carry` is this same state's settled preview, if one was kept.
    ///
    /// Complexity: O(1); without a carried preview the preview runs once, on
    /// the first proposed read, in O(changed records + postures) plus
    /// minting each birth.
    pub fn for_coupled_candidate(
        lineage: LineagePreparationAuthority<'a>,
        before: &AdmittedRelationalBranchBasis,
        mutation: &'a AppliedMutationCore,
        plan: RelationalSemanticPlan<'a>,
        identity: &RelationalMutationProposalIdentity,
        naming_plan: Option<&'a contracts::lineage::data::CommitNamingPlan>,
        carry: ProposalNameCarry,
    ) -> Self {
        let proposal = ProposalNameInputs {
            branch: before.identity().branch_id().clone(),
            basis: before.observation().commit_id(),
            staged: mutation.state(),
            version_id: identity.proposed_version_id(),
            outcome: &mutation.outcome,
            naming_plan,
            graph_selection: plan.graph_selection(),
        };
        Self::new(lineage.naming, lineage.schema, proposal, carry)
    }
}
