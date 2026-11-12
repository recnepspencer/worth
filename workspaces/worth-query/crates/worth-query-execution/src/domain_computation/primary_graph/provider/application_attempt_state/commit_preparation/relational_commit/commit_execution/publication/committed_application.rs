//! Durable committed-application evidence minted only from publication completion.

use worth_relational::facade::history::{BranchId, RelationalCommitReceipt};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPrimaryGraphCommittedApplication {
    application_outcome_identity: Option<
        crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitOutcomeIdentity,
    >,
    coupled_members: Option<std::sync::Arc<worth_execution::ExecutionArray<std::sync::OnceLock<Self>>>>,
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
    pub(in crate::domain_computation::primary_graph) fn publication_for_home_retirement(
        &mut self,
    ) -> &mut crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication {
        &mut self.committed_product_publication
    }

    pub(super) fn from_publication(
        seal: super::WorthQueryCommittedApplicationPublicationSeal,
    ) -> Self {
        let super::WorthQueryCommittedApplicationPublicationSeal {
            runtime_instance_id,
            changed_record_count,
            emitted_effect_count,
            outcome_identity,
            basis_descriptor,
            evidence,
            product_publication,
        } = seal;
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

    pub(in crate::domain_computation::primary_graph::provider) fn detach_coupled_delivery(
        &mut self,
    ) {
        self.committed_product_publication =
            self.committed_product_publication.co_committed_member();
        self.coupled_member = true;
    }
    pub(in crate::domain_computation::primary_graph) fn is_coupled_member(&self) -> bool {
        self.coupled_member
    }
    pub(in crate::domain_computation::primary_graph) fn retain_coupled_members(
        &mut self,
        members: std::sync::Arc<worth_execution::ExecutionArray<std::sync::OnceLock<Self>>>,
    ) {
        assert!(self.coupled_members.replace(members).is_none());
    }
    pub(in crate::domain_computation::primary_graph) fn complete_coupled_member(
        &self,
        ordinal: usize,
        member: Self,
    ) {
        self.coupled_members
            .as_ref()
            .expect("pre-effect group evidence slots")[ordinal]
            .set(member)
            .unwrap_or_else(|_| panic!("each performed writer settles exactly once"));
    }
    pub(in crate::domain_computation::primary_graph) fn publication_members(
        &self,
    ) -> impl Iterator<Item = &Self> {
        std::iter::once(self).chain(
            self.coupled_members
                .iter()
                .flat_map(|members| members.iter())
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
    pub(in crate::domain_computation::primary_graph) fn take_history(
        &mut self,
    ) -> Option<
        crate::domain_computation::execution_runtime::product_world::WorthQueryCommitHistoryHold,
    > {
        self.committed_product_publication.take_history()
    }

    pub(in crate::domain_computation::primary_graph) fn seal_aftermath_causality(
        &mut self,
        causality: crate::domain_computation::application_aftermath::WorthQueryCommittedAftermathCausality,
    ) {
        assert!(self.aftermath_causality.replace(causality).is_none());
    }

    pub(in crate::domain_computation::primary_graph) const fn aftermath_causality(
        &self,
    ) -> Option<
        &crate::domain_computation::application_aftermath::WorthQueryCommittedAftermathCausality,
    > {
        self.aftermath_causality.as_ref()
    }

    /// Evidence whose commit's history retired keeps only the publication's
    /// identities, so replay still answers without pinning World or owner
    /// state.
    pub(in crate::domain_computation::primary_graph) fn retire_publication(&mut self) {
        self.committed_product_publication = self.committed_product_publication.retired();
    }

    pub(in crate::domain_computation::primary_graph) fn retain_exact_output_settlement(
        &mut self,
        identity: std::sync::Arc<
            crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity,
        >,
    ) {
        assert!(self.exact_output_settlement.replace(identity).is_none());
    }

    pub(in crate::domain_computation::primary_graph) fn exact_output_settlement(
        &self,
    ) -> Option<
        &std::sync::Arc<
            crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity,
        >,
    > {
        self.exact_output_settlement.as_ref()
    }

    pub(in crate::domain_computation::primary_graph) const fn committed_product_publication(
        &self,
    ) -> &crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication {
        &self.committed_product_publication
    }

    pub(in crate::domain_computation::primary_graph) fn take_fresh_product_change(
        &self,
    ) -> Option<crate::domain_computation::execution_runtime::product_world::WorthQueryPerformedRelationalProductChange>{
        self.committed_product_publication
            .take_fresh_product_change()
    }

    pub(in crate::domain_computation::primary_graph) const fn runtime_instance_id(&self) -> u64 {
        self.runtime_instance_id
    }

    pub(in crate::domain_computation::primary_graph) const fn application_outcome_identity(
        &self,
    ) -> Option<
        crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitOutcomeIdentity,
    >{
        self.application_outcome_identity
    }

    pub(in crate::domain_computation::primary_graph) const fn branch(&self) -> &BranchId {
        &self.commit_evidence.commit_reference().branch_id
    }

    pub(in crate::domain_computation::primary_graph) const fn commit_reference(
        &self,
    ) -> &RelationalCommitReceipt {
        self.commit_evidence.commit_reference()
    }

    pub(in crate::domain_computation::primary_graph) const fn basis_descriptor(
        &self,
    ) -> &worth_relational::facade::branch::RelationalBranchBasisDescriptor {
        &self.basis_descriptor
    }

    pub(in crate::domain_computation::primary_graph) const fn changed_record_count(&self) -> usize {
        self.changed_record_count
    }

    pub(in crate::domain_computation::primary_graph) const fn emitted_effect_count(&self) -> usize {
        self.emitted_effect_count
    }

    pub(in crate::domain_computation::primary_graph) fn mutation_work(
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
