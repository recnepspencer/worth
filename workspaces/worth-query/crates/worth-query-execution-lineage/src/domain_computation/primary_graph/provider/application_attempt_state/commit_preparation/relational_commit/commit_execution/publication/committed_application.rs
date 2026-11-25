//! Durable committed-application evidence minted only from publication completion.

use std::fmt;
use std::sync::{Arc, OnceLock};
use worth_execution::ExecutionArray;
use worth_relational::facade::history::{BranchId, RelationalCommitReceipt};

type CoupledMembersArray = ExecutionArray<OnceLock<WorthQueryPrimaryGraphCommittedApplication>>;

trait CoupledMembersBacking: Send + Sync {
    fn as_members(&self) -> &CoupledMembersArray;
}

impl CoupledMembersBacking for CoupledMembersArray {
    fn as_members(&self) -> &CoupledMembersArray {
        self
    }
}

#[derive(Clone)]
struct SharedCoupledMembers(Arc<dyn CoupledMembersBacking>);

impl SharedCoupledMembers {
    fn from_arc(members: Arc<CoupledMembersArray>) -> Self {
        Self(members)
    }

    fn as_members(&self) -> &CoupledMembersArray {
        CoupledMembersBacking::as_members(&*self.0)
    }
}

impl PartialEq for SharedCoupledMembers {
    fn eq(&self, other: &Self) -> bool {
        self.as_members() == other.as_members()
    }
}

impl Eq for SharedCoupledMembers {}

impl fmt::Debug for SharedCoupledMembers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_members().fmt(f)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPrimaryGraphCommittedApplication {
    application_outcome_identity: Option<
        crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitOutcomeIdentity,
    >,
    coupled_members: Option<SharedCoupledMembers>,
    coupled_member: bool,
    runtime_instance_id: u64,
    changed_record_count: usize,
    emitted_effect_count: usize,
    basis_descriptor: worth_relational::facade::branch::RelationalBranchBasisDescriptor,
    commit_evidence: super::super::super::WorthQueryPrimaryGraphCommitEvidence,
    committed_product_publication: crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication,
    exact_output_settlement: Option<std::sync::Arc<crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity>>,
    /// The co-committed aftermath relation, sealed at publication so replay
    /// answers it after the commit's history retires.
    aftermath_causality: Option<crate::domain_computation::application_aftermath::WorthQueryCommittedAftermathCausality>,
}

