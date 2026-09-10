//! Durable committed-application evidence minted only from publication completion.

use worth_relational::facade::history::{BranchId, RelationalCommitReceipt};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) struct WorthQueryPrimaryGraphCommittedApplication {
    application_outcome_identity: Option<
        crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitOutcomeIdentity,
    >,
    runtime_instance_id: u64,
    changed_record_count: usize,
    emitted_effect_count: usize,
    basis_descriptor: worth_relational::facade::branch::RelationalBranchBasisDescriptor,
    commit_evidence: super::super::super::WorthQueryPrimaryGraphCommitEvidence,
    committed_product_publication: crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication,
    product_publication: crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationReceipt,
}

impl WorthQueryPrimaryGraphCommittedApplication {
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
        let committed_product_publication = crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication::from_receipt(product_publication.clone());
        Self {
            application_outcome_identity: Some(outcome_identity),
            runtime_instance_id,
            changed_record_count,
            emitted_effect_count,
            basis_descriptor,
            commit_evidence: evidence,
            committed_product_publication,
            product_publication,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn product_publication(
        &self,
    ) -> &worth_runtime_world::facade::ConsumedCompositePublication {
        self.product_publication.publication()
    }

    pub(in crate::domain_computation::primary_graph) const fn committed_product_publication(
        &self,
    ) -> &crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication {
        &self.committed_product_publication
    }

    pub(in crate::domain_computation::primary_graph) fn take_fresh_product_change(
        &self,
    ) -> Option<crate::domain_computation::execution_runtime::product_world::WorthQueryPerformedRelationalProductChange>{
        self.product_publication.take_fresh_delivery()
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

    pub(in crate::domain_computation::primary_graph) fn commit_evidence(
        &self,
    ) -> &super::super::super::WorthQueryPrimaryGraphCommitEvidence {
        &self.commit_evidence
    }
}
