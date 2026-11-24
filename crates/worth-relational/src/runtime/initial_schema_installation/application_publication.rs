use super::RelationalInitialSchemaInstallationReceipt;
use worth_relational_publication::internal_port::mvcc::publication::application_publication::RelationalInstalledApplicationPublicationAuthorityOwnerPort as _;

impl RelationalInitialSchemaInstallationReceipt {
    /// Issue the exact publisher occurrence to the installed application owner.
    /// The native counterpart remains bound to this schema and invariant install.
    pub fn issue_application_publication_owner(
        &self,
    ) -> (
        worth_proof::InstalledApplicationPublicationOwner,
        crate::mvcc::RelationalInstalledApplicationPublicationAuthority,
    ) {
        let owner = worth_proof::InstalledApplicationPublicationOwner::fresh();
        let installed =
            crate::mvcc::RelationalInstalledApplicationPublicationAuthority::from_installation(
                self.runtime_instance_id,
                self.custom_invariant_generation,
                self.retained_schema_authority_digest,
                owner.authority(),
                Default::default(),
            );
        (owner, installed)
    }
}
