//! The native installation's publisher mint, retained by this graph only.
use worth_proof::InstalledApplicationPublicationOwner;
use worth_relational::facade::{
    mvcc::RelationalInstalledApplicationPublicationAuthority,
    runtime::RelationalInitialSchemaInstallationReceipt,
};

pub(in crate::domain_computation::primary_graph) struct ApplicationPublication {
    pub(in crate::domain_computation::primary_graph) owner: InstalledApplicationPublicationOwner,
    pub(in crate::domain_computation::primary_graph) native:
        RelationalInstalledApplicationPublicationAuthority,
}

impl ApplicationPublication {
    pub(super) fn installed(receipt: &RelationalInitialSchemaInstallationReceipt) -> Self {
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

impl super::WorthQueryPrimaryGraph {
    pub(in crate::domain_computation::primary_graph) fn bind_application_publication(
        &mut self,
        receipt: &RelationalInitialSchemaInstallationReceipt,
    ) {
        self.application_publication = Some(std::sync::Arc::new(
            ApplicationPublication::installed(receipt),
        ));
    }
}
