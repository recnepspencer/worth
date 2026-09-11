//! One-way crossing from Query execution evidence into Bank publication.

use worth_query_host::facade::primary_graph::WorthQueryApplicationCommitReceipt;
use worth_query_host::facade::publication::domain_computation::{
    publish_application_commit, WorthQueryApplicationCommitPublicationReceipt,
};

use super::BankCommitReceipt;

pub(super) struct BankCommitPublicationProjection {
    publication: WorthQueryApplicationCommitPublicationReceipt,
    recovery_description: WorthQueryApplicationCommitReceipt,
}

impl BankCommitPublicationProjection {
    fn from_execution(execution: WorthQueryApplicationCommitReceipt) -> Self {
        // The fresh terminal is consumed by publication. Its descriptive clone
        // deliberately drops the move-only performed-product-change witness and
        // therefore cannot become a second publication authority lane.
        let recovery_description = execution.clone();
        let publication = publish_application_commit(execution).into_receipt();
        Self {
            publication,
            recovery_description,
        }
    }

    pub(super) fn into_parts(
        self,
    ) -> (
        WorthQueryApplicationCommitPublicationReceipt,
        WorthQueryApplicationCommitReceipt,
    ) {
        (self.publication, self.recovery_description)
    }
}

pub(crate) fn commit_receipt(execution: WorthQueryApplicationCommitReceipt) -> BankCommitReceipt {
    BankCommitReceipt::from_publication_projection(BankCommitPublicationProjection::from_execution(
        execution,
    ))
}
