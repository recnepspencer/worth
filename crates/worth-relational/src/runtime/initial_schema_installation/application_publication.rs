use super::RelationalInitialSchemaInstallationReceipt;

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
        let installed = crate::mvcc::RelationalInstalledApplicationPublicationAuthority {
            runtime_instance_id: self.runtime_instance_id,
            custom_invariant_generation: self.custom_invariant_generation,
            schema_digest: self.retained_schema_authority_digest,
            authority: owner.authority(),
        };
        (owner, installed)
    }
}