impl WorthQueryPrimaryGraphCommittedApplication {
    pub fn publication_for_home_retirement(
        &mut self,
    ) -> &mut crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication {
        &mut self.committed_product_publication
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_publication_parts(
        runtime_instance_id: u64,
        changed_record_count: usize,
        emitted_effect_count: usize,
        outcome_identity: crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitOutcomeIdentity,
        basis_descriptor: worth_relational::facade::branch::RelationalBranchBasisDescriptor,
        evidence: super::super::super::WorthQueryPrimaryGraphCommitEvidence,
        product_publication: crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationReceipt,
    ) -> Self {
        let committed_product_publication = crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication::from_receipt(product_publication);
        Self {
            application_outcome_identity: Some(outcome_identity),
            coupled_members: None,
            coupled_member: false,
            runtime_instance_id,
            changed_record_count,
            emitted_effect_count,
            basis_descriptor,
            commit_evidence: evidence,
            committed_product_publication,
            exact_output_settlement: None,
            aftermath_causality: None,
        }
    }

    pub fn detach_coupled_delivery(&mut self) {
        self.committed_product_publication =
            self.committed_product_publication.co_committed_member();
        self.coupled_member = true;
    }
    pub fn is_coupled_member(&self) -> bool {
        self.coupled_member
    }
    pub fn retain_coupled_members(
        &mut self,
        members: std::sync::Arc<worth_execution::ExecutionArray<std::sync::OnceLock<Self>>>,
    ) {
        assert!(self
            .coupled_members
            .replace(SharedCoupledMembers::from_arc(members))
            .is_none());
    }
    pub fn complete_coupled_member(&self, ordinal: usize, member: Self) {
        self.coupled_members
            .as_ref()
            .expect("pre-effect group evidence slots")
            .as_members()[ordinal]
            .set(member)
            .unwrap_or_else(|_| panic!("each performed writer settles exactly once"));
    }
    pub fn publication_members(&self) -> impl Iterator<Item = &Self> {
        std::iter::once(self).chain(
            self.coupled_members
                .iter()
                .flat_map(|members| members.as_members().iter())
                .map(|member| {
                    member.get().expect(
                        "group settlement completes every writer before receipt publication",
                    )
                }),
        )
    }

    /// Takes the World history protection out of this copy. Evidence retained
    /// for receipt resolution and replay is detached, so replay still answers
    /// once the commit's history retires.
    pub fn take_history(
        &mut self,
    ) -> Option<
        crate::domain_computation::execution_runtime::product_world::WorthQueryCommitHistoryHold,
    > {
        self.committed_product_publication.take_history()
    }

    pub fn seal_aftermath_causality(
        &mut self,
        causality: crate::domain_computation::application_aftermath::WorthQueryCommittedAftermathCausality,
    ) {
        assert!(self.aftermath_causality.replace(causality).is_none());
    }

    pub const fn aftermath_causality(
        &self,
    ) -> Option<
        &crate::domain_computation::application_aftermath::WorthQueryCommittedAftermathCausality,
    > {
        self.aftermath_causality.as_ref()
    }

    /// Evidence whose commit's history retired keeps only the publication's
    /// identities, so replay still answers without pinning World or owner
    /// state.
    pub fn retire_publication(&mut self) {
        self.committed_product_publication = self.committed_product_publication.retired();
    }

    pub fn retain_exact_output_settlement(
        &mut self,
        identity: std::sync::Arc<
            crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity,
        >,
    ) {
        assert!(self.exact_output_settlement.replace(identity).is_none());
    }

    pub fn exact_output_settlement(
        &self,
    ) -> Option<
        &std::sync::Arc<
            crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity,
        >,
    > {
        self.exact_output_settlement.as_ref()
    }

    pub const fn committed_product_publication(
        &self,
    ) -> &crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication {
        &self.committed_product_publication
    }

    pub fn take_fresh_product_change(
        &self,
    ) -> Option<crate::domain_computation::execution_runtime::product_world::WorthQueryPerformedRelationalProductChange>{
        self.committed_product_publication
            .take_fresh_product_change()
    }

    pub const fn runtime_instance_id(&self) -> u64 {
        self.runtime_instance_id
    }

    pub const fn application_outcome_identity(
        &self,
    ) -> Option<
        crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitOutcomeIdentity,
    >{
        self.application_outcome_identity
    }

    pub const fn branch(&self) -> &BranchId {
        &self.commit_evidence.commit_reference().branch_id
    }

    pub const fn commit_reference(&self) -> &RelationalCommitReceipt {
        self.commit_evidence.commit_reference()
    }

    pub const fn basis_descriptor(
        &self,
    ) -> &worth_relational::facade::branch::RelationalBranchBasisDescriptor {
        &self.basis_descriptor
    }

    pub const fn changed_record_count(&self) -> usize {
        self.changed_record_count
    }

    pub const fn emitted_effect_count(&self) -> usize {
        self.emitted_effect_count
    }

    pub fn mutation_work(
        &self,
    ) -> Option<
        &crate::domain_computation::primary_graph::provider::WorthQueryPrimaryMutationWorkEvidence,
    > {
        Some(self.commit_evidence.mutation_work())
    }

    pub fn commit_evidence(&self) -> &super::super::super::WorthQueryPrimaryGraphCommitEvidence {
        &self.commit_evidence
    }
}

mod court_doors_for_test;
