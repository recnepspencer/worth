use super::{
    primary_graph_denial, ApplicationSchema, WorthQueryExecutionInstallationAuthority,
    WorthQueryExecutionRuntime, WorthQueryPrimaryGraphBootstrap,
    WorthQueryPrimaryGraphInstallationDenial, WorthQueryPrimaryGraphInstallationDenialKind,
};

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphBootstrap<Schema> {
    pub(super) fn validate_publication_target(
        &self,
        runtime: &WorthQueryExecutionRuntime,
        authority: &WorthQueryExecutionInstallationAuthority,
    ) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
        if self.runtime_authority != runtime.authority_identity() || !authority.belongs_to(runtime)
        {
            return Err(primary_graph_denial(
                WorthQueryPrimaryGraphInstallationDenialKind::ForeignRuntime,
                "primary graph bootstrap belongs to another execution runtime",
            ));
        }
        if runtime.primary_graph().is_some() {
            return Err(primary_graph_denial(
                WorthQueryPrimaryGraphInstallationDenialKind::AlreadyInstalled,
                "execution runtime already owns a primary graph",
            ));
        }
        if self.invariant_installation_receipt.runtime_instance_id()
            != self.graph.relational_runtime_instance_id()
            || self
                .invariant_installation_receipt
                .custom_invariant_generation()
                != 1
            || self
                .invariant_installation_receipt
                .custom_invariant_inventory_digest()
                != &self.expected_invariant_inventory_digest
        {
            return Err(primary_graph_denial(
                WorthQueryPrimaryGraphInstallationDenialKind::InvariantInstallationReceiptMismatch,
                "initial invariant installation receipt no longer matches the primary graph",
            ));
        }
        let declaration = Schema::declaration().map_err(|denial| {
            primary_graph_denial(
                WorthQueryPrimaryGraphInstallationDenialKind::StaleInstalledSchema,
                format!("{denial:?}"),
            )
        })?;
        let current = runtime
            .installed_packages()
            .bind_application_schema(declaration)
            .map_err(|denial| {
                primary_graph_denial(
                    WorthQueryPrimaryGraphInstallationDenialKind::StaleInstalledSchema,
                    denial.subject(),
                )
            })?;
        if current.binding_identity() != *self.graph.binding_identity() {
            return Err(primary_graph_denial(
                WorthQueryPrimaryGraphInstallationDenialKind::StaleInstalledSchema,
                "installed schema generation changed after bootstrap preparation",
            ));
        }
        Ok(())
    }
}
