//! The native installation's publisher mint, retained by this graph only.
use worth_proof::InstalledApplicationPublicationOwner;
use worth_relational::facade::{
    mvcc::RelationalInstalledApplicationPublicationAuthority,
    runtime::RelationalInitialSchemaInstallationReceipt,
};

pub struct ApplicationPublication {
    pub(in crate::domain_computation::primary_graph) owner: InstalledApplicationPublicationOwner,
    pub native: RelationalInstalledApplicationPublicationAuthority,
}

impl ApplicationPublication {
    pub(in crate::domain_computation::primary_graph) fn installed(
        receipt: &RelationalInitialSchemaInstallationReceipt,
    ) -> Self {
        let (owner, native) = receipt.issue_application_publication_owner();
        Self { owner, native }
    }

    pub(in crate::domain_computation::primary_graph) fn recovered(
        owner: InstalledApplicationPublicationOwner,
        native: RelationalInstalledApplicationPublicationAuthority,
    ) -> Self {
        Self { owner, native }
    }
}

impl super::WorthQueryPrimaryGraph {}

mod court_doors_for_test;
