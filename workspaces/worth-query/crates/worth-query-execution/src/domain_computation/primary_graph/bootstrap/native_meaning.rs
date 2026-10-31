//! Exclusive schema and custom-invariant installation on the actual Native owner.
use super::*;
use worth_relational::facade::runtime::RelationalInitialSchemaInstallationReceipt;

pub(in crate::domain_computation::primary_graph) struct InstalledNativeSchemaMeaning {
    pub(in crate::domain_computation::primary_graph) layout: WorthQueryPrimaryGraphLayout,
    pub(in crate::domain_computation::primary_graph) receipt:
        RelationalInitialSchemaInstallationReceipt,
    pub(in crate::domain_computation::primary_graph) expected_invariant_inventory_digest: [u8; 32],
}

impl WorthQueryExecutionInstallationAuthority {
    pub(in crate::domain_computation::primary_graph) fn prepare_native_schema<
        Schema: ApplicationSchema,
    >(
        &self,
        _phase: &super::super::WorthQueryBootstrapAdvancementPhase<'_>,
        runtime: &WorthQueryExecutionRuntime,
        installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
        relational_runtime: &mut RelationalRuntime,
        invariant_factories: WorthQueryApplicationInvariantFactories<Schema>,
    ) -> Result<InstalledNativeSchemaMeaning, WorthQueryPrimaryGraphInstallationDenial> {
        if !self.belongs_to(runtime) {
            return Err(primary_graph_denial(
                WorthQueryPrimaryGraphInstallationDenialKind::ForeignRuntime,
                "execution installation authority belongs to another runtime",
            ));
        }
        if runtime.primary_graph().is_some() {
            return Err(primary_graph_denial(
                WorthQueryPrimaryGraphInstallationDenialKind::AlreadyInstalled,
                "execution runtime already owns a primary graph",
            ));
        }
        runtime
            .installed_packages()
            .validate_application_schema(installed_schema)
            .map_err(|denial| {
                primary_graph_denial(
                    WorthQueryPrimaryGraphInstallationDenialKind::StaleInstalledSchema,
                    denial.subject(),
                )
            })?;
        let (layout, additions) = WorthQueryPrimaryGraphLayout::lower(
            installed_schema.installed_declaration(),
            installed_schema.native_contracts(),
            &relational_runtime.config().schema.registry,
        )?;
        let registrations =
            invariant_factories.lower(installed_schema.binding_identity(), &layout)?;
        let expected_invariant_inventory_digest =
            worth_relational::facade::runtime::custom_invariant_inventory_digest(&registrations);
        let invariant_installation_receipt = relational_runtime
            .prepare_initial_schema_installation()
            .map_err(map_initial_schema_installation_denial)?
            .install_with_custom_invariants(additions, registrations)
            .map_err(map_initial_schema_installation_denial)?;
        if invariant_installation_receipt.custom_invariant_inventory_digest()
            != &expected_invariant_inventory_digest
        {
            return Err(primary_graph_denial(
                WorthQueryPrimaryGraphInstallationDenialKind::InvariantInstallationReceiptMismatch,
                "Relational installed an invariant inventory outside the application catalog",
            ));
        }
        Ok(InstalledNativeSchemaMeaning {
            layout,
            receipt: invariant_installation_receipt,
            expected_invariant_inventory_digest,
        })
    }
}
